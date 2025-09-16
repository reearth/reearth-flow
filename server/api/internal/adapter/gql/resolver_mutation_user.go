package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *mutationResolver) Signup(ctx context.Context, input gqlmodel.SignupInput) (*gqlmodel.SignupPayload, error) {
	u, err := usecases(ctx).User.SignupOIDC(ctx, interfaces.SignupOIDCParam{
		UserID:      gqlmodel.ToIDRef[id.User](input.UserID),
		Lang:        input.Lang,
		WorkspaceID: gqlmodel.ToIDRef[id.Workspace](input.WorkspaceID),
		Secret:      input.Secret,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.SignupPayload{User: gqlmodel.ToUser(u)}, nil
}

func (r *mutationResolver) UpdateMe(ctx context.Context, input gqlmodel.UpdateMeInput) (*gqlmodel.UpdateMePayload, error) {
	res, err := usecases(ctx).User.UpdateMe(ctx, interfaces.UpdateMeParam{
		Name:                 input.Name,
		Email:                input.Email,
		Lang:                 input.Lang,
		Password:             input.Password,
		PasswordConfirmation: input.PasswordConfirmation,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateMePayload{Me: gqlmodel.ToMe(res)}, nil
}

func (r *mutationResolver) RemoveMyAuth(ctx context.Context, input gqlmodel.RemoveMyAuthInput) (*gqlmodel.UpdateMePayload, error) {
	res, err := usecases(ctx).User.RemoveMyAuth(ctx, input.Auth)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateMePayload{Me: gqlmodel.ToMe(res)}, nil
}

func (r *mutationResolver) DeleteMe(ctx context.Context, input gqlmodel.DeleteMeInput) (*gqlmodel.DeleteMePayload, error) {
	uid, err := gqlmodel.ToID[id.User](input.UserID)
	if err != nil {
		return nil, err
	}

	if err := usecases(ctx).User.DeleteMe(ctx, uid); err != nil {
		return nil, err
	}

	return &gqlmodel.DeleteMePayload{UserID: input.UserID}, nil
}
