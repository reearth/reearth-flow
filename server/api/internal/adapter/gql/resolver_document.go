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
	history, err := interactor.GetHistoryByVersion(ctx, string(projectId), version)
	if err != nil {
		return nil, err
	}

	snapshots := make([]*gqlmodel.ProjectSnapshot, len(history))
	for i, h := range history {
		snapshots[i] = &gqlmodel.ProjectSnapshot{
			Updates:   h.Updates,
			Version:   h.Version,
			Timestamp: h.Timestamp,
		}
	}

	return snapshots, nil
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

func (r *mutationResolver) FlushProjectToGcs(ctx context.Context, projectId gqlmodel.ID) (*bool, error) {
	err := interactor.FlushToGCS(ctx, string(projectId))
	if err != nil {
		return nil, err
	}
	result := true
	return &result, nil
}

type projectDocumentResolver struct{ *Resolver }

func (r *Resolver) ProjectDocument() ProjectDocumentResolver {
	return &projectDocumentResolver{r}
}

func (r *projectDocumentResolver) Updates(ctx context.Context, obj *gqlmodel.ProjectDocument) ([]int, error) {
	return obj.Updates, nil
}
