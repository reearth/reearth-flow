package gateway

import (
	"context"

	"github.com/google/uuid"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
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
	// RenderView POSTs to the Service's dedicated render-view route and blocks
	// until the render completes. Returns the terminal infra status. Like
	// PreviewSchema this is its own seam, not a flag on RunJob: it targets a
	// different worker route running the `reearth-flow-worker render-view`
	// subcommand, and it consumes an artifact rather than a workflow.
	//
	// Unlike the other two, this one is called from the request path rather
	// than a goroutine: a view is a seconds-long artifact transform, and its
	// caller wants the answer, not a handle to poll.
	RenderView(ctx context.Context, p RenderViewParam) (JobStatus, error)
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

// RenderViewParam carries the inputs for the worker's render-view route.
//
// There is deliberately no WorkflowURL: a view is rendered from the
// intermediate data a finished run already left in storage, so this path
// uploads no workflow and consumes no metadata. There is no job id either —
// nothing tracks a view render, so the renderer has no completion event to
// publish and no work root to name.
type RenderViewParam struct {
	// InputURI is the gs:// intermediate-data file to render.
	InputURI string
	// OutputURI is the gs:// directory the view's files are written under.
	OutputURI string
	// ReportURI is the gs:// destination for the render's report.
	ReportURI string
	// Name is the view key, which the renderer uses as its output prefix.
	Name  string
	Shape featureview.Shape
	// Row is set for a gltf view, Filter may be set for a tiles view; the two
	// are mutually exclusive and validated before dispatch.
	Row     *int
	Filter  *string
	Options featureview.Options
}
