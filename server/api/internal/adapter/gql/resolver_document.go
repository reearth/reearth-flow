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

func (r *queryResolver) ProjectSnapshot(ctx context.Context, projectId gqlmodel.ID, version int) ([]*gqlmodel.ProjectSnapshot, error) {
	history, err := interactor.GetHistory(ctx, string(projectId))
	if err != nil {
		return nil, err
	}

	var filteredHistory []*gqlmodel.ProjectSnapshot
	for _, h := range history {
		if h.Version == version {
			filteredHistory = append(filteredHistory, &gqlmodel.ProjectSnapshot{
				Updates:   h.Updates,
				Version:   h.Version,
				Timestamp: h.Timestamp,
			})
			break
		}
	}

	return filteredHistory, nil
}

func (r *queryResolver) ProjectHistory(ctx context.Context, projectId gqlmodel.ID) ([]*gqlmodel.ProjectSnapshotMetadata, error) {
	metadata, err := interactor.GetHistoryMetadata(ctx, string(projectId))
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.ProjectSnapshotMetadata, len(metadata))
	for i, m := range metadata {
		nodes[i] = &gqlmodel.ProjectSnapshotMetadata{
			Version:   m.Version,
			Timestamp: m.Timestamp,
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
