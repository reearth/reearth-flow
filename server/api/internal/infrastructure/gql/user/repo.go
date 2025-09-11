package user

import (
	"context"

	"github.com/hasura/go-graphql-client"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/util"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/user"
)

type userRepo struct {
	client *graphql.Client
}

func NewRepo(gql *graphql.Client) user.Repo {
	return &userRepo{client: gql}
}

func (r *userRepo) FindMe(ctx context.Context) (*user.User, error) {
	var q findMeQuery
	if err := r.client.Query(ctx, &q, nil); err != nil {
		return nil, err
	}

	return util.ToMe(q.Me)
}

func (r *userRepo) FindByIDs(ctx context.Context, ids id.UserIDList) (user.List, error) {
	if len(ids) == 0 {
		return nil, nil
	}

	graphqlIDs := make([]graphql.ID, 0, len(ids))
	for _, id := range ids {
		graphqlIDs = append(graphqlIDs, graphql.ID(id.String()))
	}

	var q findUsersByIDsQuery
	vars := map[string]interface{}{
		"ids": graphqlIDs,
	}
	if err := r.client.Query(ctx, &q, vars); err != nil {
		return nil, err
	}

	return util.ToUsers(q.Users)
}

func (r *userRepo) UserByNameOrEmail(ctx context.Context, nameOrEmail string) (*user.User, error) {
	if nameOrEmail == "" {
		return nil, nil
	}

	var q userByNameOrEmailQuery
	vars := map[string]interface{}{
		"nameOrEmail": graphql.String(nameOrEmail),
	}
	if err := r.client.Query(ctx, &q, vars); err != nil {
		return nil, err
	}

	return util.ToUserFromSimple(q.User)
}

func (r *userRepo) UpdateMe(ctx context.Context, a user.UpdateAttrs) (*user.User, error) {
	in := UpdateMeInput{}
	if a.Name != nil {
		s := graphql.String(*a.Name)
		in.Name = &s
	}
	if a.Email != nil {
		s := graphql.String(*a.Email)
		in.Email = &s
	}
	if a.Lang != nil {
		langCode := graphql.String(a.Lang.String())
		in.Lang = &langCode
	}
	if a.Password != nil && a.PasswordConfirmation != nil {
		p := graphql.String(*a.Password)
		pc := graphql.String(*a.PasswordConfirmation)
		in.Password = &p
		in.PasswordConfirmation = &pc
	}

	var m updateMeMutation
	vars := map[string]interface{}{
		"input": in,
	}
	if err := r.client.Mutate(ctx, &m, vars); err != nil {
		return nil, err
	}

	return util.ToMe(m.UpdateMe.Me)
}

func (r *userRepo) SignupOIDC(ctx context.Context, a user.SignupOIDCAttrs) (*user.User, error) {
	in := SignupOIDCInput{}
	if a.UserID != nil {
		s := graphql.ID(a.UserID.String())
		in.ID = &s
	}
	if a.Lang != nil {
		langCode := graphql.String(a.Lang.String())
		in.Lang = &langCode
	}
	if a.WorkspaceID != nil {
		s := graphql.ID(a.WorkspaceID.String())
		in.WorkspaceID = &s
	}
	if a.Secret != nil {
		s := graphql.String(*a.Secret)
		in.Secret = &s
	}

	var m signupOIDCMutation
	vars := map[string]interface{}{
		"input": in,
	}
	if err := r.client.Mutate(ctx, &m, vars); err != nil {
		return nil, err
	}

	return util.ToUser(m.SignupOIDC.User)
}
