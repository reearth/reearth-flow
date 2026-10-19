package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// planChecker answers general_private_project_creation: paid workspaces may
// have private projects, every other workspace is on the free plan.
type planChecker struct {
	paid  map[accountsid.WorkspaceID]bool
	err   error
	calls int
}

func (c *planChecker) CheckPolicy(_ context.Context, req gateway.PolicyCheckRequest) (*gateway.PolicyCheckResponse, error) {
	c.calls++
	if c.err != nil {
		return nil, c.err
	}
	return &gateway.PolicyCheckResponse{CheckType: req.CheckType, Allowed: c.paid[req.WorkspaceID], Value: req.Value}, nil
}

func TestFreeTier_EnsureCanRun(t *testing.T) {
	ctx := context.Background()
	now := time.Date(2026, 10, 19, 12, 0, 0, 0, time.UTC)
	free, paid := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()

	running := func(ws accountsid.WorkspaceID, startedAt time.Time) *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(startedAt).MustBuild()
	}

	tests := []struct {
		checker   *planChecker
		wantErr   error
		name      string
		jobs      []*job.Job
		ws        accountsid.WorkspaceID
		limits    gateway.FreeTierLimits
		wantCalls int
	}{
		{
			name:      "no limit configured never asks the plan",
			limits:    gateway.FreeTierLimits{},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now)},
			ws:        free,
			wantCalls: 0,
		},
		{
			name:      "free workspace with nothing running may run",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{},
			ws:        free,
			wantCalls: 1,
		},
		{
			name:      "free workspace with a job running is refused",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now.Add(-time.Hour))},
			ws:        free,
			wantErr:   interfaces.ErrJobAlreadyRunning,
			wantCalls: 1,
		},
		{
			name:      "a job older than the default lifetime no longer blocks",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now.Add(-gateway.DefaultJobLifetime-time.Minute))},
			ws:        free,
			wantCalls: 1,
		},
		{
			name:      "a longer worker lifetime keeps a long job counting",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1, JobLifetime: gateway.JobLifetimeFor(12*time.Hour, 1)},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now.Add(-8*time.Hour))},
			ws:        free,
			wantErr:   interfaces.ErrJobAlreadyRunning,
			wantCalls: 1,
		},
		{
			name:      "a job older than the configured lifetime no longer blocks",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1, JobLifetime: gateway.JobLifetimeFor(12*time.Hour, 1)},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now.Add(-14*time.Hour))},
			ws:        free,
			wantCalls: 1,
		},
		{
			name:      "a higher limit allows more jobs at once",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 2},
			checker:   &planChecker{},
			jobs:      []*job.Job{running(free, now)},
			ws:        free,
			wantCalls: 1,
		},
		{
			name:      "paid workspace is not limited",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{paid: map[accountsid.WorkspaceID]bool{paid: true}},
			jobs:      []*job.Job{running(paid, now)},
			ws:        paid,
			wantCalls: 1,
		},
		{
			name:      "a policy service failure allows the run",
			limits:    gateway.FreeTierLimits{ConcurrentJobs: 1},
			checker:   &planChecker{err: errors.New("policy: check returned status 503")},
			jobs:      []*job.Job{running(free, now)},
			ws:        free,
			wantCalls: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			jobRepo := memory.NewJob()
			for _, j := range tt.jobs {
				require.NoError(t, jobRepo.Save(ctx, j))
			}
			f := freeTier{
				limits:        tt.limits,
				policyChecker: tt.checker,
				jobRepo:       jobRepo,
				now:           func() time.Time { return now },
			}

			_, err := f.ensureCanRun(ctx, tt.ws)

			if tt.wantErr != nil {
				assert.ErrorIs(t, err, tt.wantErr)
			} else {
				assert.NoError(t, err)
			}
			assert.Equal(t, tt.wantCalls, tt.checker.calls)
		})
	}
}

func TestFreeTier_EnsureCanRun_NoPolicyChecker(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	jobRepo := memory.NewJob()
	require.NoError(t, jobRepo.Save(ctx, job.New().NewID().Workspace(ws).Status(job.StatusRunning).StartedAt(time.Now()).MustBuild()))

	f := freeTier{limits: gateway.FreeTierLimits{ConcurrentJobs: 1}, jobRepo: jobRepo, now: time.Now}

	limit, err := f.ensureCanRun(ctx, ws)
	assert.NoError(t, err, "without a policy checker no workspace is on the free plan")
	assert.Zero(t, limit)
}

// Two runs that both pass the early check must not both be saved: the slot is
// taken when the job is saved.
func TestFreeTier_SaveJobReservesTheSlot(t *testing.T) {
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	f := freeTier{limits: gateway.FreeTierLimits{ConcurrentJobs: 1}, policyChecker: &planChecker{}, jobRepo: memory.NewJob(), now: time.Now}

	limitA, err := f.ensureCanRun(ctx, ws)
	require.NoError(t, err)
	limitB, err := f.ensureCanRun(ctx, ws)
	require.NoError(t, err)
	require.Equal(t, 1, limitB, "both runs see an empty workspace")

	newJob := func() *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(time.Now()).MustBuild()
	}
	assert.NoError(t, f.saveJob(ctx, newJob(), limitA))
	assert.ErrorIs(t, f.saveJob(ctx, newJob(), limitB), interfaces.ErrJobAlreadyRunning)
}

func TestFreeTier_HourlyAllowed(t *testing.T) {
	ctx := context.Background()
	free, paid := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()

	tests := []struct {
		checker   *planChecker
		wantErr   error
		name      string
		ws        accountsid.WorkspaceID
		limits    gateway.FreeTierLimits
		wantCalls int
	}{
		{name: "hourly allowed by config never asks the plan", checker: &planChecker{}, ws: free},
		{name: "free workspace is refused", limits: gateway.FreeTierLimits{DisallowHourly: true}, checker: &planChecker{}, ws: free, wantErr: interfaces.ErrHourlyScheduleNotAllowed, wantCalls: 1},
		{name: "paid workspace is allowed", limits: gateway.FreeTierLimits{DisallowHourly: true}, checker: &planChecker{paid: map[accountsid.WorkspaceID]bool{paid: true}}, ws: paid, wantCalls: 1},
		{name: "a policy service failure allows it", limits: gateway.FreeTierLimits{DisallowHourly: true}, checker: &planChecker{err: errors.New("timeout")}, ws: free, wantCalls: 1},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := freeTier{limits: tt.limits, policyChecker: tt.checker, jobRepo: memory.NewJob(), now: time.Now}

			err := f.hourlyAllowed(ctx, tt.ws)

			if tt.wantErr != nil {
				assert.ErrorIs(t, err, tt.wantErr)
			} else {
				assert.NoError(t, err)
			}
			assert.Equal(t, tt.wantCalls, tt.checker.calls)
		})
	}
}
