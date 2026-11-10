package gateway

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type JobStatus string

const (
	JobStatusUnknown   JobStatus = "UNKNOWN"
	JobStatusPending   JobStatus = "PENDING"
	JobStatusRunning   JobStatus = "RUNNING"
	JobStatusCancelled JobStatus = "CANCELLED"
	JobStatusCompleted JobStatus = "COMPLETED"
	JobStatusFailed    JobStatus = "FAILED"
)

type JobInfo struct {
	Name   string
	Status JobStatus
	ID     id.JobID
}

// ErrJobNotSubmitted wraps a submission error after which the job is known
// not to exist: Batch refused it, or a lookup found no such job. Any other
// submission error leaves open whether the job was created and may run.
var ErrJobNotSubmitted = errors.New("the job was not submitted")

type Batch interface {
	// SubmitJob dispatches a run. maxRunDuration caps how long it may run below
	// the configured limit; 0 leaves the configured limit. An error wraps
	// ErrJobNotSubmitted only when the job is known not to exist.
	SubmitJob(ctx context.Context, jobID id.JobID, workflowsURL string, metadataURL string, variables map[string]string, projectID id.ProjectID, workspaceID accountsid.WorkspaceID, previousJobID *id.JobID, startNodeID *uuid.UUID, maxRunDuration time.Duration) (string, error)
	// SubmitProbeJob dispatches a Batch job invoking the `probe-schema` subcommand
	// (not `run`). It is the Batch fallback for the preview-schema pipeline.
	SubmitProbeJob(ctx context.Context, jobID id.JobID, workflowsURL string, variables map[string]string, sampleSize *int, reportURL string, projectID id.ProjectID, workspaceID accountsid.WorkspaceID) (string, error)
	GetJobStatus(ctx context.Context, jobName string) (JobStatus, error)
	ListJobs(ctx context.Context, projectID id.ProjectID) ([]JobInfo, error)
	CancelJob(ctx context.Context, jobName string) error
}
