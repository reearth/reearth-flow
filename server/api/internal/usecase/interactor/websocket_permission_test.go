package interactor

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	ws "github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// countingWSClient records how many times the underlying client was reached.
// The count is the assertion that matters: a permission check that returns an
// error but still calls through has denied nothing.
type countingWSClient struct {
	calls int
}

func (c *countingWSClient) GetLatest(context.Context, string) (*ws.Document, error) {
	c.calls++
	return nil, nil
}
func (c *countingWSClient) GetHistory(context.Context, string) ([]*ws.History, error) {
	c.calls++
	return nil, nil
}
func (c *countingWSClient) GetHistoryByVersion(context.Context, string, int) (*ws.History, error) {
	c.calls++
	return nil, nil
}
func (c *countingWSClient) GetHistoryMetadata(context.Context, string) ([]*ws.HistoryMetadata, error) {
	c.calls++
	return nil, nil
}
func (c *countingWSClient) Rollback(context.Context, string, int) (*ws.Document, error) {
	c.calls++
	return nil, nil
}
func (c *countingWSClient) FlushToGCS(context.Context, string) error { c.calls++; return nil }
func (c *countingWSClient) CreateSnapshot(context.Context, string, int, string) (*ws.Document, error) {
	c.calls++
	return nil, nil
}
func (c *countingWSClient) CopyDocument(context.Context, string, string) error {
	c.calls++
	return nil
}
func (c *countingWSClient) ImportDocument(context.Context, string, []byte) error {
	c.calls++
	return nil
}
func (c *countingWSClient) DeleteDocument(context.Context, string) error { c.calls++; return nil }
func (c *countingWSClient) Close() error                                 { c.calls++; return nil }

var _ interfaces.WebsocketClient = (*countingWSClient)(nil)

// wsFixture builds a Websocket interactor over one saved project.
func wsFixture(t *testing.T, allow bool) (*Websocket, *countingWSClient, *recordingChecker, string, accountsid.WorkspaceID) {
	t.Helper()
	ctx := context.Background()
	projectRepo := memory.NewProject()
	wsID := project.NewWorkspaceID()
	prj := project.New().NewID().Workspace(wsID).MustBuild()
	require.NoError(t, projectRepo.Save(ctx, prj))

	client := &countingWSClient{}
	rc := &recordingChecker{allow: allow}
	return NewWebsocket(client, projectRepo, rc), client, rc, prj.ID().String(), accountsid.WorkspaceID(wsID)
}

// wsOps is every operation on the surface, with the action each must demand.
// Table-driven so a newly added method without a permission check shows up as a
// missing row rather than passing silently.
var wsOps = []struct {
	// call first: keeping the pointer-bearing fields adjacent shortens the range
	// the GC has to scan (govet fieldalignment).
	call   func(context.Context, *Websocket, string) error
	name   string
	action string
}{
	{name: "GetLatest", action: rbac.ActionRead, call: func(ctx context.Context, i *Websocket, d string) error {
		_, err := i.GetLatest(ctx, d)
		return err
	}},
	{name: "GetHistory", action: rbac.ActionRead, call: func(ctx context.Context, i *Websocket, d string) error {
		_, err := i.GetHistory(ctx, d)
		return err
	}},
	{name: "GetHistoryByVersion", action: rbac.ActionRead, call: func(ctx context.Context, i *Websocket, d string) error {
		_, err := i.GetHistoryByVersion(ctx, d, 1)
		return err
	}},
	{name: "GetHistoryMetadata", action: rbac.ActionRead, call: func(ctx context.Context, i *Websocket, d string) error {
		_, err := i.GetHistoryMetadata(ctx, d)
		return err
	}},
	{name: "CreateSnapshot", action: rbac.ActionRead, call: func(ctx context.Context, i *Websocket, d string) error {
		_, err := i.CreateSnapshot(ctx, d, 1, "n")
		return err
	}},
	{name: "Rollback", action: rbac.ActionEdit, call: func(ctx context.Context, i *Websocket, d string) error {
		_, err := i.Rollback(ctx, d, 1)
		return err
	}},
	{name: "FlushToGCS", action: rbac.ActionEdit, call: func(ctx context.Context, i *Websocket, d string) error {
		return i.FlushToGCS(ctx, d)
	}},
	{name: "ImportDocument", action: rbac.ActionEdit, call: func(ctx context.Context, i *Websocket, d string) error {
		return i.ImportDocument(ctx, d, []byte("x"))
	}},
	{name: "DeleteDocument", action: rbac.ActionDelete, call: func(ctx context.Context, i *Websocket, d string) error {
		return i.DeleteDocument(ctx, d)
	}},
}

// TestWebsocket_DeniedOperationsNeverReachTheClient is the security assertion.
// Before this interactor existed, Container.Websocket held the bare HTTP client,
// so any authenticated user could act on any project in any workspace. The worst
// of those was Rollback: it prunes every update above the target clock, making it
// a destructive cross-tenant operation.
func TestWebsocket_DeniedOperationsNeverReachTheClient(t *testing.T) {
	for _, op := range wsOps {
		t.Run(op.name, func(t *testing.T) {
			i, client, _, docID, _ := wsFixture(t, false) // permission denied
			err := op.call(context.Background(), i, docID)
			assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
			assert.Zero(t, client.calls, "denied %s still called through to the websocket client", op.name)
		})
	}
}

// TestWebsocket_ChecksTargetProjectWorkspace: the check must be evaluated against
// the workspace that owns the addressed project, not the caller's own. Passing no
// workspace would let the checker approve based on membership of any workspace,
// which is the cross-tenant hole this fixes.
func TestWebsocket_ChecksTargetProjectWorkspace(t *testing.T) {
	for _, op := range wsOps {
		t.Run(op.name, func(t *testing.T) {
			i, client, rc, docID, wsID := wsFixture(t, true)
			require.NoError(t, op.call(context.Background(), i, docID))

			assert.Equal(t, rbac.ResourceProject, rc.gotResource)
			assert.Equal(t, op.action, rc.gotAction, "%s must demand %q", op.name, op.action)
			require.Len(t, rc.gotWorkspace, 1, "%s must scope the check to one workspace", op.name)
			assert.Equal(t, wsID, rc.gotWorkspace[0], "%s checked the wrong workspace", op.name)
			assert.Equal(t, 1, client.calls, "%s should reach the client once when allowed", op.name)
		})
	}
}

// TestWebsocket_UnresolvableProjectIsDenied: authorization depends on resolving
// the project to a workspace, so anything that stops it resolving must deny
// rather than fall through. Otherwise a malformed or unknown id would be a way to
// skip the check entirely.
func TestWebsocket_UnresolvableProjectIsDenied(t *testing.T) {
	for _, docID := range []string{
		"",                       // empty
		"not-a-valid-ulid",       // unparseable
		project.NewID().String(), // well-formed but no such project
	} {
		t.Run(docID, func(t *testing.T) {
			i, client, _, _, _ := wsFixture(t, true) // checker would ALLOW
			_, err := i.GetLatest(context.Background(), docID)
			assert.Error(t, err, "an unresolvable project id must not be allowed through")
			assert.Zero(t, client.calls)
		})
	}
}

// TestWebsocket_CopyDocumentChecksBothProjects: a copy reads the source and
// writes the destination, so it needs edit on the destination AND read on the
// source. Checking only the destination would let a caller pull another
// workspace's document into a project they legitimately own.
func TestWebsocket_CopyDocumentChecksBothProjects(t *testing.T) {
	ctx := context.Background()
	projectRepo := memory.NewProject()

	dstWS := project.NewWorkspaceID()
	dst := project.New().NewID().Workspace(dstWS).MustBuild()
	require.NoError(t, projectRepo.Save(ctx, dst))

	srcWS := project.NewWorkspaceID()
	src := project.New().NewID().Workspace(srcWS).MustBuild()
	require.NoError(t, projectRepo.Save(ctx, src))

	// Deny only the SOURCE workspace: the destination is the caller's own project,
	// so a destination-only check would wrongly permit this.
	denySource := &perWorkspaceChecker{denied: accountsid.WorkspaceID(srcWS)}
	client := &countingWSClient{}
	i := NewWebsocket(client, projectRepo, denySource)

	err := i.CopyDocument(ctx, dst.ID().String(), src.ID().String())
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
	assert.Zero(t, client.calls, "copy proceeded despite no read permission on the source")

	// With both workspaces permitted it goes through.
	allowAll := &recordingChecker{allow: true}
	client2 := &countingWSClient{}
	i2 := NewWebsocket(client2, projectRepo, allowAll)
	require.NoError(t, i2.CopyDocument(ctx, dst.ID().String(), src.ID().String()))
	assert.Equal(t, 1, client2.calls)
}

// perWorkspaceChecker denies exactly one workspace and allows everything else.
type perWorkspaceChecker struct {
	denied accountsid.WorkspaceID
}

func (p *perWorkspaceChecker) CheckPermission(_ context.Context, _, _ string, workspaceID ...accountsid.WorkspaceID) (bool, error) {
	for _, w := range workspaceID {
		if w == p.denied {
			return false, nil
		}
	}
	return true, nil
}

// TestWebsocket_CloseNeedsNoPermission: Close is connection lifecycle, not a
// document operation, and is called on shutdown where there is no user context to
// authorize against.
func TestWebsocket_CloseNeedsNoPermission(t *testing.T) {
	i, client, _, _, _ := wsFixture(t, false) // deny everything
	require.NoError(t, i.Close())
	assert.Equal(t, 1, client.calls)
}
