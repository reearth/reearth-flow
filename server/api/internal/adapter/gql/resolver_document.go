package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
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
		Version:   &history.Version,
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

// ProjectNamedSnapshot reads one snapshot's state by its per-room snapshot number.
func (r *queryResolver) ProjectNamedSnapshot(ctx context.Context, projectId gqlmodel.ID, snapshotNumber int) (*gqlmodel.ProjectSnapshot, error) {
	state, err := usecases(ctx).Websocket.GetSnapshotState(ctx, string(projectId), snapshotNumber)
	if err != nil {
		return nil, err
	}
	if state == nil {
		return nil, interfaces.ErrSnapshotNotFound
	}

	// Version deliberately unset: a snapshot carries no update-log clock, and a
	// fabricated one passed to rollbackProject would prune real history.
	num := int(state.SnapshotID)
	return &gqlmodel.ProjectSnapshot{
		SnapshotNumber: &num,
		Updates:        state.Updates,
	}, nil
}

func (r *queryResolver) ProjectNamedSnapshots(ctx context.Context, projectId gqlmodel.ID) ([]*gqlmodel.NamedSnapshot, error) {
	snaps, err := usecases(ctx).Websocket.GetNamedSnapshots(ctx, string(projectId))
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.NamedSnapshot, len(snaps))
	for i, s := range snaps {
		nodes[i] = &gqlmodel.NamedSnapshot{
			SnapshotNumber: int(s.ID), // per-room counter; 32 bits is ample
			Label:          s.Label,
			Timestamp:      s.Timestamp,
			Size:           s.Size,
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

func (r *mutationResolver) SaveNamedSnapshot(ctx context.Context, projectId gqlmodel.ID, label string) (*gqlmodel.NamedSnapshot, error) {
	s, err := usecases(ctx).Websocket.SaveNamedSnapshot(ctx, string(projectId), label)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.NamedSnapshot{
		SnapshotNumber: int(s.ID),
		Label:          s.Label,
		Timestamp:      s.Timestamp,
		Size:           s.Size,
	}, nil
}

type projectDocumentResolver struct{ *Resolver }

func (r *projectDocumentResolver) Updates(ctx context.Context, obj *gqlmodel.ProjectDocument) ([]int, error) {
	return obj.Updates, nil
}
