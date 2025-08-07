package user

import (
	"context"

	"github.com/hasura/go-graphql-client"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/util"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/samber/lo"
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

	return user.New().
		ID(string(q.Me.ID)).
		Name(string(q.Me.Name)).
		Alias(string(q.Me.Alias)).
		Email(string(q.Me.Email)).
		Metadata(util.ToUserMetadata(q.Me.Metadata)).
		Host(lo.ToPtr(string(q.Me.Host))).
		MyWorkspaceID(string(q.Me.MyWorkspaceID)).
		Auths(util.ToStringSlice(q.Me.Auths)).
		Workspaces(util.ToWorkspaces(q.Me.Workspaces)).
		Build()
}
