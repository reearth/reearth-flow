package workspace

import (
	"context"

	"github.com/hasura/go-graphql-client"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/util"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
)

type workspaceRepo struct {
	client *graphql.Client
}

func NewRepo(gql *graphql.Client) workspace.Repo {
	return &workspaceRepo{client: gql}
}

func (r *workspaceRepo) FindByIDs(ctx context.Context, ids id.WorkspaceIDList) (workspace.List, error) {
	if len(ids) == 0 {
		return nil, nil
	}

	graphqlIDs := make([]graphql.ID, 0, len(ids))
	for _, id := range ids {
		graphqlIDs = append(graphqlIDs, graphql.ID(id.String()))
	}

	var q findByIDsQuery
	vars := map[string]interface{}{
		"ids": graphqlIDs,
	}
	if err := r.client.Query(ctx, &q, vars); err != nil {
		return nil, err
	}

	return util.ToWorkspaces(q.Workspaces)
}

func (r *workspaceRepo) FindByUser(ctx context.Context, uid id.UserID) (workspace.List, error) {
	var q findByUserQuery
	vars := map[string]interface{}{
		"userId": graphql.ID(uid.String()),
	}
	if err := r.client.Query(ctx, &q, vars); err != nil {
		return nil, err
	}

	return util.ToWorkspaces(q.Workspaces)
}
