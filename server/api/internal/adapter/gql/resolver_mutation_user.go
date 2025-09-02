package gql

import (
	"context"
	"log"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountinterfaces"
)

func (r *mutationResolver) Signup(ctx context.Context, input gqlmodel.SignupInput) (*gqlmodel.SignupPayload, error) {
	// TODO: After migration, remove this logic and use the new usecase directly.
	if usecases(ctx).TempNewUser != nil {
		flowUser, flowWorkspace, err := usecases(ctx).TempNewUser.SignupOIDC(ctx, interfaces.SignupOIDCParam{
			UserID:      gqlmodel.ToIDRef[id.User](input.UserID),
			Lang:        input.Lang,
			WorkspaceID: gqlmodel.ToIDRef[id.Workspace](input.WorkspaceID),
			Secret:      input.Secret,
		})
		if err != nil {
			log.Printf("WARNING:[mutationResolver.signupWithTempNewUsecase] Failed to sign up user: %v", err)
		} else {
			log.Printf("DEBUG:[mutationResolver.signupWithTempNewUsecase] Signed up user with tempNewUsecase")
			return &gqlmodel.SignupPayload{
				User:      gqlmodel.ToUserFromFlow(flowUser),
				Workspace: gqlmodel.ToWorkspaceFromFlow(flowWorkspace),
			}, nil
		}
	}
	log.Printf("WARNING:[mutationResolver.Signup] Fallback to traditional usecase")

	au := adapter.GetAuthInfo(ctx)
	if au == nil {
		return nil, interfaces.ErrOperationDenied
	}

	u, err := usecases(ctx).User.SignupOIDC(ctx, accountinterfaces.SignupOIDCParam{
		Sub:         au.Sub,
		AccessToken: au.Token,
		Issuer:      au.Iss,
		Email:       au.Email,
		Name:        au.Name,
		Secret:      input.Secret,
		User: accountinterfaces.SignupUserParam{
			Lang:        input.Lang,
			UserID:      gqlmodel.ToIDRef[accountdomain.User](input.UserID),
			WorkspaceID: gqlmodel.ToIDRef[accountdomain.Workspace](input.WorkspaceID),
		},
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.SignupPayload{User: gqlmodel.ToUser(u)}, nil
}

// TODO: After migration, remove this logic and use the new usecase directly.
func (r *mutationResolver) UpdateMe(ctx context.Context, input gqlmodel.UpdateMeInput) (*gqlmodel.UpdateMePayload, error) {
	if usecases(ctx).TempNewUser != nil {
		tempRes, err := usecases(ctx).TempNewUser.UpdateMe(ctx, interfaces.UpdateMeParam{
			Name:                 input.Name,
			Email:                input.Email,
			Lang:                 input.Lang,
			Password:             input.Password,
			PasswordConfirmation: input.PasswordConfirmation,
		})
		if err != nil {
			log.Printf("WARNING:[mutationResolver.updateMeWithTempNewUsecase] Failed to update user: %v", err)
		} else {
			log.Printf("DEBUG:[mutationResolver.updateMeWithTempNewUsecase] Updated user with tempNewUsecase")
			return &gqlmodel.UpdateMePayload{Me: gqlmodel.ToMeFromFlow(tempRes)}, nil
		}
	}
	log.Printf("WARNING:[mutationResolver.UpdateMe] Fallback to traditional usecase")

	res, err := usecases(ctx).User.UpdateMe(ctx, accountinterfaces.UpdateMeParam{
		Name:                 input.Name,
		Email:                input.Email,
		Lang:                 input.Lang,
		Password:             input.Password,
		PasswordConfirmation: input.PasswordConfirmation,
	}, getAcOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateMePayload{Me: gqlmodel.ToMe(res)}, nil
}

func (r *mutationResolver) RemoveMyAuth(ctx context.Context, input gqlmodel.RemoveMyAuthInput) (*gqlmodel.UpdateMePayload, error) {
	res, err := usecases(ctx).User.RemoveMyAuth(ctx, input.Auth, getAcOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateMePayload{Me: gqlmodel.ToMe(res)}, nil
}

func (r *mutationResolver) DeleteMe(ctx context.Context, input gqlmodel.DeleteMeInput) (*gqlmodel.DeleteMePayload, error) {
	uid, err := gqlmodel.ToID[accountdomain.User](input.UserID)
	if err != nil {
		return nil, err
	}

	if err := usecases(ctx).User.DeleteMe(ctx, uid, getAcOperator(ctx)); err != nil {
		return nil, err
	}

	return &gqlmodel.DeleteMePayload{UserID: input.UserID}, nil
}
