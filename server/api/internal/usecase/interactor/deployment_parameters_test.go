package interactor

import (
	"context"
	"errors"
	"net/url"
	"testing"
	"time"

	"github.com/google/uuid"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// workflowFile accepts every upload, which is all a deploy or a run needs from
// file storage.
type workflowFile struct{ gateway.File }

func (workflowFile) UploadWorkflow(context.Context, *file.File) (*url.URL, error) {
	return url.Parse("https://example.com/workflows/" + uuid.NewString() + ".json")
}

func (workflowFile) RemoveWorkflow(context.Context, *url.URL) error { return nil }

func (workflowFile) UploadMetadata(context.Context, string, []string) (*url.URL, error) {
	return nil, nil
}

var errSubmitRefused = errors.New("batch: submit refused")

// recordingBatch keeps the variables a run is submitted with and refuses the
// job, so a run stops right after the point the variables are decided.
type recordingBatch struct {
	gateway.Batch
	submitted []map[string]string
}

func (b *recordingBatch) SubmitJob(_ context.Context, _ id.JobID, _ string, _ string, vars map[string]string, _ id.ProjectID, _ accountsid.WorkspaceID, _ *id.JobID, _ *uuid.UUID) (string, error) {
	b.submitted = append(b.submitted, vars)
	return "", errSubmitRefused
}

type paramFixture struct {
	ctx     context.Context
	job     interfaces.Job
	perm    gateway.PermissionChecker
	repo    *repo.Container
	gw      *gateway.Container
	batch   *recordingBatch
	plan    *planChecker
	project *project.Project
	ws      accountsid.WorkspaceID
}

func newParamFixture(t *testing.T) *paramFixture {
	t.Helper()
	ctx := adapter.AttachAuthInfo(context.Background(), &appx.AuthInfo{Token: "token"})
	ctx = adapter.AttachUser(ctx, accountsuser.New().NewID().Name("user").Email("user@example.com").MustBuild())

	ws := accountsid.NewWorkspaceID()
	r := memory.New()
	r.Job = memory.NewJob()
	p := project.New().NewID().Workspace(ws).Name("p").MustBuild()
	require.NoError(t, r.Project.Save(ctx, p))

	batch := &recordingBatch{}
	plan := &planChecker{}
	gw := &gateway.Container{PolicyChecker: plan, File: workflowFile{}, Batch: batch}
	perm := NewMockPermissionChecker(func(context.Context, string, string) (bool, error) { return true, nil })
	return &paramFixture{
		ctx: ctx, ws: ws, repo: r, gw: gw, batch: batch, plan: plan, project: p,
		job: NewJob(r, gw, perm), perm: perm,
	}
}

// param saves a workflow variable on the fixture's project.
func (f *paramFixture) param(t *testing.T, name string, index int, required bool, value any) *parameter.Parameter {
	t.Helper()
	p, err := parameter.New().ID(id.NewParameterID()).ProjectID(f.project.ID()).Name(name).
		Type(parameter.TypeText).Index(index).Required(required).DefaultValue(value).Build()
	require.NoError(t, err)
	require.NoError(t, f.repo.Parameter.Save(f.ctx, p))
	return p
}

// deployment saves a deployment of the fixture's project that carries params.
func (f *paramFixture) deployment(t *testing.T, params ...*parameter.Parameter) *deployment.Deployment {
	t.Helper()
	pid := f.project.ID()
	d := deployment.New().NewID().Workspace(f.ws).Project(&pid).WorkflowURL("workflow.json").Version("v1").
		Parameters(params).MustBuild()
	require.NoError(t, f.repo.Deployment.Save(f.ctx, d))
	return d
}

// trigger saves a scheduled trigger directly, skipping the checks Create makes.
func (f *paramFixture) trigger(t *testing.T, d *deployment.Deployment, vars ...variable.Variable) *trigger.Trigger {
	t.Helper()
	b := trigger.New().NewID().Workspace(f.ws).Deployment(d.ID()).Description("t").Enabled(true).
		EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay).
		CreatedAt(time.Now()).UpdatedAt(time.Now())
	if len(vars) > 0 {
		b = b.Variables(vars)
	}
	tr := b.MustBuild()
	require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
	return tr
}

func parameterNames(params []*parameter.Parameter) []string {
	names := make([]string, 0, len(params))
	for _, p := range params {
		names = append(names, p.Name())
	}
	return names
}

func TestDeployment_KeepsTheProjectParameters(t *testing.T) {
	t.Run("a deploy keeps the project's variables in their order", func(t *testing.T) {
		f := newParamFixture(t)
		f.param(t, "second", 1, false, "b")
		f.param(t, "first", 0, true, "a")
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)
		pid := f.project.ID()

		d, err := i.Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})

		require.NoError(t, err)
		assert.Equal(t, []string{"first", "second"}, parameterNames(d.Parameters()))
		assert.True(t, d.Parameters()[0].Required())
	})

	t.Run("a deploy without a project keeps none", func(t *testing.T) {
		f := newParamFixture(t)
		f.param(t, "first", 0, false, "a")
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)

		d, err := i.Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})

		require.NoError(t, err)
		assert.Empty(t, d.Parameters())
	})

	t.Run("only a new workflow file refreshes them", func(t *testing.T) {
		f := newParamFixture(t)
		original := f.param(t, "first", 0, false, "deployed")
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)
		pid := f.project.ID()
		d, err := i.Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})
		require.NoError(t, err)

		// The project's variable changes after the deploy.
		changed, err := parameter.New().ID(original.ID()).ProjectID(pid).Name("first").Type(parameter.TypeText).
			DefaultValue("edited").Build()
		require.NoError(t, err)
		require.NoError(t, f.repo.Parameter.Save(f.ctx, changed))

		desc := "renamed"
		d, err = i.Update(f.ctx, interfaces.UpdateDeploymentParam{ID: d.ID(), Description: &desc})
		require.NoError(t, err)
		assert.Equal(t, "deployed", d.Parameters()[0].DefaultValue(), "a description change keeps the deployed variables")

		d, err = i.Update(f.ctx, interfaces.UpdateDeploymentParam{ID: d.ID(), Workflow: &file.File{Path: "workflow.json"}})
		require.NoError(t, err)
		assert.Equal(t, "edited", d.Parameters()[0].DefaultValue(), "a new workflow file brings the project's variables as they are now")
	})

	t.Run("editing the project's variable leaves the deployed one as it was", func(t *testing.T) {
		f := newParamFixture(t)
		original := f.param(t, "first", 0, false, "deployed")
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)
		pid := f.project.ID()
		d, err := i.Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})
		require.NoError(t, err)

		_, err = NewParameter(f.repo, f.perm).UpdateParameter(f.ctx, interfaces.UpdateParameterParam{
			ParamID: original.ID(), NameValue: "first", TypeValue: parameter.TypeText, DefaultValue: "edited",
		})
		require.NoError(t, err)

		stored, err := f.repo.Deployment.FindByID(f.ctx, d.ID())
		require.NoError(t, err)
		assert.Equal(t, "deployed", stored.Parameters()[0].DefaultValue())
	})

	t.Run("a project of another workspace is refused", func(t *testing.T) {
		f := newParamFixture(t)
		other := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).Name("other").MustBuild()
		require.NoError(t, f.repo.Project.Save(f.ctx, other))
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)
		pid := other.ID()

		_, err := i.Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})

		assert.ErrorIs(t, err, rerror.ErrNotFound)
	})

	t.Run("a failed deploy removes the workflow it uploaded", func(t *testing.T) {
		f := newParamFixture(t)
		files := &removingFile{}
		f.gw.File = files
		f.repo.Parameter = failingParameters{Parameter: f.repo.Parameter}
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)
		pid := f.project.ID()

		_, err := i.Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})

		assert.ErrorIs(t, err, errParametersUnavailable)
		require.Len(t, files.uploaded, 1)
		assert.Equal(t, files.uploaded, files.removed)
	})

	t.Run("a failed commit keeps the workflow the row may still reference", func(t *testing.T) {
		f := newParamFixture(t)
		files := &removingFile{}
		f.gw.File = files
		f.repo.Transaction = usecasex.NewTransactor(&usecasex.NopTransaction{CommitError: errCommitFailed}, 0)
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)
		pid := f.project.ID()

		_, err := i.Create(f.ctx, interfaces.CreateDeploymentParam{
			Workspace: f.ws, Project: &pid, Workflow: &file.File{Path: "workflow.json"}, Description: "d",
		})

		assert.ErrorIs(t, err, errCommitFailed)
		require.Len(t, files.uploaded, 1)
		assert.Empty(t, files.removed)
	})
}

var (
	errParametersUnavailable = errors.New("parameters unavailable")
	errCommitFailed          = errors.New("commit failed")
)

// failingParameters fails to list a project's variables.
type failingParameters struct{ repo.Parameter }

func (failingParameters) FindByProject(context.Context, id.ProjectID) (*parameter.ParameterList, error) {
	return nil, errParametersUnavailable
}

// removingFile records the workflows it uploads and removes.
type removingFile struct {
	workflowFile
	uploaded []string
	removed  []string
}

func (f *removingFile) UploadWorkflow(ctx context.Context, w *file.File) (*url.URL, error) {
	u, err := f.workflowFile.UploadWorkflow(ctx, w)
	if err == nil {
		f.uploaded = append(f.uploaded, u.String())
	}
	return u, err
}

func (f *removingFile) RemoveWorkflow(_ context.Context, u *url.URL) error {
	f.removed = append(f.removed, u.String())
	return nil
}

func TestDeployment_ExecuteRequiresRequiredVariables(t *testing.T) {
	t.Run("an empty required default refuses the run before the plan is asked", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, ""))
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)

		_, err := i.Execute(f.ctx, interfaces.ExecuteDeploymentParam{DeploymentID: d.ID()})

		assert.ErrorIs(t, err, interfaces.ErrRequiredVariableMissing)
		assert.ErrorContains(t, err, "area")
		assert.Empty(t, f.plan.asked)
	})

	t.Run("a filled required default runs", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, "tokyo"))
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)

		_, err := i.Execute(f.ctx, interfaces.ExecuteDeploymentParam{DeploymentID: d.ID()})

		// The batch refuses every job, which shows the run got past the check.
		assert.ErrorIs(t, err, interfaces.ErrJobCreationFailed)
		// A manual run sends no variables: the worker takes the deployed
		// defaults from the workflow file's own `with`.
		require.Len(t, f.batch.submitted, 1)
		assert.Empty(t, f.batch.submitted[0])
	})
}

func TestTrigger_RequiredVariables(t *testing.T) {
	area := func(v any) variable.Variable {
		return variable.Variable{Key: "area", Type: parameter.TypeText, Value: v}
	}

	t.Run("a trigger that leaves a required variable empty is not created", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: d.ID(), Description: "t", Enabled: true,
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryDay,
			Variables: []variable.Variable{area("  ")},
		})

		assert.ErrorIs(t, err, interfaces.ErrRequiredVariableMissing)
	})

	t.Run("a trigger's own value fills a required variable", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		tr, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: d.ID(), Description: "t", Enabled: true,
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryDay,
			Variables: []variable.Variable{area("tokyo")},
		})

		require.NoError(t, err)
		assert.Equal(t, "tokyo", tr.Variables()[0].Value)
	})

	t.Run("an update that empties a required variable is not saved", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		tr := f.trigger(t, d, area("tokyo"))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{ID: tr.ID(), Variables: []variable.Variable{}})

		assert.ErrorIs(t, err, interfaces.ErrRequiredVariableMissing)
		stored, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
		require.NoError(t, err)
		assert.Equal(t, "tokyo", stored.Variables()[0].Value)
	})

	t.Run("an API trigger is created with a required variable its calls will fill", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: d.ID(), Description: "t", Enabled: true,
			EventSource: "API_DRIVEN", AuthToken: "secret",
		})

		require.NoError(t, err)
	})

	t.Run("an API trigger's update is saved with a required variable empty", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		tr := f.trigger(t, d, area("tokyo"))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{
			ID: tr.ID(), EventSource: "API_DRIVEN", AuthToken: "secret", Variables: []variable.Variable{},
		})

		require.NoError(t, err)
		stored, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
		require.NoError(t, err)
		assert.Equal(t, trigger.EventSourceType("API_DRIVEN"), stored.EventSource())
		assert.Empty(t, stored.Variables())
	})

	t.Run("an API trigger switched to a schedule must fill its required variables", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		tr := trigger.New().NewID().Workspace(f.ws).Deployment(d.ID()).Description("t").Enabled(true).
			EventSource("API_DRIVEN").AuthToken("secret").CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{
			ID: tr.ID(), EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryDay,
		})

		assert.ErrorIs(t, err, interfaces.ErrRequiredVariableMissing)
	})

	t.Run("a scheduled run with a required variable empty is refused before the plan is asked", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		tr := f.trigger(t, d)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrRequiredVariableMissing)
		assert.Empty(t, f.plan.asked)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("an API call that leaves a required variable empty is refused before the plan is asked", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		tr := trigger.New().NewID().Workspace(f.ws).Deployment(d.ID()).Description("t").Enabled(true).
			EventSource("API_DRIVEN").AuthToken("secret").CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{
			TriggerID: tr.ID(), AuthenticationToken: "secret", Variables: map[string]interface{}{"area": " "},
		})

		assert.ErrorIs(t, err, interfaces.ErrRequiredVariableMissing)
		assert.Empty(t, f.plan.asked)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("an API call can fill a required variable", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, true, nil))
		tr := trigger.New().NewID().Workspace(f.ws).Deployment(d.ID()).Description("t").Enabled(true).
			EventSource("API_DRIVEN").AuthToken("secret").CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{
			TriggerID: tr.ID(), AuthenticationToken: "secret", Variables: map[string]interface{}{"area": "osaka"},
		})

		assert.ErrorIs(t, err, interfaces.ErrJobCreationFailed)
		require.Len(t, f.batch.submitted, 1)
		assert.Equal(t, "osaka", f.batch.submitted[0]["area"])
	})
}

func TestTrigger_RunsUseTheDeployedVariables(t *testing.T) {
	t.Run("the deployed default, not the project's current one", func(t *testing.T) {
		f := newParamFixture(t)
		deployed := f.param(t, "area", 0, false, "deployed")
		d := f.deployment(t, deployed)
		// The project's variable changes after the deploy.
		changed, err := parameter.New().ID(deployed.ID()).ProjectID(f.project.ID()).Name("area").
			Type(parameter.TypeText).DefaultValue("edited").Build()
		require.NoError(t, err)
		require.NoError(t, f.repo.Parameter.Save(f.ctx, changed))
		tr := f.trigger(t, d)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err = i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrJobCreationFailed)
		require.Len(t, f.batch.submitted, 1)
		assert.Equal(t, "deployed", f.batch.submitted[0]["area"])
	})

	t.Run("the trigger's own value over the deployed default", func(t *testing.T) {
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, false, "deployed"))
		tr := f.trigger(t, d, variable.Variable{Key: "area", Type: parameter.TypeText, Value: "mine"})
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrJobCreationFailed)
		require.Len(t, f.batch.submitted, 1)
		assert.Equal(t, "mine", f.batch.submitted[0]["area"])
	})
}
