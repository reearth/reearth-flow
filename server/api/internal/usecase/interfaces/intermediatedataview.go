package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

var (
	// ErrJobNotFinished is a view asked of a job that is still running. The
	// intermediate-data file is written as the run goes, so rendering a partial
	// one would produce a view that silently disagrees with the table.
	ErrJobNotFinished = errors.New("job has not finished")
	// ErrViewsUnavailable is a view asked of a deployment with no worker
	// service configured. Views are rendered on demand by the engine worker and
	// have no second pipeline, so where that is absent they are simply not
	// offered.
	ErrViewsUnavailable = errors.New("intermediate data views are not available")
	// ErrIntermediateDataNotFound is a view asked of an output port that left
	// no intermediate data — a node that never ran, or a run with the feature
	// writer disabled.
	ErrIntermediateDataNotFound = errors.New("intermediate data not found")
)

// RenderIntermediateDataViewParam identifies the intermediate data to render
// and how.
type RenderIntermediateDataViewParam struct {
	// JobID is the finished job whose feature-store holds the input.
	JobID id.JobID
	// FileID is the engine's `[subgraphPrefix.]nodeID.port`, the same id the
	// editor builds to fetch the table.
	FileID  string
	Request featureview.Request
}

// IntermediateDataViewResult is a view's current state.
//
// Format and EntryPointURL are empty until a render has finished, because the
// engine chooses the output format from the geometry it finds: a tiles view
// becomes 3D Tiles or vector tiles, and the server cannot know which before the
// render runs.
type IntermediateDataViewResult struct {
	// Key identifies the view by its content: the same request always yields
	// the same key, so a repeat request reuses a rendered view.
	Key    string
	JobID  id.JobID
	FileID string
	Shape  featureview.Shape
	Status featureview.Status

	Format        *featureview.Format
	EntryPointURL string
	// SelectedFeatures and RenderedFeatures are set for any terminal status,
	// including EMPTY: "0 of 900 selected" is the explanation, so withholding
	// the counts exactly when the view is empty would withhold the reason.
	SelectedFeatures *int
	RenderedFeatures *int
	Error            *string
}

type IntermediateDataView interface {
	// Render returns an existing view, or renders one and waits for it. Either
	// way the result is terminal: there is no in-flight state to poll.
	Render(context.Context, RenderIntermediateDataViewParam) (*IntermediateDataViewResult, error)
	// Get reads an already-rendered view, without rendering. It is how a client
	// recovers a view it has the key for — after a reload, say — and how it
	// checks for one before offering to render.
	Get(ctx context.Context, jobID id.JobID, fileID, key string) (*IntermediateDataViewResult, error)
}
