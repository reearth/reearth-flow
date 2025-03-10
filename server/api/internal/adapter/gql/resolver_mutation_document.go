package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *mutationResolver) RollbackProject(ctx context.Context, projectId gqlmodel.ID, version int) (*gqlmodel.ProjectDocument, error) {
	doc, err := usecases(ctx).Document.Rollback(ctx, string(projectId), version)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.ProjectDocument{
		ID:        projectId,
		Updates:   doc.Updates,
		Version:   doc.Version,
		Timestamp: doc.Timestamp,
	}, nil
}
