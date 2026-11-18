package interactor

import (
	"context"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

// Job events are structured log lines for usage analytics, routed to BigQuery by
// a Cloud Logging sink on jsonPayload.event and joined with the Batch job labels
// on workspace_id and job_id. A job that finished while no monitoring loop
// watched it, such as across a server restart, logs no finished event.
const (
	jobEventSubmitted    = "flow.job.submitted"
	jobEventSubmitFailed = "flow.job.submit_failed"
	jobEventFinished     = "flow.job.finished"
	jobEventRejected     = "flow.job.rejected"
)

// What started a job, logged as the trigger field of a submitted event.
const (
	jobTriggerManual     = "manual"
	jobTriggerDeployment = "deployment"
	jobTriggerAPI        = "api"
	jobTriggerSchedule   = "schedule"
	jobTriggerCMS        = "cms"
)

// Why the plan refused a job, logged as the reason field of a rejected event.
const (
	jobRejectedAPITriggerCalls       = "api_trigger_calls"
	jobRejectedDebugConcurrentRuns   = "debug_concurrent_runs"
	jobRejectedDebugRunsPerHour      = "debug_runs_per_hour"
	jobRejectedDeploymentRuns        = "deployment_runs"
	jobRejectedTimeDrivenTriggerRuns = "time_driven_trigger_runs"
)

// Where a job runs, logged as the backend field.
const (
	jobBackendBatch    = "batch"
	jobBackendCloudRun = "cloudrun"
)

func logJobEvent(ctx context.Context, event string, j *job.Job, kv ...any) {
	logJobEventOn(ctx, event, j, jobBackend(j), kv...)
}

// logJobEventOn is logJobEvent for a job whose backend cannot be told from the
// job itself, such as a Batch submission that failed before it got a Batch job.
func logJobEventOn(ctx context.Context, event string, j *job.Job, backend string, kv ...any) {
	fields := []any{
		"event", event,
		"job_id", j.ID().String(),
		"workspace_id", j.Workspace().String(),
		"mode", string(j.Mode()),
		"status", string(j.Status()),
		"backend", backend,
	}
	if pid := j.ProjectID(); pid != nil {
		fields = append(fields, "project_id", pid.String())
	}
	if did := j.Deployment(); did != nil {
		fields = append(fields, "deployment_id", did.String())
	}
	log.GetLoggerFromContextOrDefault(ctx).Infow(event, append(fields, kv...)...)
}

// logJobFinished logs the finished event for j, with the time from the job's
// request to its completion. elapsed_sec therefore includes upload and queue
// time, since no execution-start timestamp is recorded.
func logJobFinished(ctx context.Context, j *job.Job) {
	end := time.Now()
	if c := j.CompletedAt(); c != nil {
		end = *c
	}
	logJobEvent(ctx, jobEventFinished, j, "elapsed_sec", end.Sub(j.StartedAt()).Seconds())
}

// logJobRejected logs that the plan refused a job for wid. There is no job
// yet, so only the workspace is known.
func logJobRejected(ctx context.Context, wid accountsid.WorkspaceID, reason string, kv ...any) {
	fields := []any{
		"event", jobEventRejected,
		"workspace_id", wid.String(),
		"reason", reason,
	}
	log.GetLoggerFromContextOrDefault(ctx).Infow(jobEventRejected, append(fields, kv...)...)
}

// jobBackend is where j runs: a job with a Batch job behind it runs on Batch,
// any other on the Cloud Run worker.
func jobBackend(j *job.Job) string {
	if j.GCPJobID() != "" {
		return jobBackendBatch
	}
	return jobBackendCloudRun
}
