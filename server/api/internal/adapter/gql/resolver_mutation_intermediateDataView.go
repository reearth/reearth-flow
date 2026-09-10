package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *mutationResolver) RenderIntermediateDataView(
	ctx context.Context,
	input gqlmodel.RenderIntermediateDataViewInput,
) (*gqlmodel.RenderIntermediateDataViewPayload, error) {
	jid, err := id.JobIDFrom(string(input.JobID))
	if err != nil {
		return nil, err
	}

	shape, err := gqlmodel.FromIntermediateDataViewShape(input.Shape)
	if err != nil {
		return nil, err
	}

	options, err := gqlmodel.FromIntermediateDataViewOptions(input.Options)
	if err != nil {
		return nil, err
	}

	view, err := usecases(ctx).IntermediateDataView.Render(ctx, interfaces.RenderIntermediateDataViewParam{
		JobID:  jid,
		FileID: input.FileID,
		Request: featureview.Request{
			Shape: shape,
			Selection: featureview.Selection{
				Row:    input.Row,
				Filter: input.Filter,
			},
			Options: options,
		},
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.RenderIntermediateDataViewPayload{
		View: gqlmodel.ToIntermediateDataView(view),
	}, nil
}
