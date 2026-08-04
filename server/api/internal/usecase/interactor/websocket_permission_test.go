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

// assertDenied: a denied operation must error AND never reach the client. The
// call count is the assertion that matters, since an error that still calls
// through has denied nothing.
func assertDenied(t *testing.T, name string, call func(context.Context, *Websocket, string) error) {
	t.Helper()
	i, client, _, docID, _ := wsFixture(t, false)
	err := call(context.Background(), i, docID)
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied, name)
	assert.Zero(t, client.calls, "denied %s still called the client", name)
}

// assertChecks: the check must be scoped to the workspace owning the addressed
// project, with the right action. Without the workspace the checker could
// approve on membership of any workspace, which is the cross-tenant hole.
func assertChecks(t *testing.T, name, action string, call func(context.Context, *Websocket, string) error) {
	t.Helper()
	i, client, rc, docID, wsID := wsFixture(t, true)
	require.NoError(t, call(context.Background(), i, docID), name)
	assert.Equal(t, rbac.ResourceProject, rc.gotResource, name)
	assert.Equal(t, action, rc.gotAction, "%s action", name)
	require.Len(t, rc.gotWorkspace, 1, "%s workspace", name)
	assert.Equal(t, wsID, rc.gotWorkspace[0], "%s checked the wrong workspace", name)
	assert.Equal(t, 1, client.calls, "%s should reach the client once", name)
}

func getLatest(ctx context.Context, i *Websocket, d string) error {
	_, err := i.GetLatest(ctx, d)
	return err
}
func getHistory(ctx context.Context, i *Websocket, d string) error {
	_, err := i.GetHistory(ctx, d)
	return err
}
func getHistoryByVersion(ctx context.Context, i *Websocket, d string) error {
	_, err := i.GetHistoryByVersion(ctx, d, 1)
	return err
}
func getHistoryMetadata(ctx context.Context, i *Websocket, d string) error {
	_, err := i.GetHistoryMetadata(ctx, d)
	return err
}
func createSnapshot(ctx context.Context, i *Websocket, d string) error {
	_, err := i.CreateSnapshot(ctx, d, 1, "n")
	return err
}
func rollback(ctx context.Context, i *Websocket, d string) error {
	_, err := i.Rollback(ctx, d, 1)
	return err
}
func flushToGCS(ctx context.Context, i *Websocket, d string) error { return i.FlushToGCS(ctx, d) }
func importDocument(ctx context.Context, i *Websocket, d string) error {
	return i.ImportDocument(ctx, d, []byte("x"))
}
func deleteDocument(ctx context.Context, i *Websocket, d string) error {
	return i.DeleteDocument(ctx, d)
}

// TestWebsocket_DeniedOperationsNeverReachTheClient is the security assertion:
// before this interactor, any authenticated user could act on any project.
//
// CopyDocument and Close are covered separately below. CopyDocument authorizes
// twice and recordingChecker keeps only the last call, so listing it here would
// assert ActionRead and stop checking the destination.
func TestWebsocket_DeniedOperationsNeverReachTheClient(t *testing.T) {
	assertDenied(t, "GetLatest", getLatest)
	assertDenied(t, "GetHistory", getHistory)
	assertDenied(t, "GetHistoryByVersion", getHistoryByVersion)
	assertDenied(t, "GetHistoryMetadata", getHistoryMetadata)
	assertDenied(t, "CreateSnapshot", createSnapshot)
	assertDenied(t, "Rollback", rollback)
	assertDenied(t, "FlushToGCS", flushToGCS)
	assertDenied(t, "ImportDocument", importDocument)
	assertDenied(t, "DeleteDocument", deleteDocument)
}

// TestWebsocket_ChecksTargetProjectWorkspace pins the action each operation
// demands, and that it is evaluated against the addressed project's workspace.
func TestWebsocket_ChecksTargetProjectWorkspace(t *testing.T) {
	assertChecks(t, "GetLatest", rbac.ActionRead, getLatest)
	assertChecks(t, "GetHistory", rbac.ActionRead, getHistory)
	assertChecks(t, "GetHistoryByVersion", rbac.ActionRead, getHistoryByVersion)
	assertChecks(t, "GetHistoryMetadata", rbac.ActionRead, getHistoryMetadata)
	assertChecks(t, "CreateSnapshot", rbac.ActionRead, createSnapshot)
	assertChecks(t, "Rollback", rbac.ActionEdit, rollback)
	assertChecks(t, "FlushToGCS", rbac.ActionEdit, flushToGCS)
	assertChecks(t, "ImportDocument", rbac.ActionEdit, importDocument)
	assertChecks(t, "DeleteDocument", rbac.ActionDelete, deleteDocument)
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
