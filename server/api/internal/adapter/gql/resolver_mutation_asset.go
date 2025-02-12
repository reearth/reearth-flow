package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

func (r *mutationResolver) CreateAsset(ctx context.Context, input gqlmodel.CreateAssetInput) (*gqlmodel.CreateAssetPayload, error) {
	tid, err := gqlmodel.ToID[accountdomain.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Asset.Create(ctx, interfaces.CreateAssetParam{
		WorkspaceID: tid,
		File:        gqlmodel.FromFile(&input.File),
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.CreateAssetPayload{Asset: gqlmodel.ToAsset(res)}, nil
}

func (r *mutationResolver) RemoveAsset(ctx context.Context, input gqlmodel.RemoveAssetInput) (*gqlmodel.RemoveAssetPayload, error) {
	aid, err := gqlmodel.ToID[id.Asset](input.AssetID)
	if err != nil {
		return nil, err
	}

	res, err2 := usecases(ctx).Asset.Remove(ctx, aid)
	if err2 != nil {
		return nil, err2
	}

	return &gqlmodel.RemoveAssetPayload{AssetID: gqlmodel.IDFrom(res)}, nil
}
