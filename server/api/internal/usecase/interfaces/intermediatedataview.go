package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

var (
	// ErrJobNotFinished is a view asked of a job that is still running. The
	// intermediate-data file is still being written, so a view of it would
	// disagree with the table.
	ErrJobNotFinished = errors.New("job has not finished")
	// ErrViewsUnavailable is a view asked of a deployment with no worker
	// service configured, since only the engine worker renders views.
	ErrViewsUnavailable = errors.New("intermediate data views are not available")
	// ErrRenderTimedOut is a render whose worker never answered within the
	// request's wait. A merely slow render is stopped by the worker's own,
	// shorter limit and reported as a failed view instead.
	ErrRenderTimedOut = errors.New("the view did not finish rendering in time")
	// ErrViewWorkersBusy is a render turned away because every worker that
	// renders views was busy. The pool is shared with debug runs and schema
	// previews, so this is not a fault.
	ErrViewWorkersBusy = errors.New("no worker is free to render the view")
	// ErrIntermediateDataNotFound is a view asked of an output port that left
	// no intermediate data — a node that never ran, or a run with the feature
	// writer disabled.
	ErrIntermediateDataNotFound = errors.New("intermediate data not found")
	// ErrConcurrentRenderLimit is a render refused because the workspace
	// already has as many renders in progress as its plan allows.
	ErrConcurrentRenderLimit = errors.New("this workspace already has as many views rendering as its plan allows")
)

// RenderIntermediateDataViewParam fields are ordered for struct packing
// (govet fieldalignment), not by meaning.
type RenderIntermediateDataViewParam struct {
	// FileID is the engine's `[subgraphPrefix.]nodeID.port`, the same id the
	// editor builds to fetch the table.
	FileID  string
	Request featureview.Request
	// JobID is the finished job whose feature-store holds the input.
	JobID id.JobID
}

// IntermediateDataViewLayer is one entry point of a ready view. Fields are
// ordered for struct packing (govet fieldalignment), not by meaning.
type IntermediateDataViewLayer struct {
	// SizeLimitedFeatures and SizeLimitedTiles are nil for every format but
	// vector tiles, which alone have a size limit.
	SizeLimitedFeatures *int
	SizeLimitedTiles    *int
	Format              featureview.Format
	EntryPointURL       string
	SelectedFeatures    int
	RenderedFeatures    int
}

// IntermediateDataViewResult fields are ordered for struct packing (govet
// fieldalignment), not by meaning.
type IntermediateDataViewResult struct {
	// SelectedFeatures and RenderedFeatures are set for any terminal status,
	// including EMPTY, where they explain why. They are nil only when the
	// render's report could not be read.
	SelectedFeatures *int
	RenderedFeatures *int
	// SizeLimitedFeatures and SizeLimitedTiles say what the size limit left
	// out of the vector tile layer. They are nil when there is no such layer
	// or the report could not be read.
	SizeLimitedFeatures *int
	SizeLimitedTiles    *int
	Error               *string
	// Key identifies the view by its content: the same request always yields
	// the same key, so a repeat request reuses a rendered view.
	Key    string
	FileID string
	Shape  featureview.Shape
	Status featureview.Status
	// Layers are the view's entry points, 3D Tiles before vector tiles. The
	// engine picks formats from the geometry, so Layers are empty unless
	// Status is ready.
	Layers []IntermediateDataViewLayer
	JobID  id.JobID
}

type IntermediateDataView interface {
	// Render returns an existing view, or renders one and waits for it. Either
	// way the result is terminal: there is no in-flight state to poll.
	Render(context.Context, RenderIntermediateDataViewParam) (*IntermediateDataViewResult, error)
	// Get reads an already-rendered view, without rendering. A nil result with
	// a nil error means no view exists for the key.
	Get(ctx context.Context, jobID id.JobID, fileID, key string) (*IntermediateDataViewResult, error)
}
