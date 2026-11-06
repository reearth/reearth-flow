package gcpscheduler

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	scheduler "cloud.google.com/go/scheduler/apiv1"
	schedulerpb "cloud.google.com/go/scheduler/apiv1/schedulerpb"
	"github.com/googleapis/gax-go/v2"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/reearth/reearthx/log"
	"google.golang.org/protobuf/types/known/durationpb"
)

type SchedulerConfig struct {
	ProjectID string
	Location  string
	Host      string
}

type SchedulerClient interface {
	CreateJob(ctx context.Context, req *schedulerpb.CreateJobRequest, opts ...gax.CallOption) (*schedulerpb.Job, error)
	GetJob(ctx context.Context, req *schedulerpb.GetJobRequest, opts ...gax.CallOption) (*schedulerpb.Job, error)
	UpdateJob(ctx context.Context, req *schedulerpb.UpdateJobRequest, opts ...gax.CallOption) (*schedulerpb.Job, error)
	DeleteJob(ctx context.Context, req *schedulerpb.DeleteJobRequest, opts ...gax.CallOption) error
	Close() error
}

type SchedulerRepo struct {
	client SchedulerClient
	config SchedulerConfig
}

func NewScheduler(ctx context.Context, config SchedulerConfig) (gateway.Scheduler, error) {
	client, err := scheduler.NewCloudSchedulerClient(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to create scheduler client: %v", err)
	}

	return &SchedulerRepo{
		client: client,
		config: config,
	}, nil
}

type scheduledPayload struct {
	With      map[string]string `json:"with,omitempty"`
	TriggerID string            `json:"triggerID"`
}

func (s *SchedulerRepo) CreateScheduledJob(ctx context.Context, t *trigger.Trigger) error {
	job, err := s.buildJob(t)
	if err != nil {
		return err
	}

	req := &schedulerpb.CreateJobRequest{
		Parent: s.parent(),
		Job:    job,
	}

	_, err = s.client.CreateJob(ctx, req)
	if err != nil {
		log.Debugfc(ctx, "[Scheduler] Error creating scheduled job: %v", err)
		return fmt.Errorf("failed to create scheduled job: %v", err)
	}

	log.Debugfc(ctx, "[Scheduler] Scheduled job created successfully: name=%s", job.Name)
	return nil
}

func (s *SchedulerRepo) UpdateScheduledJob(ctx context.Context, t *trigger.Trigger) error {
	job, err := s.buildJob(t)
	if err != nil {
		return err
	}

	req := &schedulerpb.UpdateJobRequest{
		Job: job,
	}

	_, err = s.client.UpdateJob(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to update scheduled job: %v", err)
	}

	log.Debugfc(ctx, "[Scheduler] Scheduled job updated successfully: name=%s", job.Name)
	return nil
}

// buildJob is the whole Cloud Scheduler job for a time-driven trigger. Create
// and update both send all of it, so a job always matches its trigger.
func (s *SchedulerRepo) buildJob(t *trigger.Trigger) (*schedulerpb.Job, error) {
	if t.EventSource() != "TIME_DRIVEN" {
		return nil, fmt.Errorf("trigger is not time-driven")
	}

	if t.TimeInterval() == nil {
		return nil, fmt.Errorf("time interval is required for time-driven triggers")
	}

	schedule, err := s.convertTimeIntervalToSchedule(*t.TimeInterval())
	if err != nil {
		return nil, fmt.Errorf("failed to convert time interval to schedule: %v", err)
	}

	targetURL := fmt.Sprintf("%s/api/triggers/%s/execute-scheduled", s.config.Host, t.ID().String())

	varsMap := variable.SliceToMap(t.Variables())
	workerVars := variable.ToWorkerMap(varsMap)

	body, err := json.Marshal(scheduledPayload{
		TriggerID: t.ID().String(),
		With:      workerVars,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to marshal scheduled job payload: %w", err)
	}

	return &schedulerpb.Job{
		Name:        s.jobName(t.ID()),
		Description: fmt.Sprintf("Scheduled trigger for deployment %s", t.Deployment().String()),
		Schedule:    schedule,
		TimeZone:    "UTC",
		Target: &schedulerpb.Job_HttpTarget{
			HttpTarget: &schedulerpb.HttpTarget{
				Uri:        targetURL,
				HttpMethod: schedulerpb.HttpMethod_POST,
				Headers: map[string]string{
					"Content-Type": "application/json",
				},
				Body: body,
			},
		},
		RetryConfig:     retryConfigFor(*t.TimeInterval()),
		AttemptDeadline: durationpb.New(attemptDeadline),
	}, nil
}

// attemptDeadline is how long Cloud Scheduler waits for one call to answer
// before it counts as failed. It is Cloud Scheduler's default, set so a job
// does not depend on it.
const attemptDeadline = 3 * time.Minute

// retryConfigFor is how a run refused by the server, such as one refused
// because the workspace already runs as many jobs as its plan allows, is tried
// again. Cloud Scheduler ignores Retry-After and allows at most 5 retries, so
// the waits double from a start that suits the interval. Every retry ends well
// before the next run, which Cloud Scheduler may skip while a run is still
// retrying. The retry duration is left unset: set together with a retry
// count, retries continue until both are reached.
func retryConfigFor(interval trigger.TimeInterval) *schedulerpb.RetryConfig {
	// 1, 2, 4, 8 and 16 minutes: about half an hour in all.
	minBackoff, maxBackoff := time.Minute, 16*time.Minute
	if interval != trigger.TimeIntervalEveryHour {
		// 5, 10, 20, 40 and 60 minutes: about two and a quarter hours.
		minBackoff, maxBackoff = 5*time.Minute, time.Hour
	}
	return &schedulerpb.RetryConfig{
		RetryCount:         5,
		MinBackoffDuration: durationpb.New(minBackoff),
		MaxBackoffDuration: durationpb.New(maxBackoff),
		MaxDoublings:       4,
	}
}

func (s *SchedulerRepo) parent() string {
	return fmt.Sprintf("projects/%s/locations/%s", s.config.ProjectID, s.config.Location)
}

func (s *SchedulerRepo) jobName(triggerID id.TriggerID) string {
	return fmt.Sprintf("%s/jobs/%s", s.parent(), formatSchedulerJobID(triggerID.String()))
}

func (s *SchedulerRepo) DeleteScheduledJob(ctx context.Context, triggerID id.TriggerID) error {
	name := s.jobName(triggerID)

	req := &schedulerpb.DeleteJobRequest{
		Name: name,
	}

	err := s.client.DeleteJob(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to delete scheduled job: %v", err)
	}

	log.Debugfc(ctx, "[Scheduler] Scheduled job deleted successfully: name=%s", name)
	return nil
}

func (s *SchedulerRepo) GetScheduledJob(ctx context.Context, triggerID id.TriggerID) (*gateway.ScheduledJobInfo, error) {
	name := s.jobName(triggerID)

	req := &schedulerpb.GetJobRequest{
		Name: name,
	}

	job, err := s.client.GetJob(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to get scheduled job: %v", err)
	}

	return &gateway.ScheduledJobInfo{
		Name:         job.Name,
		Schedule:     job.Schedule,
		State:        convertSchedulerStateToGatewayState(job.State),
		LastAttempt:  job.LastAttemptTime,
		NextSchedule: job.ScheduleTime,
	}, nil
}

func (s *SchedulerRepo) Close() error {
	return s.client.Close()
}

func (s *SchedulerRepo) convertTimeIntervalToSchedule(interval trigger.TimeInterval) (string, error) {
	switch interval {
	case trigger.TimeIntervalEveryHour:
		return "0 * * * *", nil
	case trigger.TimeIntervalEveryDay:
		return "0 0 * * *", nil
	case trigger.TimeIntervalEveryWeek:
		return "0 0 * * 0", nil
	case trigger.TimeIntervalEveryMonth:
		return "0 0 1 * *", nil
	default:
		return "", fmt.Errorf("unsupported time interval: %s", interval)
	}
}

func formatSchedulerJobID(triggerID string) string {
	jobID := strings.ToLower(triggerID)
	jobID = strings.ReplaceAll(jobID, "_", "-")

	jobID = "trigger-" + jobID

	if len(jobID) > 500 {
		jobID = jobID[:500]
	}

	return jobID
}

func convertSchedulerStateToGatewayState(state schedulerpb.Job_State) gateway.ScheduledJobState {
	switch state {
	case schedulerpb.Job_ENABLED:
		return gateway.ScheduledJobStateEnabled
	case schedulerpb.Job_PAUSED:
		return gateway.ScheduledJobStatePaused
	case schedulerpb.Job_DISABLED:
		return gateway.ScheduledJobStateDisabled
	default:
		return gateway.ScheduledJobStateUnknown
	}
}
