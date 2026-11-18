package gateway

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// ErrWorkerBusy is a request the worker Service turned away because every
// instance it may run was busy. Nothing ran.
var ErrWorkerBusy = errors.New("the worker service has no free instance")

// CloudRunWorker runs debug jobs on a min=0 Cloud Run Service.
type CloudRunWorker interface {
	// RunJob POSTs to the Service and blocks until the workflow completes.
	// Returns the terminal infra status (COMPLETED / FAILED / CANCELLED).
	RunJob(ctx context.Context, p RunJobParam) (JobStatus, error)
	// PreviewSchema POSTs to the Service's probe-schema route, which runs the
	// worker's `probe-schema` subcommand, and blocks until it completes.
	PreviewSchema(ctx context.Context, p ProbeSchemaParam) (JobStatus, error)
	// RenderView POSTs to the Service's render-view route and blocks until the
	// render completes. Unlike the others it is called from the request path,
	// not a goroutine.
	RenderView(ctx context.Context, p RenderViewParam) (JobStatus, error)
	// CancelJob writes the cancel flag for jobID so the wrapper kills the subprocess.
	CancelJob(ctx context.Context, jobID id.JobID) error
	// EngineVersion is part of a view's key, since a cached view outlives the
	// engine that rendered it. Implementations cache it to avoid a request per
	// render.
	EngineVersion(ctx context.Context) (string, error)
}

// ErrRunNotStarted wraps a RunJob error after which the worker is known not to
// have started the run: the request never reached it, or it refused the
// request before starting anything. Any other error may follow a run that
// started.
var ErrRunNotStarted = errors.New("the worker did not start the run")

type RunJobParam struct {
	Variables     map[string]string
	PreviousJobID *id.JobID
	StartNodeID   *uuid.UUID
	WorkflowURL   string
	MetadataURL   string
	JobID         id.JobID
	// MaxRunDuration is how long the run may take before it is cancelled; 0
	// leaves the Service's own request timeout.
	MaxRunDuration time.Duration
}

// ProbeSchemaParam carries the inputs for the worker's probe-schema route.
type ProbeSchemaParam struct {
	Variables   map[string]string
	SampleSize  *int
	WorkflowURL string
	ReportURL   string
	JobID       id.JobID
}

// RenderViewParam has no WorkflowURL or job id: a view is rendered from a
// finished run's intermediate data, and nothing tracks the render as a job.
type RenderViewParam struct {
	InputURI string
	// OutputURI is the gs:// directory the view's files are written under.
	OutputURI string
	ReportURI string
	// Name is the view key, which the renderer uses as its output prefix.
	Name  string
	Shape featureview.Shape
	// Row is set for a gltf view, Filter may be set for a tiles view; the two
	// are mutually exclusive and validated before dispatch.
	Row    *int
	Filter *string
	// TilesURL is the absolute tile URL template a vector tileset's tilejson
	// names. Unset, the engine writes a root-relative one.
	TilesURL *string
	Options  featureview.Options
}
