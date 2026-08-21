package websocket

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
)

func TestClient_GetNamedSnapshots_DecodesList(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodGet, r.Method)
		assert.Equal(t, "/api/document/proj1/snapshots", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`[{"id":2,"label":"second","timestamp":"2026-07-30T10:00:00Z","size":11},
		                        {"id":1,"label":"first","timestamp":"2026-07-30T09:00:00Z","size":7}]`))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	got, err := client.GetNamedSnapshots(context.Background(), "proj1")
	assert.NoError(t, err)
	assert.Len(t, got, 2)

	assert.Equal(t, int64(2), got[0].ID)
	assert.Equal(t, "second", got[0].Label)
	assert.Equal(t, int64(11), got[0].Size)
	assert.False(t, got[0].Timestamp.IsZero())

	assert.Equal(t, int64(1), got[1].ID)
	assert.Equal(t, "first", got[1].Label)
	assert.Equal(t, int64(7), got[1].Size)
	assert.False(t, got[1].Timestamp.IsZero())
}

// An unparseable timestamp must stay zero rather than becoming time.Now(), which
// would misorder and mislabel history. The row is still returned.
func TestClient_GetNamedSnapshots_UnparseableTimestampStaysZero(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`[{"id":1,"label":"broken","timestamp":"not-a-timestamp","size":7}]`))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	got, err := client.GetNamedSnapshots(context.Background(), "proj1")
	assert.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, int64(1), got[0].ID)
	assert.Equal(t, "broken", got[0].Label)
	assert.Equal(t, int64(7), got[0].Size)
	assert.True(t, got[0].Timestamp.IsZero(), "timestamp must stay zero, not be fabricated from time.Now()")
}

func TestClient_GetNamedSnapshots_SetsAPISecretHeader(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "my-secret", r.Header.Get("X-API-Secret"))
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`[]`))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL, APISecret: "my-secret"})
	assert.NoError(t, err)

	_, err = client.GetNamedSnapshots(context.Background(), "proj1")
	assert.NoError(t, err)
}

func TestClient_GetNamedSnapshots_NonOKStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		_, _ = w.Write([]byte("boom"))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	_, err = client.GetNamedSnapshots(context.Background(), "proj1")
	assert.Error(t, err)
}

// TestClient_SaveNamedSnapshot_EnrichesFromList: save returns only {id, label},
// so the client must fill Timestamp and Size from the list.
func TestClient_SaveNamedSnapshot_EnrichesFromList(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/api/document/proj1/snapshots", r.URL.Path)
		assert.Equal(t, "my-secret", r.Header.Get("X-API-Secret"))

		switch r.Method {
		case http.MethodPost:
			var body struct {
				Label string `json:"label"`
			}
			assert.NoError(t, json.NewDecoder(r.Body).Decode(&body))
			assert.Equal(t, "milestone", body.Label)

			w.Header().Set("Content-Type", "application/json")
			_, _ = w.Write([]byte(`{"id":3,"label":"milestone"}`))
		case http.MethodGet:
			w.Header().Set("Content-Type", "application/json")
			_, _ = w.Write([]byte(`[{"id":3,"label":"milestone","timestamp":"2026-08-03T12:00:00Z","size":42},
			                        {"id":2,"label":"second","timestamp":"2026-07-30T10:00:00Z","size":11}]`))
		default:
			t.Fatalf("unexpected method %s", r.Method)
		}
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL, APISecret: "my-secret"})
	assert.NoError(t, err)

	got, err := client.SaveNamedSnapshot(context.Background(), "proj1", "milestone")
	assert.NoError(t, err)
	assert.Equal(t, int64(3), got.ID)
	assert.Equal(t, "milestone", got.Label)
	assert.Equal(t, int64(42), got.Size)
	assert.False(t, got.Timestamp.IsZero())
}

func TestClient_SaveNamedSnapshot_FallsBackToThinMetadata_WhenNotInList(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		switch r.Method {
		case http.MethodPost:
			_, _ = w.Write([]byte(`{"id":9,"label":"missing"}`))
		case http.MethodGet:
			_, _ = w.Write([]byte(`[]`))
		}
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	got, err := client.SaveNamedSnapshot(context.Background(), "proj1", "missing")
	assert.NoError(t, err)
	assert.Equal(t, int64(9), got.ID)
	assert.Equal(t, "missing", got.Label)
	assert.True(t, got.Timestamp.IsZero())
}

func TestClient_GetSnapshotState_DecodesIntArrayUpdates(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodGet, r.Method)
		// Addressed by snapshot number, never by the update-log clock.
		assert.Equal(t, "/api/document/proj1/snapshots/7", r.URL.Path)
		w.Header().Set("Content-Type", "application/json")
		// websocket-go's UpdateBytes marshals as a JSON int array, not base64.
		_, _ = w.Write([]byte(`{"id":"proj1","snapshot_id":7,"updates":[1,2,255]}`))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	got, err := client.GetSnapshotState(context.Background(), "proj1", 7)
	assert.NoError(t, err)
	assert.Equal(t, int64(7), got.SnapshotID)
	// The exact bytes matter: this state is applied to a live Y.Doc, so a decode
	// that silently produced empty updates would make restore a no-op.
	assert.Equal(t, []int{1, 2, 255}, got.Updates)
}

func TestClient_GetSnapshotState_NotFoundIsItsOwnError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		_, _ = w.Write([]byte(`{"error":"snapshot not found"}`))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	// Retention (KeepSnapshots) evicts snapshots, so a listed row can be gone by
	// the time it is clicked. That must be distinguishable from a server fault.
	_, err = client.GetSnapshotState(context.Background(), "proj1", 7)
	assert.ErrorIs(t, err, interfaces.ErrSnapshotNotFound)
}

func TestClient_GetSnapshotState_ServerErrorIsNotNotFound(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	_, err = client.GetSnapshotState(context.Background(), "proj1", 7)
	assert.Error(t, err)
	assert.NotErrorIs(t, err, interfaces.ErrSnapshotNotFound)
}

func TestClient_GetSnapshotState_SetsAPISecretHeader(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "shh", r.Header.Get("X-API-Secret"))
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"id":"proj1","snapshot_id":1,"updates":[]}`))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL, APISecret: "shh"})
	assert.NoError(t, err)

	_, err = client.GetSnapshotState(context.Background(), "proj1", 1)
	assert.NoError(t, err)
}
