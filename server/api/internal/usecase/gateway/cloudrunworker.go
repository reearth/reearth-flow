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
	// PreviewSchema POSTs to the Service's dedicated probe-schema route and blocks
	// until the probe completes. Returns the terminal infra status. This is a
	// distinct seam from RunJob: it targets a different worker route that runs the
	// `reearth-flow-worker probe-schema` subcommand, not `run`.
	PreviewSchema(ctx context.Context, p ProbeSchemaParam) (JobStatus, error)
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

// ProbeSchemaParam carries the inputs for the worker's probe-schema route.
type ProbeSchemaParam struct {
	Variables   map[string]string
	SampleSize  *int
	WorkflowURL string
	ReportURL   string
	JobID       id.JobID
}
