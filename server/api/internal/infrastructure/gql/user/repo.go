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
