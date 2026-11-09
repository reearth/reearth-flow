package gcpscheduler

import (
	"context"
	"testing"
	"time"

	schedulerpb "cloud.google.com/go/scheduler/apiv1/schedulerpb"
	"github.com/googleapis/gax-go/v2"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

// fakeClient records the jobs it is asked to create and update, and keeps each
// job's state as Cloud Scheduler does: a new job is enabled, an update keeps
// the state, only pause and resume change it, and a job that does not exist
// cannot be updated.
type fakeClient struct {
	// pauseErr and resumeErr, when set, fail every pause or resume.
	pauseErr  error
	resumeErr error
	states    map[string]schedulerpb.Job_State
	created   []*schedulerpb.Job
	updated   []*schedulerpb.Job
	paused    []string
	resumed   []string
}

func newFakeClient() *fakeClient {
	return &fakeClient{states: map[string]schedulerpb.Job_State{}}
}

func (f *fakeClient) withState(job *schedulerpb.Job) *schedulerpb.Job {
	out := proto.Clone(job).(*schedulerpb.Job)
	out.State = f.states[job.Name]
	return out
}

func (f *fakeClient) CreateJob(_ context.Context, req *schedulerpb.CreateJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	f.created = append(f.created, req.Job)
	f.states[req.Job.Name] = schedulerpb.Job_ENABLED
	return f.withState(req.Job), nil
}

func (f *fakeClient) GetJob(_ context.Context, _ *schedulerpb.GetJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	return nil, nil
}

func (f *fakeClient) UpdateJob(_ context.Context, req *schedulerpb.UpdateJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	if _, ok := f.states[req.Job.Name]; !ok {
		return nil, status.Error(codes.NotFound, "job not found")
	}
	f.updated = append(f.updated, req.Job)
	return f.withState(req.Job), nil
}

func (f *fakeClient) PauseJob(_ context.Context, req *schedulerpb.PauseJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	if f.pauseErr != nil {
		return nil, f.pauseErr
	}
	if f.states[req.Name] != schedulerpb.Job_ENABLED {
		return nil, status.Error(codes.FailedPrecondition, "a job must be enabled to be paused")
	}
	f.paused = append(f.paused, req.Name)
	f.states[req.Name] = schedulerpb.Job_PAUSED
	return &schedulerpb.Job{Name: req.Name, State: schedulerpb.Job_PAUSED}, nil
}

func (f *fakeClient) ResumeJob(_ context.Context, req *schedulerpb.ResumeJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	if f.resumeErr != nil {
		return nil, f.resumeErr
	}
	if f.states[req.Name] != schedulerpb.Job_PAUSED {
		return nil, status.Error(codes.FailedPrecondition, "a job must be paused to be resumed")
	}
	f.resumed = append(f.resumed, req.Name)
	f.states[req.Name] = schedulerpb.Job_ENABLED
	return &schedulerpb.Job{Name: req.Name, State: schedulerpb.Job_ENABLED}, nil
}

func (f *fakeClient) DeleteJob(_ context.Context, _ *schedulerpb.DeleteJobRequest, _ ...gax.CallOption) error {
	return nil
}

func (f *fakeClient) Close() error { return nil }

func newTestRepo(client *fakeClient) *SchedulerRepo {
	return &SchedulerRepo{client: client, config: SchedulerConfig{ProjectID: "p", Location: "l", Host: "https://api.example.com"}}
}

func timeDrivenTrigger(t *testing.T, interval trigger.TimeInterval, enabled bool) *trigger.Trigger {
	t.Helper()
	trg, err := trigger.New().
		NewID().
		Workspace(accountsid.NewWorkspaceID()).
		Deployment(id.NewDeploymentID()).
		Description("nightly").
		EventSource("TIME_DRIVEN").
		TimeInterval(interval).
		Enabled(enabled).
		Build()
	require.NoError(t, err)
	return trg
}

type wantRetry struct {
	minBackoff  time.Duration
	maxBackoff  time.Duration
	retryCount  int32
	maxDoubling int32
}

func assertRetries(t *testing.T, job *schedulerpb.Job, want wantRetry) {
	t.Helper()
	rc := job.GetRetryConfig()
	require.NotNil(t, rc, "the job has no retry config")
	assert.Equal(t, want.retryCount, rc.GetRetryCount())
	assert.Equal(t, want.minBackoff, rc.GetMinBackoffDuration().AsDuration())
	assert.Equal(t, want.maxBackoff, rc.GetMaxBackoffDuration().AsDuration())
	assert.Equal(t, want.maxDoubling, rc.GetMaxDoublings())
	// Left unset: set together with the retry count, Cloud Scheduler retries
	// until both are reached, so it could only make retries last longer.
	assert.Nil(t, rc.GetMaxRetryDuration())
	assert.Equal(t, 3*time.Minute, job.GetAttemptDeadline().AsDuration())
}

func TestSchedulerRepo_RetryConfig(t *testing.T) {
	t.Parallel()

	hourly := wantRetry{retryCount: 5, minBackoff: time.Minute, maxBackoff: 16 * time.Minute, maxDoubling: 4}
	longer := wantRetry{retryCount: 5, minBackoff: 5 * time.Minute, maxBackoff: time.Hour, maxDoubling: 4}

	tests := []struct {
		name     string
		interval trigger.TimeInterval
		want     wantRetry
	}{
		{name: "every hour", interval: trigger.TimeIntervalEveryHour, want: hourly},
		{name: "every day", interval: trigger.TimeIntervalEveryDay, want: longer},
		{name: "every week", interval: trigger.TimeIntervalEveryWeek, want: longer},
		{name: "every month", interval: trigger.TimeIntervalEveryMonth, want: longer},
	}

	for _, tc := range tests {
		t.Run(tc.name+" on create", func(t *testing.T) {
			t.Parallel()

			client := newFakeClient()
			s := newTestRepo(client)

			require.NoError(t, s.CreateScheduledJob(context.Background(), timeDrivenTrigger(t, tc.interval, true)))
			require.Len(t, client.created, 1)
			assertRetries(t, client.created[0], tc.want)
		})

		t.Run(tc.name+" on update", func(t *testing.T) {
			t.Parallel()

			client := newFakeClient()
			s := newTestRepo(client)
			trg := timeDrivenTrigger(t, tc.interval, true)
			client.states[s.jobName(trg.ID())] = schedulerpb.Job_ENABLED

			require.NoError(t, s.UpdateScheduledJob(context.Background(), trg))
			require.Len(t, client.updated, 1)
			assertRetries(t, client.updated[0], tc.want)
		})
	}
}

// A disabled trigger's job is paused, so Cloud Scheduler does not call the
// server, and fail and retry, at every scheduled time.
func TestSchedulerRepo_PausesDisabledTriggers(t *testing.T) {
	t.Parallel()

	ctx := context.Background()

	t.Run("a trigger created disabled", func(t *testing.T) {
		t.Parallel()

		client := newFakeClient()
		s := newTestRepo(client)
		trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, false)

		require.NoError(t, s.CreateScheduledJob(ctx, trg))
		assert.Equal(t, []string{s.jobName(trg.ID())}, client.paused)
		assert.Equal(t, schedulerpb.Job_PAUSED, client.states[s.jobName(trg.ID())])
	})

	t.Run("a trigger created enabled", func(t *testing.T) {
		t.Parallel()

		client := newFakeClient()
		s := newTestRepo(client)
		trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, true)

		require.NoError(t, s.CreateScheduledJob(ctx, trg))
		assert.Empty(t, client.paused)
		assert.Equal(t, schedulerpb.Job_ENABLED, client.states[s.jobName(trg.ID())])
	})

	t.Run("a trigger that is disabled", func(t *testing.T) {
		t.Parallel()

		client := newFakeClient()
		s := newTestRepo(client)
		trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, false)
		client.states[s.jobName(trg.ID())] = schedulerpb.Job_ENABLED

		require.NoError(t, s.UpdateScheduledJob(ctx, trg))
		assert.Equal(t, []string{s.jobName(trg.ID())}, client.paused)
		assert.Equal(t, schedulerpb.Job_PAUSED, client.states[s.jobName(trg.ID())])
	})

	t.Run("a trigger that is enabled again", func(t *testing.T) {
		t.Parallel()

		client := newFakeClient()
		s := newTestRepo(client)
		trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, true)
		client.states[s.jobName(trg.ID())] = schedulerpb.Job_PAUSED

		require.NoError(t, s.UpdateScheduledJob(ctx, trg))
		assert.Equal(t, []string{s.jobName(trg.ID())}, client.resumed)
		assert.Equal(t, schedulerpb.Job_ENABLED, client.states[s.jobName(trg.ID())])
	})

	t.Run("a disabled trigger edited again", func(t *testing.T) {
		t.Parallel()

		client := newFakeClient()
		s := newTestRepo(client)
		trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, false)
		client.states[s.jobName(trg.ID())] = schedulerpb.Job_PAUSED

		require.NoError(t, s.UpdateScheduledJob(ctx, trg))
		assert.Empty(t, client.paused)
		assert.Equal(t, schedulerpb.Job_PAUSED, client.states[s.jobName(trg.ID())])
	})

	t.Run("an enabled trigger edited again", func(t *testing.T) {
		t.Parallel()

		client := newFakeClient()
		s := newTestRepo(client)
		trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, true)
		client.states[s.jobName(trg.ID())] = schedulerpb.Job_ENABLED

		require.NoError(t, s.UpdateScheduledJob(ctx, trg))
		assert.Empty(t, client.resumed)
		assert.Equal(t, schedulerpb.Job_ENABLED, client.states[s.jobName(trg.ID())])
	})
	// A failed resume fails the update, so its user is told that the enabled
	// trigger's job stays paused and never calls.
	t.Run("a resume that fails", func(t *testing.T) {
		t.Parallel()

		client := newFakeClient()
		client.resumeErr = status.Error(codes.Unavailable, "unavailable")
		s := newTestRepo(client)
		trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, true)
		client.states[s.jobName(trg.ID())] = schedulerpb.Job_PAUSED

		assert.Error(t, s.UpdateScheduledJob(ctx, trg))
	})

	// A failed pause does not fail the update: a job left enabled only calls
	// for runs the server skips, because the trigger is disabled.
	t.Run("a pause that fails", func(t *testing.T) {
		t.Parallel()

		client := newFakeClient()
		client.pauseErr = status.Error(codes.Unavailable, "unavailable")
		s := newTestRepo(client)
		trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, false)
		client.states[s.jobName(trg.ID())] = schedulerpb.Job_ENABLED

		assert.NoError(t, s.UpdateScheduledJob(ctx, trg))
		assert.NoError(t, s.CreateScheduledJob(ctx, timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, false)))
	})
}

// A trigger can be left without a job, when creating its job failed, so
// updating creates the job a trigger does not have. Otherwise none of its
// edits could be saved, not even disabling it.
func TestSchedulerRepo_UpdateCreatesAMissingJob(t *testing.T) {
	t.Parallel()

	for _, enabled := range []bool{true, false} {
		name := "an enabled trigger"
		want := schedulerpb.Job_ENABLED
		if !enabled {
			name, want = "a disabled trigger", schedulerpb.Job_PAUSED
		}
		t.Run(name, func(t *testing.T) {
			t.Parallel()

			client := newFakeClient()
			s := newTestRepo(client)
			trg := timeDrivenTrigger(t, trigger.TimeIntervalEveryDay, enabled)

			require.NoError(t, s.UpdateScheduledJob(context.Background(), trg))
			require.Len(t, client.created, 1)
			assert.Equal(t, s.jobName(trg.ID()), client.created[0].GetName())
			assertRetries(t, client.created[0], wantRetry{retryCount: 5, minBackoff: 5 * time.Minute, maxBackoff: time.Hour, maxDoubling: 4})
			assert.Equal(t, want, client.states[s.jobName(trg.ID())])
		})
	}
}
