package gql

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
	"github.com/samber/lo"
)

func (r *mutationResolver) CreateAsset(ctx context.Context, input gqlmodel.CreateAssetInput) (*gqlmodel.CreateAssetPayload, error) {
	wid, err := gqlmodel.ToID[id.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Asset.Create(ctx, interfaces.CreateAssetParam{
		WorkspaceID: wid,
		File:        gqlmodel.FromFile(&input.File),
		Name:        input.Name,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.CreateAssetPayload{Asset: gqlmodel.ToAsset(res)}, nil
}

func (r *mutationResolver) CreateAssetFromUpload(ctx context.Context, input gqlmodel.CreateAssetFromUploadInput) (*gqlmodel.CreateAssetPayload, error) {
	wid, err := gqlmodel.ToID[id.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	params := interfaces.CreateAssetFromUploadParam{
		WorkspaceID: wid,
		File:        gqlmodel.FromFile(input.File),
		Name:        input.Name,
	}
	if input.URL != nil {
		params.File, err = file.FromURL(ctx, *input.URL)
		if err != nil {
			return nil, err
		}
	}
	params.Token = lo.FromPtr(input.Token)

	res, err := usecases(ctx).Asset.CreateFromUpload(ctx, params)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.CreateAssetPayload{Asset: gqlmodel.ToAsset(res)}, nil
}

func (r *mutationResolver) UpdateAsset(ctx context.Context, input gqlmodel.UpdateAssetInput) (*gqlmodel.UpdateAssetPayload, error) {
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

func (r *mutationResolver) CreateAssetUpload(ctx context.Context, input gqlmodel.CreateAssetUploadInput) (*gqlmodel.CreateAssetUploadPayload, error) {
	wid, err := gqlmodel.ToID[id.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}
	au, err := usecases(ctx).Asset.CreateUpload(ctx, interfaces.CreateAssetUploadParam{
		WorkspaceID:     wid,
		Filename:        lo.FromPtr(input.Filename),
		ContentLength:   int64(lo.FromPtr(input.ContentLength)),
		ContentEncoding: lo.FromPtr(input.ContentEncoding),
		Cursor:          lo.FromPtr(input.Cursor),
	})
	if err != nil && errors.Is(err, rerror.ErrNotFound) {
		return &gqlmodel.CreateAssetUploadPayload{}, nil
	}
	if err != nil {
		return nil, err
	}

	return &gqlmodel.CreateAssetUploadPayload{
		URL:             au.URL,
		Token:           au.UUID,
		ContentType:     lo.EmptyableToPtr(au.ContentType),
		ContentLength:   int(au.ContentLength),
		ContentEncoding: lo.EmptyableToPtr(au.ContentEncoding),
		Next:            lo.EmptyableToPtr(au.Next),
	}, nil
}
