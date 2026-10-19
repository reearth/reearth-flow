package interactor

import (
	"context"
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
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/appx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// freeTierFixture is a free workspace that already has a job running, wired
// into interactors built the way NewContainer builds them. Every entry point
// that starts a job must refuse before doing any work, so the gateways a real
// run needs (websocket, file, batch) are left nil: reaching them would panic.
type freeTierFixture struct {
	ctx  context.Context
	job  interfaces.Job
	perm gateway.PermissionChecker
	repo *repo.Container
	gw   *gateway.Container
	ws   accountsid.WorkspaceID
}

func newFreeTierFixture(t *testing.T, limits gateway.FreeTierLimits) *freeTierFixture {
	t.Helper()
	ctx := adapter.AttachAuthInfo(context.Background(), &appx.AuthInfo{Token: "token"})
	ctx = adapter.AttachUser(ctx, accountsuser.New().NewID().Name("free").Email("free@example.com").MustBuild())

	ws := accountsid.NewWorkspaceID()
	r := memory.New()
	r.Job = memory.NewJob()
	require.NoError(t, r.Job.Save(ctx, job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(time.Now()).MustBuild()))

	gw := &gateway.Container{PolicyChecker: &planChecker{}, FreeTier: limits}
	perm := NewMockPermissionChecker(func(context.Context, string, string) (bool, error) { return true, nil })
	return &freeTierFixture{ctx: ctx, ws: ws, repo: r, gw: gw, job: NewJob(r, gw, perm), perm: perm}
}

func (f *freeTierFixture) deployment(t *testing.T) *deployment.Deployment {
	t.Helper()
	d := deployment.New().NewID().Workspace(f.ws).WorkflowURL("workflow.yml").Version("v1").MustBuild()
	require.NoError(t, f.repo.Deployment.Save(f.ctx, d))
	return d
}

func (f *freeTierFixture) trigger(t *testing.T, b *trigger.Builder) *trigger.Trigger {
	t.Helper()
	tr := b.NewID().Workspace(f.ws).Deployment(f.deployment(t).ID()).Description("t").Enabled(true).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
	return tr
}

func TestFreeTier_RunEntryPointsRefuseASecondJob(t *testing.T) {
	limits := gateway.FreeTierLimits{ConcurrentJobs: 1}

	t.Run("debug run", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		p := project.New().NewID().Workspace(f.ws).Name("p").MustBuild()
		require.NoError(t, f.repo.Project.Save(f.ctx, p))
		i := NewProject(f.repo, f.gw, f.job, f.perm, nil, nil)

		_, err := i.Run(f.ctx, interfaces.RunProjectParam{ProjectID: p.ID(), Workflow: &file.File{Path: "workflow.yml"}})

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
	})

	t.Run("deployment run", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		d := f.deployment(t)
		i := NewDeployment(f.repo, f.gw, f.job, f.perm)

		_, err := i.Execute(f.ctx, interfaces.ExecuteDeploymentParam{DeploymentID: d.ID()})

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
	})

	t.Run("API trigger", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		tr := f.trigger(t, trigger.New().EventSource("API_DRIVEN").AuthToken("secret"))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tr.ID(), AuthenticationToken: "secret"})

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
	})

	t.Run("scheduled trigger", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
	})
}

func TestFreeTier_HourlyTriggers(t *testing.T) {
	limits := gateway.FreeTierLimits{DisallowHourly: true}

	t.Run("creating an hourly trigger is refused", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Enabled: true, Description: "t",
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryHour,
		})

		assert.ErrorIs(t, err, interfaces.ErrHourlyScheduleNotAllowed)
	})

	t.Run("creating a daily trigger is allowed", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		got, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Enabled: true, Description: "t",
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryDay,
		})

		require.NoError(t, err)
		assert.Equal(t, trigger.TimeIntervalEveryDay, *got.TimeInterval())
	})

	t.Run("switching a trigger to hourly is refused", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{
			ID: tr.ID(), EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryHour,
		})

		assert.ErrorIs(t, err, interfaces.ErrHourlyScheduleNotAllowed)
		stored, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
		require.NoError(t, err)
		assert.Equal(t, trigger.TimeIntervalEveryDay, *stored.TimeInterval(), "the refused update must not be saved")
	})

	t.Run("an existing hourly trigger can still be edited", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryHour))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)
		desc := "renamed"

		got, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{
			ID: tr.ID(), Description: &desc, EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryHour,
		})

		require.NoError(t, err)
		assert.Equal(t, "renamed", got.Description())
	})

	t.Run("a paid workspace may create an hourly trigger", func(t *testing.T) {
		f := newFreeTierFixture(t, limits)
		f.gw.PolicyChecker = &planChecker{paid: map[accountsid.WorkspaceID]bool{f.ws: true}}
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Enabled: true, Description: "t",
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryHour,
		})

		assert.NoError(t, err)
	})
}
