package interactor

import (
	"context"
	"errors"
	"io"
	"net/url"
	"strings"
	"testing"
	"time"

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
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/appx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// planFixture is a workspace on the default Flow plan that already has a debug
// run running, wired into interactors built the way NewContainer builds them.
// Its file gateway fails the first file step of a run, so a run that gets past
// the plan's checks fails with errUploadFailed, and one refused by them fails
// before reaching any other gateway (websocket, batch): reaching them would
// panic.
type planFixture struct {
	ctx  context.Context
	job  interfaces.Job
	perm gateway.PermissionChecker
	repo *repo.Container
	gw   *gateway.Container
	ws   accountsid.WorkspaceID
}

func newPlanFixture(t *testing.T) *planFixture {
	t.Helper()
	ctx := adapter.AttachAuthInfo(context.Background(), &appx.AuthInfo{Token: "token"})
	ctx = adapter.AttachUser(ctx, accountsuser.New().NewID().Name("user").Email("user@example.com").MustBuild())

	ws := accountsid.NewWorkspaceID()
	r := memory.New()
	r.Job = memory.NewJob()
	require.NoError(t, r.Job.Save(ctx, debugRun(ws, job.StatusRunning, time.Now())))

	gw := &gateway.Container{PolicyChecker: &planChecker{}, File: failingFile{}}
	perm := NewMockPermissionChecker(func(context.Context, string, string) (bool, error) { return true, nil })
	return &planFixture{ctx: ctx, ws: ws, repo: r, gw: gw, job: NewJob(r, gw, perm), perm: perm}
}

func (f *planFixture) deployment(t *testing.T) *deployment.Deployment {
	t.Helper()
	d := deployment.New().NewID().Workspace(f.ws).WorkflowURL("workflow.yml").Version("v1").MustBuild()
	require.NoError(t, f.repo.Deployment.Save(f.ctx, d))
	return d
}

func (f *planFixture) trigger(t *testing.T, b *trigger.Builder) *trigger.Trigger {
	t.Helper()
	tr := b.NewID().Workspace(f.ws).Deployment(f.deployment(t).ID()).Description("t").Enabled(true).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
	return tr
}

func TestPlanLimits_OnlyDebugRunsWaitForADebugRun(t *testing.T) {
	t.Parallel()

	t.Run("a second debug run is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		p := project.New().NewID().Workspace(f.ws).Name("p").MustBuild()
		require.NoError(t, f.repo.Project.Save(f.ctx, p))
		i := NewProject(f.repo, f.gw, f.job, f.perm, nil, nil)

		_, err := i.Run(f.ctx, interfaces.RunProjectParam{ProjectID: p.ID(), Workflow: &file.File{Path: "workflow.yml"}})

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
	})

	t.Run("a deployment run is not", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		d := f.deployment(t)
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)

		_, err := i.Execute(f.ctx, interfaces.ExecuteDeploymentParam{DeploymentID: d.ID()})

		assert.ErrorIs(t, err, errUploadFailed)
	})

	t.Run("an API trigger is not", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("API_DRIVEN").AuthTokenHash(trigger.HashAuthToken("secret")))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID(), AuthenticationToken: "secret"})

		assert.ErrorIs(t, err, errUploadFailed)
	})

	t.Run("a scheduled trigger is not", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, errUploadFailed)
	})
}

// usedUpCounter reports every count as used up.
type usedUpCounter struct{}

func (usedUpCounter) Increment(context.Context, string, time.Time) (int64, error) { return 1000, nil }

func (usedUpCounter) Decrement(context.Context, string, time.Time) error { return nil }

func (usedUpCounter) AcquireLease(context.Context, string, string, time.Time, time.Time) (int64, error) {
	return 1000, nil
}

func (usedUpCounter) ReleaseLease(context.Context, string, string) error { return nil }

var errUploadFailed = errors.New("gcs: upload failed")

// failingFile fails the first file step of every run: reading a deployment's
// workflow file or uploading a run's metadata.
type failingFile struct{ gateway.File }

func (failingFile) UploadMetadata(context.Context, string, []string) (*url.URL, error) {
	return nil, errUploadFailed
}

func (failingFile) ReadWorkflow(context.Context, string) (io.ReadCloser, error) {
	return nil, errUploadFailed
}

func TestPlanLimits_RunsPerHourAndDay(t *testing.T) {
	t.Parallel()

	t.Run("a debug run over the hour's limit is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		f.gw.PolicyChecker = &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{
			f.ws: defaultFlowPlan.with(func(p *flowPlan) { p.debugConcurrentRuns = -1 }),
		}}
		f.gw.UsageCounter = usedUpCounter{}
		p := project.New().NewID().Workspace(f.ws).Name("p").MustBuild()
		require.NoError(t, f.repo.Project.Save(f.ctx, p))
		i := NewProject(f.repo, f.gw, f.job, f.perm, nil, nil)

		_, err := i.Run(f.ctx, interfaces.RunProjectParam{ProjectID: p.ID(), Workflow: &file.File{Path: "workflow.yml"}})

		assert.ErrorIs(t, err, interfaces.ErrDebugRunsPerHourLimit)
	})

	t.Run("a deployment run over the day's limit is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		f.gw.UsageCounter = usedUpCounter{}
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)

		_, err := i.Execute(f.ctx, interfaces.ExecuteDeploymentParam{DeploymentID: f.deployment(t).ID()})

		assert.ErrorIs(t, err, interfaces.ErrDeploymentRunsPerDayLimit)
	})

	t.Run("running a trigger by hand counts as a deployment run", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		usage := &usageCounter{}
		f.gw.UsageCounter = usage
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, errUploadFailed)
		require.Len(t, usage.keys, 1)
		assert.True(t, strings.HasPrefix(usage.keys[0], deploymentRunsPerDay.keyPrefix), usage.keys[0])
	})

	t.Run("a scheduled run over the day's limit is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		f.gw.UsageCounter = usedUpCounter{}
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrTimeDrivenTriggerRunsPerDayLimit)
	})

	t.Run("a run the server fails is given back", func(t *testing.T) {
		t.Parallel()

		for name, run := range map[string]func(*testing.T, *planFixture) error{
			"deployment": func(t *testing.T, f *planFixture) error {
				_, err := NewDeployment(f.repo, f.gw, f.job, f.perm).Execute(f.ctx, interfaces.ExecuteDeploymentParam{DeploymentID: f.deployment(t).ID()})
				return err
			},
			"scheduled trigger": func(t *testing.T, f *planFixture) error {
				tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
				_, err := NewTrigger(f.repo, f.gw, f.job, f.perm).ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})
				return err
			},
		} {
			t.Run(name, func(t *testing.T) {
				t.Parallel()

				f := newPlanFixture(t)
				usage := &usageCounter{}
				f.gw.UsageCounter = usage

				assert.ErrorIs(t, run(t, f), errUploadFailed)
				require.Len(t, usage.keys, 1)
				assert.Zero(t, usage.counts[usage.keys[0]])
			})
		}
	})
}

// A trigger's job is saved under its deployment's workspace, so that workspace's
// limit applies, and a trigger may not point into another workspace.
func TestPlanLimits_TriggersUseTheDeploymentWorkspace(t *testing.T) {
	t.Parallel()

	t.Run("a legacy cross-workspace trigger is counted against the deployment's workspace", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		usage := &usageCounter{}
		f.gw.UsageCounter = usage
		tr := trigger.New().NewID().Workspace(accountsid.NewWorkspaceID()).Deployment(f.deployment(t).ID()).
			Description("t").Enabled(true).EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay).
			CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, errUploadFailed)
		require.Len(t, usage.keys, 1)
		assert.Contains(t, usage.keys[0], f.ws.String())
	})

	t.Run("creating a trigger for another workspace's deployment is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, _, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: accountsid.NewWorkspaceID(), DeploymentID: f.deployment(t).ID(), Enabled: true, Description: "t",
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryDay,
		})

		assert.ErrorIs(t, err, interfaces.ErrDeploymentNotInWorkspace)
	})

	t.Run("moving a trigger to another workspace's deployment is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		other := deployment.New().NewID().Workspace(accountsid.NewWorkspaceID()).WorkflowURL("workflow.yml").Version("v1").MustBuild()
		require.NoError(t, f.repo.Deployment.Save(f.ctx, other))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)
		did := other.ID()

		_, _, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{ID: tr.ID(), DeploymentID: &did})

		assert.ErrorIs(t, err, interfaces.ErrDeploymentNotInWorkspace)
		stored, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
		require.NoError(t, err)
		assert.Equal(t, tr.Deployment(), stored.Deployment(), "the refused update must not be saved")
	})
}

func TestPlanLimits_HourlyTriggers(t *testing.T) {
	t.Parallel()

	t.Run("creating an hourly trigger is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, _, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Enabled: true, Description: "t",
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryHour,
		})

		assert.ErrorIs(t, err, interfaces.ErrHourlyScheduleNotAllowed)
	})

	t.Run("creating a daily trigger is allowed", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		got, _, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Enabled: true, Description: "t",
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryDay,
		})

		require.NoError(t, err)
		assert.Equal(t, trigger.TimeIntervalEveryDay, *got.TimeInterval())
	})

	t.Run("switching a trigger to hourly is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, _, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{
			ID: tr.ID(), EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryHour,
		})

		assert.ErrorIs(t, err, interfaces.ErrHourlyScheduleNotAllowed)
		stored, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
		require.NoError(t, err)
		assert.Equal(t, trigger.TimeIntervalEveryDay, *stored.TimeInterval(), "the refused update must not be saved")
	})

	t.Run("an existing hourly trigger can still be edited", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryHour))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)
		desc := "renamed"

		got, _, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{
			ID: tr.ID(), Description: &desc, EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryHour,
		})

		require.NoError(t, err)
		assert.Equal(t, "renamed", got.Description())
	})

	t.Run("a plan with hourly schedules may create an hourly trigger", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		f.gw.PolicyChecker = &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{
			f.ws: defaultFlowPlan.with(func(p *flowPlan) { p.hourlySchedule = true }),
		}}
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, _, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Enabled: true, Description: "t",
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryHour,
		})

		assert.NoError(t, err)
	})
}

func TestPlanLimits_APITriggerDailyLimit(t *testing.T) {
	t.Parallel()

	t.Run("an API trigger call over the day's limit is refused", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		f.gw.UsageCounter = usedUpCounter{}
		tr := f.trigger(t, trigger.New().EventSource("API_DRIVEN").AuthTokenHash(trigger.HashAuthToken("secret")))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID(), AuthenticationToken: "secret"})

		assert.ErrorIs(t, err, interfaces.ErrAPITriggerDailyLimit)
	})

	t.Run("a call with a wrong token is not counted", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		f.gw.UsageCounter = usedUpCounter{}
		tr := f.trigger(t, trigger.New().EventSource("API_DRIVEN").AuthTokenHash(trigger.HashAuthToken("secret")))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID(), AuthenticationToken: "wrong"})

		assert.ErrorContains(t, err, "invalid auth token")
		assert.NotErrorIs(t, err, interfaces.ErrAPITriggerDailyLimit)
	})

	t.Run("a call the server fails is given back", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		usage := &usageCounter{}
		f.gw.UsageCounter = usage
		f.gw.File = failingFile{}
		tr := f.trigger(t, trigger.New().EventSource("API_DRIVEN").AuthTokenHash(trigger.HashAuthToken("secret")))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID(), AuthenticationToken: "secret"})

		assert.ErrorIs(t, err, errUploadFailed)
		require.Len(t, usage.keys, 1)
		assert.Zero(t, usage.counts[usage.keys[0]])
	})

	t.Run("a scheduled run does not use API trigger calls", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		usage := &usageCounter{}
		f.gw.UsageCounter = usage
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, errUploadFailed)
		require.Len(t, usage.keys, 1)
		assert.True(t, strings.HasPrefix(usage.keys[0], timeDrivenTriggerRunsPerDay.keyPrefix), usage.keys[0])
	})
}

func TestPlanLimits_EnabledTriggers(t *testing.T) {
	t.Parallel()

	create := func(f *planFixture, enabled bool) error {
		_, _, err := NewTrigger(f.repo, f.gw, f.job, f.perm).Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Enabled: enabled, Description: "t",
			EventSource: "API_DRIVEN",
		})
		return err
	}
	update := func(f *planFixture, tr *trigger.Trigger, p interfaces.UpdateTriggerParam) error {
		p.ID = tr.ID()
		_, _, err := NewTrigger(f.repo, f.gw, f.job, f.perm).Update(f.ctx, p)
		return err
	}
	enabled := func(f *planFixture, tr *trigger.Trigger) bool {
		t.Helper()
		stored, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
		require.NoError(t, err)
		return stored.Enabled()
	}
	on, off := true, false

	t.Run("creating an enabled trigger beside an enabled one is refused", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))

		assert.ErrorIs(t, create(f, true), interfaces.ErrEnabledTriggerLimit)
		n, err := f.repo.Trigger.CountEnabledByWorkspace(f.ctx, f.ws)
		require.NoError(t, err)
		assert.Equal(t, 1, n)
	})

	t.Run("the first enabled trigger is allowed", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)

		assert.NoError(t, create(f, true))
	})

	t.Run("creating a disabled trigger is always allowed", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))

		assert.NoError(t, create(f, false))
	})

	t.Run("enabling a second trigger is refused and leaves it disabled", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		f.trigger(t, trigger.New().EventSource("API_DRIVEN").AuthTokenHash(trigger.HashAuthToken("secret")))
		second := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		second.SetEnabled(false)
		require.NoError(t, f.repo.Trigger.Save(f.ctx, second))

		assert.ErrorIs(t, update(f, second, interfaces.UpdateTriggerParam{Enabled: &on}), interfaces.ErrEnabledTriggerLimit)
		assert.False(t, enabled(f, second))
	})

	t.Run("an enabled trigger can be edited and disabled", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		// Over the limit already, as a workspace from before it existed may be.
		first := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		desc := "renamed"

		assert.NoError(t, update(f, first, interfaces.UpdateTriggerParam{Description: &desc, Enabled: &on}))
		assert.NoError(t, update(f, first, interfaces.UpdateTriggerParam{Enabled: &off}))
		assert.False(t, enabled(f, first))
	})

	t.Run("a plan with more enabled triggers allows enabling another", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		f.gw.PolicyChecker = &planChecker{plans: map[accountsid.WorkspaceID]flowPlan{
			f.ws: defaultFlowPlan.with(func(p *flowPlan) { p.enabledTriggers = 2 }),
		}}
		f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))

		assert.NoError(t, create(f, true))
	})
}

// recordingScheduler records the trigger jobs it is asked to pause.
type recordingScheduler struct {
	gateway.Scheduler
	paused []id.TriggerID
}

func (s *recordingScheduler) PauseScheduledJob(_ context.Context, tid id.TriggerID) error {
	s.paused = append(s.paused, tid)
	return nil
}

func TestTrigger_DisabledTriggersDoNotRun(t *testing.T) {
	t.Parallel()

	disabled := func(f *planFixture, b *trigger.Builder) *trigger.Trigger {
		t.Helper()
		tr := f.trigger(t, b)
		tr.SetEnabled(false)
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		return tr
	}

	t.Run("an API call with the right token is refused as disabled", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		tr := disabled(f, trigger.New().EventSource("API_DRIVEN").AuthTokenHash(trigger.HashAuthToken("secret")))

		_, err := NewTrigger(f.repo, f.gw, f.job, f.perm).ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID(), AuthenticationToken: "secret"})

		assert.ErrorIs(t, err, interfaces.ErrTriggerDisabled)
	})

	t.Run("an API call with a wrong token is refused before it learns the trigger is disabled", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		tr := disabled(f, trigger.New().EventSource("API_DRIVEN").AuthTokenHash(trigger.HashAuthToken("secret")))

		_, err := NewTrigger(f.repo, f.gw, f.job, f.perm).ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID(), AuthenticationToken: "wrong"})

		assert.ErrorContains(t, err, "invalid auth token")
		assert.NotErrorIs(t, err, interfaces.ErrTriggerDisabled)
	})

	t.Run("a scheduled call is refused and pauses the schedule", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		scheduler := &recordingScheduler{}
		f.gw.Scheduler = scheduler
		tr := disabled(f, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))

		_, err := NewTrigger(f.repo, f.gw, f.job, f.perm).ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrTriggerDisabled)
		assert.Equal(t, []id.TriggerID{tr.ID()}, scheduler.paused)
	})
}
