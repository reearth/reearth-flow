package interactor

import (
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestDeployment_SavesAVersion(t *testing.T) {
	t.Parallel()

	t.Run("deploying a project saves a deploy version", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		ws := &fakeWebsocket{}
		pid := f.project.ID()

		_, err := NewDeployment(f.repo, f.gw, f.job, f.perm, ws).Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})

		require.NoError(t, err)
		assert.Equal(t, []string{pid.String() + " deploy"}, ws.versions)
	})

	t.Run("a deployment without a project saves none", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		ws := &fakeWebsocket{}

		_, err := NewDeployment(f.repo, f.gw, f.job, f.perm, ws).Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})

		require.NoError(t, err)
		assert.Empty(t, ws.versions)
	})

	t.Run("only an update with a new workflow file saves one", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		ws := &fakeWebsocket{}
		i := NewDeployment(f.repo, f.gw, f.job, f.perm, ws)
		pid := f.project.ID()
		d, err := i.Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})
		require.NoError(t, err)
		ws.versions = nil

		desc := "renamed"
		_, err = i.Update(f.ctx, interfaces.UpdateDeploymentParam{ID: d.ID(), Description: &desc})
		require.NoError(t, err)
		assert.Empty(t, ws.versions, "a description change is not a deployment of the workflow")

		_, err = i.Update(f.ctx, interfaces.UpdateDeploymentParam{ID: d.ID(), Workflow: &file.File{Path: "workflow.json"}})
		require.NoError(t, err)
		assert.Equal(t, []string{pid.String() + " deploy"}, ws.versions)
	})

	t.Run("a failed version save does not fail the deployment", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		ws := &fakeWebsocket{versionErr: errors.New("websocket down")}
		pid := f.project.ID()

		d, err := NewDeployment(f.repo, f.gw, f.job, f.perm, ws).Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})

		require.NoError(t, err)
		assert.NotNil(t, d)
		assert.Len(t, ws.versions, 1)
	})
}

func newVersionTestProjectInteractor(ws *fakeWebsocket) (*Project, *memory.Job) {
	jobRepo := memory.NewJob()
	return &Project{
		projectRepo:       memory.NewProject(),
		jobRepo:           jobRepo,
		websocket:         ws,
		file:              &previewFakeFile{},
		cloudRunWorker:    &stubCloudRunWorker{},
		job:               &previewFakeJob{},
		transaction:       usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
		permissionChecker: NewMockPermissionChecker(nil),
		planLimits:        planLimits{jobRepo: jobRepo, now: time.Now},
	}, jobRepo
}

func TestProject_Run_SavesARunVersion(t *testing.T) {
	t.Parallel()

	t.Run("a debug run saves a run version", func(t *testing.T) {
		t.Parallel()
		ws := &fakeWebsocket{}
		uc, _ := newVersionTestProjectInteractor(ws)
		prj := newPreviewProject(t, uc.projectRepo)

		_, err := uc.Run(previewTestContext(), interfaces.RunProjectParam{ProjectID: prj.ID(), Workflow: newWorkflowFile()})

		require.NoError(t, err)
		assert.Equal(t, []string{prj.ID().String() + " run"}, ws.versions)
	})

	t.Run("a failed version save does not stop the run", func(t *testing.T) {
		t.Parallel()
		ws := &fakeWebsocket{versionErr: errors.New("websocket down")}
		uc, _ := newVersionTestProjectInteractor(ws)
		prj := newPreviewProject(t, uc.projectRepo)

		got, err := uc.Run(previewTestContext(), interfaces.RunProjectParam{ProjectID: prj.ID(), Workflow: newWorkflowFile()})

		require.NoError(t, err)
		assert.NotNil(t, got)
	})

	t.Run("a schema preview saves none", func(t *testing.T) {
		t.Parallel()
		ws := &fakeWebsocket{}
		uc, _ := newVersionTestProjectInteractor(ws)
		prj := newPreviewProject(t, uc.projectRepo)

		_, err := uc.PreviewSchema(previewTestContext(), interfaces.PreviewSchemaParam{ProjectID: prj.ID(), Workflow: newWorkflowFile()})

		require.NoError(t, err)
		assert.Empty(t, ws.versions)
	})
}
