package gateway

import (
	"context"

	"github.com/google/uuid"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// CloudRunWorker runs debug jobs on a min=0 Cloud Run Service.
// RunJob holds until the run finishes (the HTTP response is the infra status).
type CloudRunWorker interface {
	// RunJob POSTs to the Service and blocks until the workflow completes.
	// Returns the terminal infra status (COMPLETED / FAILED / CANCELLED).
	RunJob(ctx context.Context, p RunJobParam) (JobStatus, error)
	// CancelJob writes the cancel flag for jobID so the wrapper kills the subprocess.
	CancelJob(ctx context.Context, jobID id.JobID) error
}

type RunJobParam struct {
	Variables     map[string]string
	PreviousJobID *id.JobID
	StartNodeID   *uuid.UUID
	WorkflowURL   string
	MetadataURL   string
	JobID         id.JobID
}
