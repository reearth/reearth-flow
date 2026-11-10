package internalapi

import (
	"context"
	"fmt"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/structpb"
)

// fakeTriggers records the CMS calls; other methods panic.
type fakeTriggers struct {
	interfaces.Trigger
	runErr   error
	ran      *interfaces.ExecuteCMSTriggerParam
	filter   *interfaces.TriggerFilter
	triggers []*trigger.Trigger
}

func (f *fakeTriggers) FindByWorkspace(_ context.Context, _ accountsid.WorkspaceID, _ *interfaces.PaginationParam, filter interfaces.TriggerFilter) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error) {
	f.filter = &filter
	return f.triggers, interfaces.NewPageBasedInfo(int64(len(f.triggers)), 1, 20), nil
}

func (f *fakeTriggers) ExecuteCMSTrigger(_ context.Context, p interfaces.ExecuteCMSTriggerParam) (*job.Job, error) {
	f.ran = &p
	if f.runErr != nil {
		return nil, f.runErr
	}
	tid := p.TriggerID
	return job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Trigger(&tid).Origin(&p.Origin).
		Status(job.StatusPending).StartedAt(time.Now()).MustBuild(), nil
}

type fakeDeployments struct {
	interfaces.Deployment
	deployments []*deployment.Deployment
}

func (f *fakeDeployments) Fetch(_ context.Context, ids []id.DeploymentID) ([]*deployment.Deployment, error) {
	res := make([]*deployment.Deployment, len(ids))
	for i, did := range ids {
		for _, d := range f.deployments {
			if d.ID() == did {
				res[i] = d
			}
		}
	}
	return res, nil
}

type fakeProjectFetch struct {
	interfaces.Project
	projects []*project.Project
}

func (f *fakeProjectFetch) Fetch(_ context.Context, ids []id.ProjectID) ([]*project.Project, error) {
	res := make([]*project.Project, len(ids))
	for i, pid := range ids {
		for _, p := range f.projects {
			if p.ID() == pid {
				res[i] = p
			}
		}
	}
	return res, nil
}

type fakeJobs struct {
	interfaces.Job
	fetchErr  error
	cancelErr error
	jobs      []*job.Job
	refreshed []id.JobID
}

func (f *fakeJobs) Fetch(_ context.Context, ids []id.JobID) ([]*job.Job, error) {
	if f.fetchErr != nil {
		return nil, f.fetchErr
	}
	res := make([]*job.Job, len(ids))
	for i, jid := range ids {
		for _, j := range f.jobs {
			if j.ID() == jid {
				res[i] = j
			}
		}
	}
	return res, nil
}

func (f *fakeJobs) Refresh(_ context.Context, j *job.Job) (*job.Job, error) {
	f.refreshed = append(f.refreshed, j.ID())
	return j, nil
}

func (f *fakeJobs) Cancel(context.Context, id.JobID) (*job.Job, error) {
	return nil, f.cancelErr
}

func withUsecases(c *interfaces.Container) context.Context {
	return adapter.AttachUsecases(context.Background(), c)
}

func cmsItemOrigin() *flowv1.JobOrigin {
	return &flowv1.JobOrigin{Origin: &flowv1.JobOrigin_CmsItem{CmsItem: &flowv1.CmsItemOrigin{
		ProjectId: "p1", ModelId: "m1", ItemId: "i1", ConnectionId: "c1",
		ProjectName: "Wayfarer", ModelName: "Buildings", ItemTitle: "Shinjuku 3-chome",
	}}}
}

func TestRunTrigger(t *testing.T) {
	t.Parallel()

	t.Run("passes the item's values and origin, and returns the job", func(t *testing.T) {
		t.Parallel()
		f := &fakeTriggers{}
		tid := id.NewTriggerID()

		res, err := NewServer().RunTrigger(withUsecases(&interfaces.Container{Trigger: f}), &flowv1.RunTriggerRequest{
			TriggerId: tid.String(),
			Variables: map[string]*structpb.Value{
				"cityGmlPath": structpb.NewStringValue("https://assets.example.com/bldg.gml"),
				"port":        structpb.NewNumberValue(8077),
			},
			Origin: cmsItemOrigin(),
		})

		require.NoError(t, err)
		require.NotNil(t, f.ran)
		assert.Equal(t, tid, f.ran.TriggerID)
		assert.Equal(t, map[string]any{"cityGmlPath": "https://assets.example.com/bldg.gml", "port": float64(8077)}, f.ran.Variables)
		require.NotNil(t, f.ran.Origin.CMSItem)
		assert.Equal(t, "i1", f.ran.Origin.CMSItem.ItemID)
		assert.Equal(t, "Shinjuku 3-chome", f.ran.Origin.CMSItem.ItemTitle)
		assert.Equal(t, tid.String(), res.GetJob().GetTriggerId())
		assert.Equal(t, "c1", res.GetJob().GetOrigin().GetCmsItem().GetConnectionId())
	})

	t.Run("needs the CMS item", func(t *testing.T) {
		t.Parallel()
		f := &fakeTriggers{}

		_, err := NewServer().RunTrigger(withUsecases(&interfaces.Container{Trigger: f}), &flowv1.RunTriggerRequest{
			TriggerId: id.NewTriggerID().String(),
		})

		assert.Equal(t, codes.InvalidArgument, status.Code(err))
		assert.Nil(t, f.ran)
	})

	t.Run("refuses a malformed trigger ID", func(t *testing.T) {
		t.Parallel()

		_, err := NewServer().RunTrigger(withUsecases(&interfaces.Container{Trigger: &fakeTriggers{}}), &flowv1.RunTriggerRequest{
			TriggerId: "nope", Origin: cmsItemOrigin(),
		})

		assert.Equal(t, codes.InvalidArgument, status.Code(err))
	})
}

func TestRunTrigger_Errors(t *testing.T) {
	t.Parallel()

	tests := []struct {
		err  error
		name string
		want codes.Code
	}{
		{name: "not a CMS trigger", err: interfaces.ErrTriggerNotCMS, want: codes.FailedPrecondition},
		{name: "turned off", err: interfaces.ErrTriggerDisabled, want: codes.FailedPrecondition},
		{name: "plan's runs used up", err: &interfaces.UsageLimitError{Err: interfaces.ErrDeploymentRunsPerDayLimit}, want: codes.ResourceExhausted},
		{name: "bad value", err: fmt.Errorf("%w: port", interfaces.ErrInvalidVariableValue), want: codes.InvalidArgument},
		{name: "required variable empty", err: fmt.Errorf("%w: cityGmlPath", interfaces.ErrRequiredVariableMissing), want: codes.InvalidArgument},
		{name: "unknown variable", err: fmt.Errorf("%w: old", interfaces.ErrUnknownVariable), want: codes.InvalidArgument},
		{name: "not allowed to run", err: interfaces.ErrOperationDenied, want: codes.PermissionDenied},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			f := &fakeTriggers{runErr: tc.err}

			_, err := NewServer().RunTrigger(withUsecases(&interfaces.Container{Trigger: f}), &flowv1.RunTriggerRequest{
				TriggerId: id.NewTriggerID().String(), Origin: cmsItemOrigin(),
			})

			assert.Equal(t, tc.want, status.Code(err))
		})
	}
}

func TestListTriggers_CMS(t *testing.T) {
	t.Parallel()

	t.Run("filters by event source and adds each trigger's deployment, project and variables", func(t *testing.T) {
		t.Parallel()
		ws := accountsid.NewWorkspaceID()
		p := project.New().NewID().Workspace(ws).Name("PLATEAU QC").MustBuild()
		pid := p.ID()
		param, err := parameter.New().ID(id.NewParameterID()).ProjectID(pid).Name("cityGmlPath").
			Type(parameter.TypeText).Required(true).Build()
		require.NoError(t, err)
		d := deployment.New().NewID().Workspace(ws).Project(&pid).Version("v3").Parameters([]*parameter.Parameter{param}).MustBuild()
		tr := trigger.New().NewID().Workspace(ws).Deployment(d.ID()).Description("Building quality check").
			EventSource(trigger.EventSourceTypeCMS).Enabled(true).MustBuild()
		orphan := trigger.New().NewID().Workspace(ws).Deployment(id.NewDeploymentID()).Description("gone").
			EventSource(trigger.EventSourceTypeCMS).MustBuild()
		triggers := &fakeTriggers{triggers: []*trigger.Trigger{tr, orphan}}
		ctx := withUsecases(&interfaces.Container{
			Trigger:    triggers,
			Deployment: &fakeDeployments{deployments: []*deployment.Deployment{d}},
			Project:    &fakeProjectFetch{projects: []*project.Project{p}},
		})
		cms := flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_CMS

		res, err := NewServer().ListTriggers(ctx, &flowv1.ListTriggersRequest{WorkspaceId: ws.String(), EventSource: &cms})

		require.NoError(t, err)
		require.NotNil(t, triggers.filter.EventSource)
		assert.Equal(t, trigger.EventSourceTypeCMS, *triggers.filter.EventSource)
		require.Len(t, res.GetTriggers(), 2)

		got := res.GetTriggers()[0]
		assert.Equal(t, flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_CMS, got.GetEventSource())
		assert.Equal(t, "v3", got.GetDeployment().GetVersion())
		assert.Equal(t, "PLATEAU QC", got.GetProjectName())
		require.Len(t, got.GetVariables(), 1)
		assert.Equal(t, "cityGmlPath", got.GetVariables()[0].GetName())
		assert.True(t, got.GetVariables()[0].GetRequired())

		gone := res.GetTriggers()[1]
		assert.Nil(t, gone.GetDeployment(), "a trigger whose deployment is gone is still listed")
		assert.Empty(t, gone.GetVariables())
	})

	t.Run("refuses an unknown event source", func(t *testing.T) {
		t.Parallel()
		unknown := flowv1.TriggerEventSource(99)

		_, err := NewServer().ListTriggers(withUsecases(&interfaces.Container{Trigger: &fakeTriggers{}}), &flowv1.ListTriggersRequest{
			WorkspaceId: accountsid.NewWorkspaceID().String(), EventSource: &unknown,
		})

		assert.Equal(t, codes.InvalidArgument, status.Code(err))
	})
}

func TestToTriggerVariables(t *testing.T) {
	t.Parallel()

	pid := id.NewProjectID()
	newParam := func(name string, typ parameter.Type, index int, def, config any) *parameter.Parameter {
		p, err := parameter.New().ID(id.NewParameterID()).ProjectID(pid).Name(name).Type(typ).Index(index).
			DefaultValue(def).Config(config).Build()
		require.NoError(t, err)
		return p
	}
	params := []*parameter.Parameter{
		newParam("port", parameter.TypeNumber, 2, float64(8077), nil),
		newParam("cityCode", parameter.TypeText, 0, "13101", nil),
		newParam("lod", parameter.TypeChoice, 1, "1", map[string]any{"choices": []any{"1", "2"}}),
		newParam("area", parameter.TypeGeometry, 3, nil, nil),
	}
	own := []variable.Variable{{Key: "cityCode", Type: parameter.TypeText, Value: "13113"}}

	got := toTriggerVariables(params, own)

	require.Len(t, got, 4)
	assert.Equal(t, []string{"cityCode", "lod", "port", "area"},
		[]string{got[0].GetName(), got[1].GetName(), got[2].GetName(), got[3].GetName()}, "in the deployment's order")
	assert.Equal(t, "13113", got[0].GetDefaultValue().GetStringValue(), "the trigger's own value wins over the default")
	assert.Equal(t, flowv1.VariableType_VARIABLE_TYPE_CHOICE, got[1].GetType())
	assert.Equal(t, []string{"1", "2"}, got[1].GetChoices())
	assert.Equal(t, float64(8077), got[2].GetDefaultValue().GetNumberValue())
	assert.Equal(t, flowv1.VariableType_VARIABLE_TYPE_UNSPECIFIED, got[3].GetType(), "a type a caller cannot set")
	assert.Nil(t, got[3].GetDefaultValue())
}

func TestToJob_CMS(t *testing.T) {
	t.Parallel()

	tid := id.NewTriggerID()
	origin := &job.Origin{CMSItem: &job.CMSItemOrigin{ItemID: "i1", ItemTitle: "Shinjuku 3-chome"}}
	j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Trigger(&tid).Origin(origin).
		OutputURLs([]string{
			"https://flow.example.com/artifacts/j1/artifacts/qc_report.json",
			"https://flow.example.com/artifacts/j1/artifacts/error%20features.geojson",
		}).MustBuild()

	got := toJob(j)

	assert.Equal(t, tid.String(), got.GetTriggerId())
	assert.Equal(t, "Shinjuku 3-chome", got.GetOrigin().GetCmsItem().GetItemTitle())
	require.Len(t, got.GetOutputs(), 2)
	assert.Equal(t, "qc_report.json", got.GetOutputs()[0].GetName())
	assert.Equal(t, "error features.geojson", got.GetOutputs()[1].GetName())
	assert.Equal(t, "https://flow.example.com/artifacts/j1/artifacts/qc_report.json", got.GetOutputs()[0].GetUrl())

	plain := toJob(job.New().NewID().Workspace(accountsid.NewWorkspaceID()).MustBuild())
	assert.Nil(t, plain.TriggerId)
	assert.Nil(t, plain.GetOrigin())
	assert.Empty(t, plain.GetOutputs())
}

func TestBatchGetJobs(t *testing.T) {
	t.Parallel()

	t.Run("returns the jobs found in the order asked, refreshing only those not finished", func(t *testing.T) {
		t.Parallel()
		ws := accountsid.NewWorkspaceID()
		running := job.New().NewID().Workspace(ws).Status(job.StatusRunning).MustBuild()
		done := job.New().NewID().Workspace(ws).Status(job.StatusCompleted).MustBuild()
		jobs := &fakeJobs{jobs: []*job.Job{running, done}}

		res, err := NewServer().BatchGetJobs(withUsecases(&interfaces.Container{Job: jobs}), &flowv1.BatchGetJobsRequest{
			JobIds: []string{done.ID().String(), id.NewJobID().String(), running.ID().String()},
		})

		require.NoError(t, err)
		require.Len(t, res.GetJobs(), 2, "a job that is not found is left out")
		assert.Equal(t, done.ID().String(), res.GetJobs()[0].GetId())
		assert.Equal(t, running.ID().String(), res.GetJobs()[1].GetId())
		assert.Equal(t, []id.JobID{running.ID()}, jobs.refreshed)
	})

	t.Run("returns nothing for jobs the caller may not see", func(t *testing.T) {
		t.Parallel()
		jobs := &fakeJobs{fetchErr: interfaces.ErrOperationDenied}

		res, err := NewServer().BatchGetJobs(withUsecases(&interfaces.Container{Job: jobs}), &flowv1.BatchGetJobsRequest{
			JobIds: []string{id.NewJobID().String()},
		})

		require.NoError(t, err)
		assert.Empty(t, res.GetJobs())
	})

	t.Run("refuses more than 100 IDs and malformed ones", func(t *testing.T) {
		t.Parallel()
		ctx := withUsecases(&interfaces.Container{Job: &fakeJobs{}})
		many := make([]string, maxBatchGetJobs+1)
		for i := range many {
			many[i] = id.NewJobID().String()
		}

		_, err := NewServer().BatchGetJobs(ctx, &flowv1.BatchGetJobsRequest{JobIds: many})
		assert.Equal(t, codes.InvalidArgument, status.Code(err))

		_, err = NewServer().BatchGetJobs(ctx, &flowv1.BatchGetJobsRequest{JobIds: []string{"nope"}})
		assert.Equal(t, codes.InvalidArgument, status.Code(err))
	})
}

func TestCancelJob(t *testing.T) {
	t.Parallel()

	t.Run("a job that has finished cannot be cancelled", func(t *testing.T) {
		t.Parallel()
		jobs := &fakeJobs{cancelErr: fmt.Errorf("%w: current status is COMPLETED", interfaces.ErrJobNotCancellable)}

		_, err := NewServer().CancelJob(withUsecases(&interfaces.Container{Job: jobs}), &flowv1.CancelJobRequest{JobId: id.NewJobID().String()})

		assert.Equal(t, codes.FailedPrecondition, status.Code(err))
	})
}
