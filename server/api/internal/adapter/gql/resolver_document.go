package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
)

func (r *queryResolver) LatestProjectSnapshot(ctx context.Context, projectId gqlmodel.ID) (*gqlmodel.ProjectDocument, error) {
	doc, err := interactor.GetLatest(ctx, string(projectId))
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

func (r *queryResolver) ProjectHistory(ctx context.Context, projectId gqlmodel.ID) ([]*gqlmodel.ProjectSnapshot, error) {
	history, err := interactor.GetHistory(ctx, string(projectId))
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.ProjectSnapshot, len(history))
	for i, h := range history {
		nodes[i] = &gqlmodel.ProjectSnapshot{
			Updates:   h.Updates,
			Version:   h.Version,
			Timestamp: h.Timestamp,
		}
	}

	return nodes, nil
}

func (r *mutationResolver) RollbackProject(ctx context.Context, projectId gqlmodel.ID, version int) (*gqlmodel.ProjectDocument, error) {
	doc, err := interactor.Rollback(ctx, string(projectId), version)
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

type projectDocumentResolver struct{ *Resolver }

func (r *Resolver) ProjectDocument() ProjectDocumentResolver {
	return &projectDocumentResolver{r}
}

func (r *projectDocumentResolver) Updates(ctx context.Context, obj *gqlmodel.ProjectDocument) ([]int, error) {
	return obj.Updates, nil
}
