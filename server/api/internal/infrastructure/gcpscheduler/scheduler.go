package gcpscheduler

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"

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
	PauseJob(ctx context.Context, req *schedulerpb.PauseJobRequest, opts ...gax.CallOption) (*schedulerpb.Job, error)
	ResumeJob(ctx context.Context, req *schedulerpb.ResumeJobRequest, opts ...gax.CallOption) (*schedulerpb.Job, error)
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

	created, err := s.client.CreateJob(ctx, req)
	if err != nil {
		log.Debugfc(ctx, "[Scheduler] Error creating scheduled job: %v", err)
		return fmt.Errorf("failed to create scheduled job: %v", err)
	}

	log.Debugfc(ctx, "[Scheduler] Scheduled job created successfully: name=%s", job.Name)
	return s.matchEnabled(ctx, created, t.Enabled())
}

func (s *SchedulerRepo) UpdateScheduledJob(ctx context.Context, t *trigger.Trigger) error {
	job, err := s.buildJob(t)
	if err != nil {
		return err
	}

	req := &schedulerpb.UpdateJobRequest{
		Job: job,
	}

	updated, err := s.client.UpdateJob(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to update scheduled job: %v", err)
	}

	log.Debugfc(ctx, "[Scheduler] Scheduled job updated successfully: name=%s", job.Name)
	return s.matchEnabled(ctx, updated, t.Enabled())
}

// matchEnabled pauses the job of a disabled trigger and resumes the job of an
// enabled one, so Cloud Scheduler only calls the server for triggers that can
// run. A job's state cannot be sent with the job: Cloud Scheduler sets it, and
// only pauses an enabled job and resumes a paused one, so the state the job
// was saved with decides the call.
func (s *SchedulerRepo) matchEnabled(ctx context.Context, job *schedulerpb.Job, enabled bool) error {
	switch {
	case !enabled && job.GetState() == schedulerpb.Job_ENABLED:
		if _, err := s.client.PauseJob(ctx, &schedulerpb.PauseJobRequest{Name: job.GetName()}); err != nil {
			return fmt.Errorf("failed to pause scheduled job: %v", err)
		}
		log.Debugfc(ctx, "[Scheduler] Scheduled job paused: name=%s", job.GetName())
	case enabled && job.GetState() == schedulerpb.Job_PAUSED:
		if _, err := s.client.ResumeJob(ctx, &schedulerpb.ResumeJobRequest{Name: job.GetName()}); err != nil {
			return fmt.Errorf("failed to resume scheduled job: %v", err)
		}
		log.Debugfc(ctx, "[Scheduler] Scheduled job resumed: name=%s", job.GetName())
	}
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
		AttemptDeadline: durationpb.New(trigger.AttemptDeadline),
	}, nil
}

// retryConfigFor is the trigger's retry policy as Cloud Scheduler's retry
// config. The retry duration is left unset: set together with a retry count,
// Cloud Scheduler retries until both are reached.
func retryConfigFor(interval trigger.TimeInterval) *schedulerpb.RetryConfig {
	p := interval.RetryPolicy()
	return &schedulerpb.RetryConfig{
		RetryCount:         int32(p.Count),
		MinBackoffDuration: durationpb.New(p.MinBackoff),
		MaxBackoffDuration: durationpb.New(p.MaxBackoff),
		MaxDoublings:       int32(p.MaxDoublings),
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
