package interactor

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
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

// authorize checks action against the project's owning workspace. Fails closed.
//
// ResourceProject, not ResourceProjectDocument: Cerbos denies actions on resources
// absent from its policy store, and projectDocument is not published there.
func (i *Websocket) authorize(ctx context.Context, projectID string, action string) error {
	_, err := i.authorizeProject(ctx, projectID, action)
	return err
}

// authorizeWorkflowChange is authorize for a call that replaces the project's
// workflow, which a locked project refuses. The lock is checked after the
// permission, so a caller without access cannot learn whether it is locked.
func (i *Websocket) authorizeWorkflowChange(ctx context.Context, projectID string) error {
	proj, err := i.authorizeProject(ctx, projectID, rbac.ActionEdit)
	if err != nil {
		return err
	}
	if proj.IsLocked() {
		return interfaces.ErrProjectLocked
	}
	return nil
}

func (i *Websocket) authorizeProject(ctx context.Context, projectID string, action string) (*project.Project, error) {
	pid, err := id.ProjectIDFrom(projectID)
	if err != nil {
		return nil, rerror.ErrNotFound
	}
	proj, err := i.projectRepo.FindByID(ctx, pid)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceProject, action, proj.Workspace()); err != nil {
		return nil, err
	}
	return proj, nil
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

// GetSnapshotState reads one snapshot's state. ActionRead: read-only, so
// restore's write is gated by the collaborative document path, not by this.
func (i *Websocket) GetSnapshotState(ctx context.Context, docID string, snapshotNumber int) (*ws.SnapshotState, error) {
	if err := i.authorize(ctx, docID, rbac.ActionRead); err != nil {
		return nil, err
	}
	return i.client.GetSnapshotState(ctx, docID, snapshotNumber)
}

// Rollback prunes every update above the target clock.
func (i *Websocket) Rollback(ctx context.Context, docID string, version int) (*ws.Document, error) {
	if err := i.authorizeWorkflowChange(ctx, docID); err != nil {
		return nil, err
	}
	return i.client.Rollback(ctx, docID, version)
}

// FlushToGCS backs the editor's save action. ActionAny, not ActionEdit, which
// is maintainer/owner and would stop writers saving: ActionAny on
// ResourceProject is writer/maintainer/owner, deliberately excluding reader.
func (i *Websocket) FlushToGCS(ctx context.Context, docID string) error {
	if err := i.authorize(ctx, docID, rbac.ActionAny); err != nil {
		return err
	}
	return i.client.FlushToGCS(ctx, docID)
}

// CreateSnapshot backs previewSnapshot; it materializes state without writing.
func (i *Websocket) CreateSnapshot(ctx context.Context, docID string, version int, name string) (*ws.Document, error) {
	if err := i.authorize(ctx, docID, rbac.ActionRead); err != nil {
		return nil, err
	}
	return i.client.CreateSnapshot(ctx, docID, version, name)
}

// CopyDocument needs write on the destination and read access to the source.
func (i *Websocket) CopyDocument(ctx context.Context, docID string, source string) error {
	if err := i.authorizeWorkflowChange(ctx, docID); err != nil {
		return err
	}
	if err := i.authorize(ctx, source, rbac.ActionRead); err != nil {
		return err
	}
	return i.client.CopyDocument(ctx, docID, source)
}

func (i *Websocket) ImportDocument(ctx context.Context, docID string, data []byte) error {
	if err := i.authorizeWorkflowChange(ctx, docID); err != nil {
		return err
	}
	return i.client.ImportDocument(ctx, docID, data)
}

// SaveVersion is gated like FlushToGCS: it backs the same save action, and
// every role that can edit the workflow may save it.
func (i *Websocket) SaveVersion(ctx context.Context, docID, label string) (bool, error) {
	if err := i.authorize(ctx, docID, rbac.ActionAny); err != nil {
		return false, err
	}
	return i.client.SaveVersion(ctx, docID, label)
}

// SetProjectLock is called by UpdateProject, which already checked edit
// permission; it is gated here like the other writes in case of other callers.
// Not authorizeWorkflowChange: unlocking a locked project must get through.
func (i *Websocket) SetProjectLock(ctx context.Context, docID string, locked bool, changedAt time.Time) error {
	if err := i.authorize(ctx, docID, rbac.ActionEdit); err != nil {
		return err
	}
	return i.client.SetProjectLock(ctx, docID, locked, changedAt)
}

// DeleteDocument is not refused for a locked project: deleting the project is
// its lifecycle, not an edit of its workflow, and needs ActionDelete anyway.
func (i *Websocket) DeleteDocument(ctx context.Context, docID string) error {
	if err := i.authorize(ctx, docID, rbac.ActionDelete); err != nil {
		return err
	}
	return i.client.DeleteDocument(ctx, docID)
}

func (i *Websocket) Close() error {
	return i.client.Close()
}
