package auth

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const testVersionSecret = "s3cret"

// fakeRecorder records the edit the handler asked for.
type fakeRecorder struct {
	at    time.Time
	err   error
	by    *accountsid.UserID
	pid   id.ProjectID
	calls int
}

func (f *fakeRecorder) RecordEdit(_ context.Context, pid id.ProjectID, at time.Time, by *accountsid.UserID) error {
	f.calls++
	f.pid, f.at, f.by = pid, at, by
	return f.err
}

var versionTestNow = time.Date(2026, 11, 18, 3, 0, 0, 0, time.UTC)

func newTestVersionHandler(secret string, projects ProjectEditRecorder) *VersionHandler {
	h := NewVersionHandler(secret, projects)
	h.now = func() time.Time { return versionTestNow }
	return h
}

func postNotice(t *testing.T, h http.Handler, secret, body string) *httptest.ResponseRecorder {
	t.Helper()
	req := httptest.NewRequest(http.MethodPost, VersionsPath, strings.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	if secret != "" {
		req.Header.Set("X-API-Secret", secret)
	}
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	return rec
}

func notice(pid id.ProjectID, extra string) string {
	return `{"projectId":"` + pid.String() + `","versionId":12,"label":"auto","savedAt":"2026-11-18T02:59:00Z","contributorIds":[]` + extra + `}`
}

func TestVersionHandler_RecordsTheLastEdit(t *testing.T) {
	t.Parallel()
	f := &fakeRecorder{}
	pid := id.NewProjectID()
	uid := accountsid.NewUserID()

	rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret,
		notice(pid, `,"lastEditedAt":"2026-11-18T02:58:41.123Z","lastEditorId":"`+uid.String()+`"`))

	assert.Equal(t, http.StatusNoContent, rec.Code)
	require.Equal(t, 1, f.calls)
	assert.Equal(t, pid, f.pid)
	assert.True(t, f.at.Equal(time.Date(2026, 11, 18, 2, 58, 41, 123_000_000, time.UTC)), "at = %v", f.at)
	assert.Equal(t, &uid, f.by)
}

func TestVersionHandler_WithoutEditTimeUsesSavedAt(t *testing.T) {
	t.Parallel()
	f := &fakeRecorder{}

	rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret, notice(id.NewProjectID(), ""))

	assert.Equal(t, http.StatusNoContent, rec.Code)
	assert.True(t, f.at.Equal(time.Date(2026, 11, 18, 2, 59, 0, 0, time.UTC)), "at = %v", f.at)
	assert.Nil(t, f.by, "no lastEditorId records an unknown editor")
}

func TestVersionHandler_NeverRecordsATimeAheadOfItsClock(t *testing.T) {
	t.Parallel()
	f := &fakeRecorder{}

	rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret,
		notice(id.NewProjectID(), `,"lastEditedAt":"2026-11-18T04:00:00Z"`))

	assert.Equal(t, http.StatusNoContent, rec.Code)
	assert.True(t, f.at.Equal(versionTestNow), "at = %v", f.at)
}

func TestVersionHandler_RefusesWithoutTheSecret(t *testing.T) {
	t.Parallel()
	pid := id.NewProjectID()

	t.Run("missing header", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{}
		rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), "", notice(pid, ""))
		assert.Equal(t, http.StatusUnauthorized, rec.Code)
		assert.Zero(t, f.calls)
	})

	t.Run("wrong secret", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{}
		rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), "guess", notice(pid, ""))
		assert.Equal(t, http.StatusUnauthorized, rec.Code)
		assert.Zero(t, f.calls)
	})

	t.Run("no secret configured", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{}
		rec := postNotice(t, newTestVersionHandler("", f), "", notice(pid, ""))
		assert.Equal(t, http.StatusUnauthorized, rec.Code)
		assert.Zero(t, f.calls)
	})
}

func TestVersionHandler_RejectsBadNotices(t *testing.T) {
	t.Parallel()
	pid := id.NewProjectID()

	t.Run("not JSON", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{}
		rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret, "{")
		assert.Equal(t, http.StatusBadRequest, rec.Code)
		assert.Zero(t, f.calls)
	})

	t.Run("invalid project id", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{}
		rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret,
			`{"projectId":"nope","savedAt":"2026-11-18T02:59:00Z"}`)
		assert.Equal(t, http.StatusBadRequest, rec.Code)
		assert.Zero(t, f.calls)
	})

	t.Run("no time at all", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{}
		rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret,
			`{"projectId":"`+pid.String()+`"}`)
		assert.Equal(t, http.StatusBadRequest, rec.Code)
		assert.Zero(t, f.calls)
	})

	t.Run("invalid editor id", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{}
		rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret,
			notice(pid, `,"lastEditorId":"nope"`))
		assert.Equal(t, http.StatusBadRequest, rec.Code)
		assert.Zero(t, f.calls)
	})

	t.Run("not a POST", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{}
		req := httptest.NewRequest(http.MethodGet, VersionsPath, nil)
		req.Header.Set("X-API-Secret", testVersionSecret)
		rec := httptest.NewRecorder()
		newTestVersionHandler(testVersionSecret, f).ServeHTTP(rec, req)
		assert.Equal(t, http.StatusMethodNotAllowed, rec.Code)
		assert.Zero(t, f.calls)
	})
}

// The websocket server retries 5xx and gives up on any other 4xx.
func TestVersionHandler_StatusTellsTheSenderWhetherToRetry(t *testing.T) {
	t.Parallel()

	t.Run("unknown project", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{err: rerror.ErrNotFound}
		rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret, notice(id.NewProjectID(), ""))
		assert.Equal(t, http.StatusNotFound, rec.Code)
	})

	t.Run("store failure", func(t *testing.T) {
		t.Parallel()
		f := &fakeRecorder{err: errors.New("db down")}
		rec := postNotice(t, newTestVersionHandler(testVersionSecret, f), testVersionSecret, notice(id.NewProjectID(), ""))
		assert.Equal(t, http.StatusInternalServerError, rec.Code)
	})
}

func TestServer_RoutesVersionNotices(t *testing.T) {
	t.Parallel()
	f := &fakeRecorder{}
	s := NewServer(nil, nil, newTestVersionHandler(testVersionSecret, f))

	rec := postNotice(t, s, testVersionSecret, notice(id.NewProjectID(), ""))

	assert.Equal(t, http.StatusNoContent, rec.Code)
	assert.Equal(t, 1, f.calls)
}

// Notices arrive unordered and repeated; a version's notice can come again
// with more contributors and the same lastEditedAt. Only the latest edit wins.
func TestVersionHandler_UnorderedAndRepeatedNotices(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	projects := memory.NewProject()
	metadataEditor, alice, bob := accountsid.NewUserID(), accountsid.NewUserID(), accountsid.NewUserID()
	base := time.Date(2026, 11, 18, 2, 0, 0, 0, time.UTC)
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).
		UpdatedAt(base).UpdatedBy(&metadataEditor).MustBuild()
	require.NoError(t, projects.Save(ctx, p))
	h := newTestVersionHandler(testVersionSecret, projects)

	send := func(versionID, editedAt string, editor accountsid.UserID, contributors string) {
		t.Helper()
		body := `{"projectId":"` + p.ID().String() + `","versionId":` + versionID + `,"label":"auto",` +
			`"savedAt":"2026-11-18T02:50:00Z","lastEditedAt":"` + editedAt + `",` +
			`"lastEditorId":"` + editor.String() + `","contributorIds":` + contributors + `}`
		require.Equal(t, http.StatusNoContent, postNotice(t, h, testVersionSecret, body).Code)
	}
	assertProject := func(at time.Time, by accountsid.UserID) {
		t.Helper()
		got, err := projects.FindByID(ctx, p.ID())
		require.NoError(t, err)
		assert.True(t, got.UpdatedAt().Equal(at), "updatedAt = %v, want %v", got.UpdatedAt(), at)
		assert.Equal(t, &by, got.UpdatedBy())
	}

	// Version 2 arrives before version 1.
	send("2", "2026-11-18T02:40:00Z", bob, `["`+bob.String()+`"]`)
	assertProject(base.Add(40*time.Minute), bob)
	send("1", "2026-11-18T02:20:00Z", alice, `["`+alice.String()+`"]`)
	assertProject(base.Add(40*time.Minute), bob)

	// Version 2 again, credited with alice's edit: same time, more contributors.
	send("2", "2026-11-18T02:40:00Z", bob, `["`+bob.String()+`","`+alice.String()+`"]`)
	assertProject(base.Add(40*time.Minute), bob)
}
