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
)

// fakeClient records the jobs it is asked to create and update.
type fakeClient struct {
	created []*schedulerpb.Job
	updated []*schedulerpb.Job
}

func (f *fakeClient) CreateJob(_ context.Context, req *schedulerpb.CreateJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	f.created = append(f.created, req.Job)
	return req.Job, nil
}

func (f *fakeClient) GetJob(_ context.Context, _ *schedulerpb.GetJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	return nil, nil
}

func (f *fakeClient) UpdateJob(_ context.Context, req *schedulerpb.UpdateJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	f.updated = append(f.updated, req.Job)
	return req.Job, nil
}

func (f *fakeClient) DeleteJob(_ context.Context, _ *schedulerpb.DeleteJobRequest, _ ...gax.CallOption) error {
	return nil
}

func (f *fakeClient) Close() error { return nil }

func timeDrivenTrigger(t *testing.T, interval trigger.TimeInterval) *trigger.Trigger {
	t.Helper()
	trg, err := trigger.New().
		NewID().
		Workspace(accountsid.NewWorkspaceID()).
		Deployment(id.NewDeploymentID()).
		Description("nightly").
		EventSource("TIME_DRIVEN").
		TimeInterval(interval).
		Enabled(true).
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

			client := &fakeClient{}
			s := &SchedulerRepo{client: client, config: SchedulerConfig{ProjectID: "p", Location: "l", Host: "https://api.example.com"}}

			require.NoError(t, s.CreateScheduledJob(context.Background(), timeDrivenTrigger(t, tc.interval)))
			require.Len(t, client.created, 1)
			assertRetries(t, client.created[0], tc.want)
		})

		t.Run(tc.name+" on update", func(t *testing.T) {
			t.Parallel()

			client := &fakeClient{}
			s := &SchedulerRepo{client: client, config: SchedulerConfig{ProjectID: "p", Location: "l", Host: "https://api.example.com"}}

			require.NoError(t, s.UpdateScheduledJob(context.Background(), timeDrivenTrigger(t, tc.interval)))
			require.Len(t, client.updated, 1)
			assertRetries(t, client.updated[0], tc.want)
		})
	}
}

// Every retry of a run has to end before the next run is due, or Cloud
// Scheduler may skip that run.
func TestRetryConfigFor_EndsBeforeTheNextRun(t *testing.T) {
	t.Parallel()

	// The shortest gap between two runs of each interval.
	gaps := map[trigger.TimeInterval]time.Duration{
		trigger.TimeIntervalEveryHour:  time.Hour,
		trigger.TimeIntervalEveryDay:   24 * time.Hour,
		trigger.TimeIntervalEveryWeek:  7 * 24 * time.Hour,
		trigger.TimeIntervalEveryMonth: 28 * 24 * time.Hour,
	}

	for interval, gap := range gaps {
		t.Run(string(interval), func(t *testing.T) {
			t.Parallel()

			rc := retryConfigFor(interval)
			total := time.Duration(rc.GetRetryCount()+1) * attemptDeadline
			wait := rc.GetMinBackoffDuration().AsDuration()
			for i := int32(0); i < rc.GetRetryCount(); i++ {
				total += min(wait, rc.GetMaxBackoffDuration().AsDuration())
				if i < rc.GetMaxDoublings() {
					wait *= 2
				}
			}
			assert.Less(t, total, gap, "the worst case of %s is %s", interval, total)
		})
	}
}
