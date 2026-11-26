package websocket

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func versionServer(t *testing.T, status int, body string) *Client {
	t.Helper()
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(status)
		_, _ = w.Write([]byte(body))
	}))
	t.Cleanup(server.Close)
	c, err := NewClient(Config{ServerURL: server.URL})
	require.NoError(t, err)
	return c
}

func TestClient_SaveVersion(t *testing.T) {
	t.Parallel()

	var got saveVersionRequest
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "/api/document/proj1/versions", r.URL.Path)
		assert.Equal(t, "my-secret", r.Header.Get("X-API-Secret"))
		assert.NoError(t, json.NewDecoder(r.Body).Decode(&got))
		_, _ = w.Write([]byte(`{"saved":true,"snapshotId":12}`))
	}))
	defer server.Close()

	c, err := NewClient(Config{ServerURL: server.URL, APISecret: "my-secret"})
	require.NoError(t, err)

	saved, err := c.SaveVersion(context.Background(), "proj1", "save")
	require.NoError(t, err)
	assert.True(t, saved)
	assert.Equal(t, saveVersionRequest{Label: "save"}, got)
}

func TestClient_SaveVersion_NothingChanged(t *testing.T) {
	t.Parallel()

	saved, err := versionServer(t, http.StatusOK, `{"saved":false}`).SaveVersion(context.Background(), "proj1", "run")
	require.NoError(t, err)
	assert.False(t, saved)
}

func TestClient_SaveVersion_Busy(t *testing.T) {
	t.Parallel()

	_, err := versionServer(t, http.StatusConflict, `{"error":"busy"}`).SaveVersion(context.Background(), "proj1", "run")
	assert.ErrorIs(t, err, interfaces.ErrVersionSaveBusy)
	assert.True(t, interfaces.IsVersionSaveSkipped(err))
}

func TestClient_SaveVersion_Unsupported(t *testing.T) {
	t.Parallel()

	_, err := versionServer(t, http.StatusNotImplemented, `{}`).SaveVersion(context.Background(), "proj1", "run")
	assert.ErrorIs(t, err, interfaces.ErrVersionsUnsupported)
	assert.True(t, interfaces.IsVersionSaveSkipped(err))
}

func TestClient_SaveVersion_MarksARefusal(t *testing.T) {
	t.Parallel()

	_, err := versionServer(t, http.StatusUnauthorized, `{}`).SaveVersion(context.Background(), "proj1", "run")
	assert.ErrorIs(t, err, interfaces.ErrWebsocketRequestRejected)
	assert.False(t, interfaces.IsVersionSaveSkipped(err))
}

func TestClient_SaveVersion_ReportsAServerError(t *testing.T) {
	t.Parallel()

	_, err := versionServer(t, http.StatusInternalServerError, `{}`).SaveVersion(context.Background(), "proj1", "run")
	require.Error(t, err)
	assert.NotErrorIs(t, err, interfaces.ErrWebsocketRequestRejected)
	assert.False(t, interfaces.IsVersionSaveSkipped(err))
}

// TestClient_RollbackRefusesAVersionTheStoreCannotRebuild: the websocket
// server answers 409 when it cannot rebuild the version exactly. That used to
// arrive as a bare status string, so the UI could not tell a refused entry from
// an outage.
func TestClient_RollbackRefusesAVersionTheStoreCannotRebuild(t *testing.T) {
	t.Parallel()

	_, err := versionServer(t, http.StatusConflict, `{"error":"this version can no longer be restored"}`).Rollback(context.Background(), "proj1", 3)
	assert.ErrorIs(t, err, interfaces.ErrVersionNotRebuildable)
}

func TestClient_CreateSnapshotRefusesAVersionTheStoreCannotRebuild(t *testing.T) {
	t.Parallel()

	_, err := versionServer(t, http.StatusConflict, `{"error":"this version can no longer be captured"}`).CreateSnapshot(context.Background(), "proj1", 3, "")
	assert.ErrorIs(t, err, interfaces.ErrVersionNotRebuildable)
}

// TestClient_RollbackFailureIsNotARefusal: only a 409 is a refusal; a server
// error must not tell the UI the entry is unusable.
func TestClient_RollbackFailureIsNotARefusal(t *testing.T) {
	t.Parallel()

	_, err := versionServer(t, http.StatusInternalServerError, `{"error":"rollback failed"}`).Rollback(context.Background(), "proj1", 3)
	require.Error(t, err)
	assert.NotErrorIs(t, err, interfaces.ErrVersionNotRebuildable)
}
