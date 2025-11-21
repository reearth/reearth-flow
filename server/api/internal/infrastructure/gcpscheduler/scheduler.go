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
	"github.com/reearth/reearthx/log"
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
	TriggerID string            `json:"triggerID"`
	With      map[string]string `json:"with,omitempty"`
}

func (s *SchedulerRepo) CreateScheduledJob(ctx context.Context, t *trigger.Trigger) error {
	if t.EventSource() != "TIME_DRIVEN" {
		return fmt.Errorf("trigger is not time-driven")
	}

	if t.TimeInterval() == nil {
		return fmt.Errorf("time interval is required for time-driven triggers")
	}

	parent := fmt.Sprintf("projects/%s/locations/%s", s.config.ProjectID, s.config.Location)
	jobID := formatSchedulerJobID(t.ID().String())
	name := fmt.Sprintf("%s/jobs/%s", parent, jobID)

	schedule, err := s.convertTimeIntervalToSchedule(*t.TimeInterval())
	if err != nil {
		return fmt.Errorf("failed to convert time interval to schedule: %v", err)
	}

	targetURL := fmt.Sprintf("%s/api/triggers/%s/execute-scheduled", s.config.Host, t.ID().String())

	body, err := json.Marshal(scheduledPayload{
		TriggerID: t.ID().String(),
		With:      t.Variables(),
	})
	if err != nil {
		return fmt.Errorf("failed to marshal scheduled job payload: %w", err)
	}

	job := &schedulerpb.Job{
		Name:        name,
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
	}

	req := &schedulerpb.CreateJobRequest{
		Parent: parent,
		Job:    job,
	}

	_, err = s.client.CreateJob(ctx, req)
	if err != nil {
		log.Debugfc(ctx, "[Scheduler] Error creating scheduled job: %v", err)
		return fmt.Errorf("failed to create scheduled job: %v", err)
	}

	log.Debugfc(ctx, "[Scheduler] Scheduled job created successfully: name=%s", name)
	return nil
}

func (s *SchedulerRepo) UpdateScheduledJob(ctx context.Context, t *trigger.Trigger) error {
	if t.EventSource() != "TIME_DRIVEN" {
		return fmt.Errorf("trigger is not time-driven")
	}

	if t.TimeInterval() == nil {
		return fmt.Errorf("time interval is required for time-driven triggers")
	}

	parent := fmt.Sprintf("projects/%s/locations/%s", s.config.ProjectID, s.config.Location)
	jobID := formatSchedulerJobID(t.ID().String())
	name := fmt.Sprintf("%s/jobs/%s", parent, jobID)

	schedule, err := s.convertTimeIntervalToSchedule(*t.TimeInterval())
	if err != nil {
		return fmt.Errorf("failed to convert time interval to schedule: %v", err)
	}

	targetURL := fmt.Sprintf("%s/api/triggers/%s/execute-scheduled", s.config.Host, t.ID().String())

	body, err := json.Marshal(scheduledPayload{
		TriggerID: t.ID().String(),
		With:      t.Variables(),
	})
	if err != nil {
		return fmt.Errorf("failed to marshal scheduled job payload: %w", err)
	}

	job := &schedulerpb.Job{
		Name:        name,
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
	}

	req := &schedulerpb.UpdateJobRequest{
		Job: job,
	}

	_, err = s.client.UpdateJob(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to update scheduled job: %v", err)
	}

	log.Debugfc(ctx, "[Scheduler] Scheduled job updated successfully: name=%s", name)
	return nil
}

func (s *SchedulerRepo) DeleteScheduledJob(ctx context.Context, triggerID id.TriggerID) error {
	parent := fmt.Sprintf("projects/%s/locations/%s", s.config.ProjectID, s.config.Location)
	jobID := formatSchedulerJobID(triggerID.String())
	name := fmt.Sprintf("%s/jobs/%s", parent, jobID)

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
	parent := fmt.Sprintf("projects/%s/locations/%s", s.config.ProjectID, s.config.Location)
	jobID := formatSchedulerJobID(triggerID.String())
	name := fmt.Sprintf("%s/jobs/%s", parent, jobID)

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
