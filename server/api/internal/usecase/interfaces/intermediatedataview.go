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
	// ErrRenderTimedOut is a render whose worker never answered within the
	// request's wait. A render that is only slow does not end here: the worker
	// stops it at its own, shorter limit and reports it as a failed view. When
	// the request gives up, the worker stops the render too, so nothing is left
	// running, and asking again renders again.
	ErrRenderTimedOut = errors.New("the view did not finish rendering in time")
	// ErrIntermediateDataNotFound is a view asked of an output port that left
	// no intermediate data — a node that never ran, or a run with the feature
	// writer disabled.
	ErrIntermediateDataNotFound = errors.New("intermediate data not found")
)

// RenderIntermediateDataViewParam identifies the intermediate data to render
// and how.
// Fields are ordered for struct packing (govet fieldalignment), not by
// meaning.
type RenderIntermediateDataViewParam struct {
	// FileID is the engine's `[subgraphPrefix.]nodeID.port`, the same id the
	// editor builds to fetch the table.
	FileID  string
	Request featureview.Request
	// JobID is the finished job whose feature-store holds the input.
	JobID id.JobID
}

// IntermediateDataViewResult is a view's current state.
//
// Format and EntryPointURL are empty until a render has finished, because the
// engine chooses the output format from the geometry it finds: a tiles view
// becomes 3D Tiles or vector tiles, and the server cannot know which before the
// render runs.
// Fields are ordered for struct packing (govet fieldalignment), not by
// meaning.
type IntermediateDataViewResult struct {
	Format *featureview.Format
	// SelectedFeatures and RenderedFeatures are set for any terminal status,
	// including EMPTY: "0 of 900 selected" is the explanation, so withholding
	// the counts exactly when the view is empty would withhold the reason. They
	// are nil only when the render's report could not be read.
	SelectedFeatures *int
	RenderedFeatures *int
	Error            *string
	// Key identifies the view by its content: the same request always yields
	// the same key, so a repeat request reuses a rendered view.
	Key           string
	FileID        string
	Shape         featureview.Shape
	Status        featureview.Status
	EntryPointURL string
	JobID         id.JobID
}

type IntermediateDataView interface {
	// Render returns an existing view, or renders one and waits for it. Either
	// way the result is terminal: there is no in-flight state to poll.
	Render(context.Context, RenderIntermediateDataViewParam) (*IntermediateDataViewResult, error)
	// Get reads an already-rendered view, without rendering. It is how a client
	// recovers a view it has the key for — after a reload, say — and how it
	// checks for one before offering to render.
	//
	// A nil result with a nil error means no view exists for the key. An unknown
	// job or a malformed key is an error.
	Get(ctx context.Context, jobID id.JobID, fileID, key string) (*IntermediateDataViewResult, error)
}
