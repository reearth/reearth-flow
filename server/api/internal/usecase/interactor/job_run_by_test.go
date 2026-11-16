package interactor

import (
	"context"
	"testing"
	"time"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestJobRunBy(t *testing.T) {
	t.Parallel()

	// The fixture's batch refuses every job, so each run below fails after its
	// job is saved; the saved job is what is checked.

	t.Run("a deployment run records the user who ran it", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		d := f.deployment(t)
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)

		_, _ = i.Execute(f.ctx, interfaces.ExecuteDeploymentParam{DeploymentID: d.ID()})

		assertRunBy(t, f.recordedJob(t), adapter.User(f.ctx))
	})

	t.Run("a CMS run records the user CMS ran it for", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		tr := f.cmsTrigger(t, f.deployment(t), true)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, _ = i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{TriggerID: tr.ID(), Origin: cmsOrigin()})

		assertRunBy(t, f.recordedJob(t), adapter.User(f.ctx))
	})

	t.Run("an API trigger run records no user", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		d := f.deployment(t)
		tr := trigger.New().NewID().Workspace(f.ws).Deployment(d.ID()).Description("t").Enabled(true).
			EventSource(trigger.EventSourceTypeAPIDriven).AuthTokenHash(trigger.HashAuthToken("secret")).
			CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, _ = i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID(), AuthenticationToken: "secret"})

		assert.Nil(t, f.recordedJob(t).RunBy(), "the trigger's token, not a user, started it")
	})

	t.Run("a scheduled run records no user", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, _ = i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.Nil(t, f.recordedJob(t).RunBy(), "the schedule, not a user, started it")
	})

	t.Run("a debug run records the user who ran it", func(t *testing.T) {
		t.Parallel()
		u, ctx := runByUserContext()
		uc, jobRepo, prj := newRunByProjectInteractor(t)

		got, err := uc.Run(ctx, interfaces.RunProjectParam{ProjectID: prj, Workflow: newWorkflowFile()})
		require.NoError(t, err)

		saved, err := jobRepo.FindByID(ctx, got.ID())
		require.NoError(t, err)
		assertRunBy(t, saved, u)
	})

	t.Run("a schema preview records the user who asked for it", func(t *testing.T) {
		t.Parallel()
		u, ctx := runByUserContext()
		uc, jobRepo, prj := newRunByProjectInteractor(t)

		got, err := uc.PreviewSchema(ctx, interfaces.PreviewSchemaParam{ProjectID: prj, Workflow: newWorkflowFile()})
		require.NoError(t, err)

		saved, err := jobRepo.FindByID(ctx, got.ID())
		require.NoError(t, err)
		assertRunBy(t, saved, u)
	})
}

func assertRunBy(t *testing.T, j *job.Job, u *accountsuser.User) {
	t.Helper()
	require.NotNil(t, u)
	require.NotNil(t, j.RunBy())
	assert.Equal(t, u.ID(), *j.RunBy())
}

func runByUserContext() (*accountsuser.User, context.Context) {
	u := accountsuser.New().NewID().Name("runner").Email("runner@example.com").MustBuild()
	return u, adapter.AttachUser(previewTestContext(), u)
}

func newRunByProjectInteractor(t *testing.T) (*Project, *memory.Job, id.ProjectID) {
	t.Helper()
	projectRepo := memory.NewProject()
	jobRepo := memory.NewJob()
	prj := newPreviewProject(t, projectRepo)
	uc := &Project{
		projectRepo:       projectRepo,
		jobRepo:           jobRepo,
		websocket:         &fakeWebsocket{},
		file:              &previewFakeFile{},
		cloudRunWorker:    &stubCloudRunWorker{},
		job:               &previewFakeJob{},
		transaction:       usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
		permissionChecker: NewMockPermissionChecker(nil),
		planLimits:        planLimits{jobRepo: jobRepo, now: time.Now},
	}
	return uc, jobRepo, prj.ID()
}
