package interactor

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Parameters sent with a run carry no project; the job records them as the
// project's, so the snapshot reads back from every repository.
func TestProject_Run_RecordsParametersWithProject(t *testing.T) {
	ctx := previewTestContext()
	projectRepo := memory.NewProject()
	jobRepo := memory.NewJob()
	fj := &previewFakeJob{}
	prj := newPreviewProject(t, projectRepo)

	uc := &Project{
		projectRepo:       projectRepo,
		jobRepo:           jobRepo,
		websocket:         &fakeWebsocket{},
		file:              &previewFakeFile{},
		cloudRunWorker:    &stubCloudRunWorker{},
		job:               fj,
		transaction:       usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
		permissionChecker: NewMockPermissionChecker(nil),
		planLimits:        planLimits{jobRepo: jobRepo, now: time.Now},
	}

	sent, err := parameter.New().Name("area").Type(parameter.TypeText).DefaultValue("osaka").Build()
	require.NoError(t, err)

	got, err := uc.Run(ctx, interfaces.RunProjectParam{
		ProjectID:  prj.ID(),
		Workflow:   newWorkflowFile(),
		Parameters: []*parameter.Parameter{sent},
	})
	require.NoError(t, err)

	saved, err := jobRepo.FindByID(ctx, got.ID())
	require.NoError(t, err)
	require.Len(t, saved.Parameters(), 1)
	assert.Equal(t, sent.ID(), saved.Parameters()[0].ID())
	assert.Equal(t, prj.ID(), saved.Parameters()[0].ProjectID())
	assert.Equal(t, "osaka", saved.Parameters()[0].DefaultValue())
	assert.Zero(t, sent.ProjectID(), "the caller's parameters are not modified")
}

func TestProject_PreviewSchema_RecordsParametersWithProject(t *testing.T) {
	ctx := previewTestContext()
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
	}

	sent, err := parameter.New().Name("area").Type(parameter.TypeText).DefaultValue("osaka").Build()
	require.NoError(t, err)

	got, err := uc.PreviewSchema(ctx, interfaces.PreviewSchemaParam{
		ProjectID:  prj.ID(),
		Workflow:   newWorkflowFile(),
		Parameters: []*parameter.Parameter{sent},
	})
	require.NoError(t, err)

	saved, err := jobRepo.FindByID(ctx, got.ID())
	require.NoError(t, err)
	require.Len(t, saved.Parameters(), 1)
	assert.Equal(t, prj.ID(), saved.Parameters()[0].ProjectID())
}

func TestWithProject(t *testing.T) {
	t.Parallel()
	own := parameter.NewProjectID()
	pid := parameter.NewProjectID()
	bare, _ := parameter.New().Name("bare").Build()
	owned, _ := parameter.New().Name("owned").ProjectID(own).Build()

	got := withProject([]*parameter.Parameter{bare, nil, owned}, &pid)

	require.Len(t, got, 3)
	assert.Equal(t, pid, got[0].ProjectID(), "a parameter with no project is given one")
	assert.Nil(t, got[1])
	assert.Equal(t, own, got[2].ProjectID(), "a parameter's own project is kept")
	assert.Zero(t, bare.ProjectID(), "the originals are not modified")

	assert.Equal(t, []*parameter.Parameter{bare}, withProject([]*parameter.Parameter{bare}, nil))
	assert.Nil(t, withProject(nil, &pid))
}
