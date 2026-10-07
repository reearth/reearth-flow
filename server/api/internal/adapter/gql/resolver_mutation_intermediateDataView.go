package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// The two render mutations are the two shapes. Each input carries only the
// selection its shape accepts, so a mismatched pair — a GLTF view without a
// row, a tiles view with one — cannot be expressed at all.

func (r *mutationResolver) RenderIntermediateDataFeatureView(
	ctx context.Context,
	input gqlmodel.RenderIntermediateDataFeatureViewInput,
) (*gqlmodel.RenderIntermediateDataViewPayload, error) {
	row := input.Row
	return renderIntermediateDataView(ctx, input.JobID, input.FileID, featureview.ShapeGLTF,
		featureview.Selection{Row: &row})
}

func (r *mutationResolver) RenderIntermediateDataTilesView(
	ctx context.Context,
	input gqlmodel.RenderIntermediateDataTilesViewInput,
) (*gqlmodel.RenderIntermediateDataViewPayload, error) {
	return renderIntermediateDataView(ctx, input.JobID, input.FileID, featureview.ShapeTiles,
		featureview.Selection{Filter: input.Filter})
}

func renderIntermediateDataView(
	ctx context.Context,
	jobID gqlmodel.ID,
	fileID string,
	shape featureview.Shape,
	selection featureview.Selection,
) (*gqlmodel.RenderIntermediateDataViewPayload, error) {
	jid, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	view, err := usecases(ctx).IntermediateDataView.Render(ctx, interfaces.RenderIntermediateDataViewParam{
		JobID:  jid,
		FileID: fileID,
		Request: featureview.Request{
			Shape:     shape,
			Selection: selection,
			// Not caller-settable: see featureview.Options.
			Options: featureview.DefaultOptions(),
		},
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.RenderIntermediateDataViewPayload{
		View: gqlmodel.ToIntermediateDataView(view),
	}, nil
}
