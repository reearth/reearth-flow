package gcpscheduler

import (
	"context"
	"encoding/json"
	"errors"
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
	"google.golang.org/api/iterator"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/durationpb"
)

type SchedulerConfig struct {
	ProjectID string
	Location  string
	Host      string
	// ServiceAccountEmail is the identity in the OIDC token Cloud Scheduler
	// sends with each run, which /execute-scheduled requires. Empty, jobs are
	// sent without a token and every run is refused.
	ServiceAccountEmail string
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
	// listJobs returns every job under parent. It is separate from client
	// because the client's ListJobs returns an iterator a fake cannot build.
	listJobs func(ctx context.Context, parent string) ([]*schedulerpb.Job, error)
	config   SchedulerConfig
}

func NewScheduler(ctx context.Context, config SchedulerConfig) (gateway.Scheduler, error) {
	client, err := scheduler.NewCloudSchedulerClient(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to create scheduler client: %v", err)
	}

	return &SchedulerRepo{
		client: client,
		listJobs: func(ctx context.Context, parent string) ([]*schedulerpb.Job, error) {
			var jobs []*schedulerpb.Job
			it := client.ListJobs(ctx, &schedulerpb.ListJobsRequest{Parent: parent})
			for {
				job, err := it.Next()
				if errors.Is(err, iterator.Done) {
					return jobs, nil
				}
				if err != nil {
					return nil, err
				}
				jobs = append(jobs, job)
			}
		},
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

// UpdateScheduledJob makes t's job match t, creating the job when t has none:
// a trigger is left without one when creating it failed, and would otherwise
// fail every update from then on.
func (s *SchedulerRepo) UpdateScheduledJob(ctx context.Context, t *trigger.Trigger) error {
	job, err := s.buildJob(t)
	if err != nil {
		return err
	}

	req := &schedulerpb.UpdateJobRequest{
		Job: job,
	}

	updated, err := s.client.UpdateJob(ctx, req)
	if status.Code(err) == codes.NotFound {
		log.Warnfc(ctx, "[Scheduler] Scheduled job %s does not exist; creating it", job.Name)
		return s.CreateScheduledJob(ctx, t)
	}
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
//
// A failed resume is returned, so its user is told that the enabled trigger's
// job does not call. A failed pause is only logged: a job left enabled only
// calls for runs the server skips, because the trigger is disabled.
func (s *SchedulerRepo) matchEnabled(ctx context.Context, job *schedulerpb.Job, enabled bool) error {
	switch {
	case !enabled && job.GetState() == schedulerpb.Job_ENABLED:
		if _, err := s.client.PauseJob(ctx, &schedulerpb.PauseJobRequest{Name: job.GetName()}); err != nil {
			log.Errorfc(ctx, "[Scheduler] Failed to pause scheduled job %s; its runs are skipped while the trigger is disabled: %v", job.GetName(), err)
			return nil
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
			HttpTarget: s.httpTarget(targetURL, body),
		},
		RetryConfig:     retryConfigFor(*t.TimeInterval()),
		AttemptDeadline: durationpb.New(trigger.AttemptDeadline),
	}, nil
}

// httpTarget is the request Cloud Scheduler sends for a run. With a service
// account configured it carries an OIDC token for that account, audience
// Host, so /execute-scheduled can tell a scheduled run from any other caller.
func (s *SchedulerRepo) httpTarget(uri string, body []byte) *schedulerpb.HttpTarget {
	target := &schedulerpb.HttpTarget{
		Uri:        uri,
		HttpMethod: schedulerpb.HttpMethod_POST,
		Headers: map[string]string{
			"Content-Type": "application/json",
		},
		Body: body,
	}
	if s.config.ServiceAccountEmail != "" {
		target.AuthorizationHeader = &schedulerpb.HttpTarget_OidcToken{
			OidcToken: &schedulerpb.OidcToken{
				ServiceAccountEmail: s.config.ServiceAccountEmail,
				Audience:            s.config.Host,
			},
		}
	}
	return target
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
	if status.Code(err) == codes.NotFound {
		// Already gone, which is what deleting it was for.
		return nil
	}
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

func (s *SchedulerRepo) ListScheduledJobs(ctx context.Context) ([]gateway.ScheduledTriggerJob, error) {
	jobs, err := s.listJobs(ctx, s.parent())
	if err != nil {
		return nil, fmt.Errorf("failed to list scheduled jobs: %w", err)
	}

	res := make([]gateway.ScheduledTriggerJob, 0, len(jobs))
	for _, job := range jobs {
		jobID := job.GetName()[strings.LastIndex(job.GetName(), "/")+1:]
		rest, ok := strings.CutPrefix(jobID, "trigger-")
		if !ok {
			continue // not a trigger's job
		}
		triggerID, err := id.TriggerIDFrom(rest)
		if err != nil {
			log.Warnfc(ctx, "[Scheduler] skipping job %s: not a trigger ID: %v", job.GetName(), err)
			continue
		}
		res = append(res, gateway.ScheduledTriggerJob{
			TriggerID:     triggerID,
			Authenticated: s.sendsToken(job),
		})
	}
	return res, nil
}

// sendsToken is whether job sends the token httpTarget would give it now. With
// no service account configured there is no token to send, so every job is as
// it should be.
func (s *SchedulerRepo) sendsToken(job *schedulerpb.Job) bool {
	if s.config.ServiceAccountEmail == "" {
		return true
	}
	oidc := job.GetHttpTarget().GetOidcToken()
	return oidc.GetServiceAccountEmail() == s.config.ServiceAccountEmail && oidc.GetAudience() == s.config.Host
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
