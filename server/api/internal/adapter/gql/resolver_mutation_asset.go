package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

func (r *mutationResolver) CreateAsset(ctx context.Context, input gqlmodel.CreateAssetInput) (*gqlmodel.CreateAssetPayload, error) {
	wid, err := gqlmodel.ToID[accountdomain.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	operator := getOperator(ctx)
	if operator == nil || operator.AcOperator == nil || operator.AcOperator.User == nil {
		return nil, interfaces.ErrOperationDenied
	}

	res, err := usecases(ctx).Asset.Create(ctx, interfaces.CreateAssetParam{
		WorkspaceID: wid,
		UserID:      *operator.AcOperator.User,
		File:        gqlmodel.FromFile(&input.File),
		Name:        input.Name,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.CreateAssetPayload{Asset: gqlmodel.ToAsset(res)}, nil
}

func (r *mutationResolver) UpdateAsset(ctx context.Context, input gqlmodel.UpdateAssetInput) (*gqlmodel.UpdateAssetPayload, error) {
	operator := getOperator(ctx)
	if operator == nil || operator.AcOperator == nil || operator.AcOperator.User == nil {
		return nil, interfaces.ErrOperationDenied
	}

	aid, err := gqlmodel.ToID[id.Asset](input.AssetID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Asset.Update(ctx, interfaces.UpdateAssetParam{
		AssetID: aid,
		Name:    input.Name,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateAssetPayload{Asset: gqlmodel.ToAsset(res)}, nil
}

func (r *mutationResolver) DeleteAsset(ctx context.Context, input gqlmodel.DeleteAssetInput) (*gqlmodel.DeleteAssetPayload, error) {
	operator := getOperator(ctx)
	if operator == nil || operator.AcOperator == nil || operator.AcOperator.User == nil {
		return nil, interfaces.ErrOperationDenied
	}

	aid, err := gqlmodel.ToID[id.Asset](input.AssetID)
	if err != nil {
		return nil, err
	}

	res, err2 := usecases(ctx).Asset.Delete(ctx, aid)
	if err2 != nil {
		return nil, err2
	}

	return &gqlmodel.DeleteAssetPayload{AssetID: gqlmodel.IDFrom(res)}, nil
}
