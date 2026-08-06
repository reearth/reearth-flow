package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	ws "github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearthx/rerror"
)

type Websocket struct {
	client            interfaces.WebsocketClient
	projectRepo       repo.Project
	permissionChecker gateway.PermissionChecker
}

func NewWebsocket(client interfaces.WebsocketClient, projectRepo repo.Project, permissionChecker gateway.PermissionChecker) *Websocket {
	return &Websocket{
		client:            client,
		projectRepo:       projectRepo,
		permissionChecker: permissionChecker,
	}
}

func (i *Websocket) authorize(ctx context.Context, docID string, action string) error {
	pid, err := id.ProjectIDFrom(docID)
	if err != nil {
		return rerror.ErrNotFound
	}
	proj, err := i.projectRepo.FindByID(ctx, pid)
	if err != nil {
		return err
	}
	if proj == nil {
		return rerror.ErrNotFound
	}
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceProjectDocument, action, proj.Workspace())
}

func (i *Websocket) GetLatest(ctx context.Context, docID string) (*ws.Document, error) {
	if err := i.authorize(ctx, docID, rbac.ActionRead); err != nil {
		return nil, err
	}
	return i.client.GetLatest(ctx, docID)
}

func (i *Websocket) GetHistory(ctx context.Context, docID string) ([]*ws.History, error) {
	if err := i.authorize(ctx, docID, rbac.ActionRead); err != nil {
		return nil, err
	}
	return i.client.GetHistory(ctx, docID)
}

func (i *Websocket) GetHistoryByVersion(ctx context.Context, docID string, version int) (*ws.History, error) {
	if err := i.authorize(ctx, docID, rbac.ActionRead); err != nil {
		return nil, err
	}
	return i.client.GetHistoryByVersion(ctx, docID, version)
}

func (i *Websocket) GetHistoryMetadata(ctx context.Context, docID string) ([]*ws.HistoryMetadata, error) {
	if err := i.authorize(ctx, docID, rbac.ActionRead); err != nil {
		return nil, err
	}
	return i.client.GetHistoryMetadata(ctx, docID)
}

func (i *Websocket) GetNamedSnapshots(ctx context.Context, docID string) ([]*ws.SnapshotMetadata, error) {
	if err := i.authorize(ctx, docID, rbac.ActionRead); err != nil {
		return nil, err
	}
	return i.client.GetNamedSnapshots(ctx, docID)
}

func (i *Websocket) SaveNamedSnapshot(ctx context.Context, docID, label string) (*ws.SnapshotMetadata, error) {
	if err := i.authorize(ctx, docID, rbac.ActionEdit); err != nil {
		return nil, err
	}
	return i.client.SaveNamedSnapshot(ctx, docID, label)
}

// Rollback prunes every update above the target clock.
func (i *Websocket) Rollback(ctx context.Context, docID string, version int) (*ws.Document, error) {
	if err := i.authorize(ctx, docID, rbac.ActionEdit); err != nil {
		return nil, err
	}
	return i.client.Rollback(ctx, docID, version)
}

func (i *Websocket) FlushToGCS(ctx context.Context, docID string) error {
	if err := i.authorize(ctx, docID, rbac.ActionEdit); err != nil {
		return err
	}
	return i.client.FlushToGCS(ctx, docID)
}

// CreateSnapshot backs previewSnapshot and does not write, so ActionRead is correct.
func (i *Websocket) CreateSnapshot(ctx context.Context, docID string, version int, name string) (*ws.Document, error) {
	if err := i.authorize(ctx, docID, rbac.ActionRead); err != nil {
		return nil, err
	}
	return i.client.CreateSnapshot(ctx, docID, version, name)
}

// CopyDocument needs edit on the destination and read on the source.
func (i *Websocket) CopyDocument(ctx context.Context, docID string, source string) error {
	if err := i.authorize(ctx, docID, rbac.ActionEdit); err != nil {
		return err
	}
	if err := i.authorize(ctx, source, rbac.ActionRead); err != nil {
		return err
	}
	return i.client.CopyDocument(ctx, docID, source)
}

func (i *Websocket) ImportDocument(ctx context.Context, docID string, data []byte) error {
	if err := i.authorize(ctx, docID, rbac.ActionEdit); err != nil {
		return err
	}
	return i.client.ImportDocument(ctx, docID, data)
}

func (i *Websocket) DeleteDocument(ctx context.Context, docID string) error {
	if err := i.authorize(ctx, docID, rbac.ActionDelete); err != nil {
		return err
	}
	return i.client.DeleteDocument(ctx, docID)
}

func (i *Websocket) Close() error {
	return i.client.Close()
}
