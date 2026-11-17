package websocket

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestClient_SetProjectLock(t *testing.T) {
	t.Parallel()

	var got setProjectLockRequest
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "/api/document/proj1/lock", r.URL.Path)
		assert.Equal(t, "my-secret", r.Header.Get("X-API-Secret"))
		assert.NoError(t, json.NewDecoder(r.Body).Decode(&got))
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	c, err := NewClient(Config{ServerURL: server.URL, APISecret: "my-secret"})
	require.NoError(t, err)

	at := time.UnixMilli(1_700_000_000_123)
	require.NoError(t, c.SetProjectLock(context.Background(), "proj1", true, at))
	assert.Equal(t, setProjectLockRequest{Locked: true, ChangedAt: 1_700_000_000_123}, got)
}

func TestClient_SetProjectLock_ReportsAServerError(t *testing.T) {
	t.Parallel()

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer server.Close()

	c, err := NewClient(Config{ServerURL: server.URL})
	require.NoError(t, err)

	err = c.SetProjectLock(context.Background(), "proj1", false, time.UnixMilli(1))
	require.Error(t, err)
	assert.NotErrorIs(t, err, interfaces.ErrWebsocketRequestRejected, "a server error may pass")
}

func TestClient_SetProjectLock_MarksARefusal(t *testing.T) {
	t.Parallel()

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
	}))
	defer server.Close()

	c, err := NewClient(Config{ServerURL: server.URL})
	require.NoError(t, err)

	err = c.SetProjectLock(context.Background(), "proj1", true, time.UnixMilli(1))
	assert.ErrorIs(t, err, interfaces.ErrWebsocketRequestRejected)
}
