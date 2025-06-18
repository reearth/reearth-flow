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

func (r *queryResolver) ProjectSnapshot(ctx context.Context, projectId gqlmodel.ID, version int) (*gqlmodel.ProjectSnapshot, error) {
	history, err := interactor.GetHistoryByVersion(ctx, string(projectId), version)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.ProjectSnapshot{
		Updates:   history.Updates,
		Version:   history.Version,
		Timestamp: history.Timestamp,
	}, nil
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

func (r *mutationResolver) SaveSnapshot(ctx context.Context, projectId gqlmodel.ID) (bool, error) {
	err := interactor.FlushToGCS(ctx, string(projectId))
	if err != nil {
		return false, err
	}
	return true, nil
}

func (r *mutationResolver) PreviewSnapshot(ctx context.Context, projectID gqlmodel.ID, version int, name *string) (*gqlmodel.PreviewSnapshot, error) {
	var snapshotName string
	if name != nil {
		snapshotName = *name
	}

	history, err := interactor.CreateSnapshot(ctx, string(projectID), version, snapshotName)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.PreviewSnapshot{
		ID:        projectID,
		Updates:   history.Updates,
		Version:   history.Version,
		Timestamp: history.Timestamp,
		Name:      name,
	}, nil
}

type projectDocumentResolver struct{ *Resolver }

func (r *Resolver) ProjectDocument() ProjectDocumentResolver {
	return &projectDocumentResolver{r}
}

func (r *projectDocumentResolver) Updates(ctx context.Context, obj *gqlmodel.ProjectDocument) ([]int, error) {
	return obj.Updates, nil
}
