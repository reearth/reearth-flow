package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *queryResolver) LatestProjectSnapshot(ctx context.Context, projectId gqlmodel.ID) (*gqlmodel.ProjectDocument, error) {
	doc, err := usecases(ctx).Websocket.GetLatest(ctx, string(projectId))
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
	history, err := usecases(ctx).Websocket.GetHistoryByVersion(ctx, string(projectId), version)
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
	metadata, err := usecases(ctx).Websocket.GetHistoryMetadata(ctx, string(projectId))
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
	doc, err := usecases(ctx).Websocket.Rollback(ctx, string(projectId), version)
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
	err := usecases(ctx).Websocket.FlushToGCS(ctx, string(projectId))
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

	history, err := usecases(ctx).Websocket.CreateSnapshot(ctx, string(projectID), version, snapshotName)
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

func (r *mutationResolver) CopyProject(ctx context.Context, projectId gqlmodel.ID, source gqlmodel.ID) (bool, error) {
	err := usecases(ctx).Websocket.CopyDocument(ctx, string(projectId), string(source))
	if err != nil {
		return false, err
	}
	return true, nil
}

func (r *mutationResolver) ImportProject(ctx context.Context, projectId gqlmodel.ID, data gqlmodel.Bytes) (bool, error) {
	err := usecases(ctx).Websocket.ImportDocument(ctx, string(projectId), []byte(data))
	if err != nil {
		return false, err
	}
	return true, nil
}

type projectDocumentResolver struct{ *Resolver }

func (r *projectDocumentResolver) Updates(ctx context.Context, obj *gqlmodel.ProjectDocument) ([]int, error) {
	return obj.Updates, nil
}
