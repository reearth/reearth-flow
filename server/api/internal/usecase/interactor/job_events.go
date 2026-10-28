package interactor

import (
	"context"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

// Job events are structured log lines for usage analytics. On GCP the logger writes JSON, so each key/value is a
// jsonPayload field in Cloud Logging, and a sink filtered on
// jsonPayload.event can route them to BigQuery. They join with the Batch job
// labels on workspace_id, and on job_id to the original_id label.
//
// They are best-effort: a job finished while no monitoring loop watched it,
// e.g. across a server restart, logs no finished event.
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
)

// Why the plan refused a job, logged as the reason field of a rejected event.
const (
	jobRejectedConcurrentJobs  = "concurrent_jobs"
	jobRejectedAPITriggerCalls = "api_trigger_calls"
)

// Where a job runs, logged as the backend field.
const (
	jobBackendBatch    = "batch"
	jobBackendCloudRun = "cloudrun"
)

// logJobEvent logs event for j, with kv as extra fields.
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
// request to its completion. StartedAt is set when the job is requested, so
// elapsed_sec includes upload and queue time, not only execution: no
// execution-start timestamp is recorded.
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
