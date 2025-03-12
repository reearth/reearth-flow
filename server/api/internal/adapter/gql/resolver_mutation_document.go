package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *mutationResolver) RollbackProject(ctx context.Context, projectId gqlmodel.ID, version int) (*gqlmodel.Document, error) {
	pid, err := id.ProjectIDFrom(string(projectId))
	if err != nil {
		return nil, err
	}

	doc, err := usecases(ctx).Document.Rollback(ctx, pid, version)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.Document{
		ID:        projectId,
		Updates:   doc.Updates(),
		Version:   doc.Version(),
		Timestamp: doc.Timestamp(),
	}, nil
}
