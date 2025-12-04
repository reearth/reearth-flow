package asset

import (
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestAsset_Builder(t *testing.T) {
	aid := NewID()
	pid := id.NewProjectID()
	wid := accountsid.NewWorkspaceID()
	uid := accountsid.NewUserID()
	now := time.Now()

	status := ArchiveExtractionStatusDone
	tid := id.NewThreadID()

	a := New().
		ID(aid).
		Project(pid).
		Workspace(wid).
		CreatedAt(now).
		CreatedByUser(uid).
		FileName("test.png").
		Name("Test Asset").
		Size(1024).
		URL("https://example.com/test.png").
		ContentType("image/png").
		UUID("test-uuid").
		Thread(&tid).
		ArchiveExtractionStatus(status).
		FlatFiles(true).
		Public(true).
		MustBuild()

	assert.Equal(t, aid, a.ID())
	assert.Equal(t, pid, a.Project())
	assert.Equal(t, wid, a.Workspace())
	assert.Equal(t, now, a.CreatedAt())
	assert.Equal(t, &uid, a.User())
	assert.Equal(t, "test.png", a.FileName())
	assert.Equal(t, "Test Asset", a.Name())
	assert.Equal(t, uint64(1024), a.Size())
	assert.Equal(t, "https://example.com/test.png", a.URL())
	assert.Equal(t, "image/png", a.ContentType())
	assert.Equal(t, "test-uuid", a.UUID())
	assert.Equal(t, &tid, a.Thread())
	assert.Equal(t, &status, a.ArchiveExtractionStatus())
	assert.True(t, a.FlatFiles())
	assert.True(t, a.Public())
}

func TestAsset_NilHandling(t *testing.T) {
	// Create minimal asset with a user (required by reearthx)
	uid := accountsid.NewUserID()
	a := New().
		NewID().
		Project(id.NewProjectID()).
		Workspace(accountsid.NewWorkspaceID()).
		CreatedByUser(uid).
		FileName("test.txt").
		Name("test").
		Size(100).
		URL("https://example.com/test.txt").
		ContentType("text/plain").
		NewUUID().
		MustBuild()

	assert.NotNil(t, a.ID())
	assert.NotNil(t, a.Project())
	assert.NotNil(t, a.Workspace())
	assert.Equal(t, &uid, a.User())
	assert.Nil(t, a.Integration())
	assert.Nil(t, a.Thread())
	assert.Nil(t, a.ArchiveExtractionStatus())
	assert.False(t, a.FlatFiles())
	assert.False(t, a.Public())
}

func TestAsset_CreatedByIntegration(t *testing.T) {
	iid := id.NewIntegrationID()

	a := New().
		NewID().
		Project(id.NewProjectID()).
		Workspace(accountsid.NewWorkspaceID()).
		CreatedByIntegration(&iid).
		FileName("test.txt").
		Name("test").
		Size(100).
		URL("https://example.com/test.txt").
		ContentType("text/plain").
		NewUUID().
		MustBuild()

	assert.Nil(t, a.User())
	assert.Equal(t, &iid, a.Integration())
}

func TestAsset_List(t *testing.T) {
	aid1 := NewID()
	aid2 := NewID()
	aid3 := NewID()
	uid := accountsid.NewUserID()

	a1 := New().
		ID(aid1).
		Project(id.NewProjectID()).
		Workspace(accountsid.NewWorkspaceID()).
		CreatedByUser(uid).
		FileName("test1.txt").
		Name("test1").
		Size(100).
		URL("https://example.com/test1.txt").
		ContentType("text/plain").
		NewUUID().
		MustBuild()

	a2 := New().
		ID(aid2).
		Project(id.NewProjectID()).
		Workspace(accountsid.NewWorkspaceID()).
		CreatedByUser(uid).
		FileName("test2.txt").
		Name("test2").
		Size(200).
		URL("https://example.com/test2.txt").
		ContentType("text/plain").
		NewUUID().
		MustBuild()

	a3 := New().
		ID(aid3).
		Project(id.NewProjectID()).
		Workspace(accountsid.NewWorkspaceID()).
		CreatedByUser(uid).
		FileName("test3.txt").
		Name("test3").
		Size(300).
		URL("https://example.com/test3.txt").
		ContentType("text/plain").
		NewUUID().
		MustBuild()

	list := List{a1, a2, nil, a3}
	ids := list.IDs()

	assert.Len(t, ids, 3)
	assert.Contains(t, ids, aid1)
	assert.Contains(t, ids, aid2)
	assert.Contains(t, ids, aid3)
}

func TestAsset_WorkspaceOnly(t *testing.T) {
	ws := accountsid.NewWorkspaceID()
	uid := accountsid.NewUserID()

	// Test creating a workspace-only asset (no project)
	a := New().
		NewID().
		Workspace(ws).
		CreatedByUser(uid).
		FileName("test.jpg").
		Size(100).
		URL("http://example.com/test").
		NewUUID().
		MustBuild()

	assert.NotNil(t, a)
	assert.NotNil(t, a.ID())
	assert.Equal(t, ws, a.Workspace())
	// Project should be empty/zero value
	assert.Equal(t, id.ProjectID{}, a.Project())
}
