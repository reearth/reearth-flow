package websocket

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestClient_GetSnapshots_DecodesList(t *testing.T) {
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

	got, err := client.GetSnapshots(context.Background(), "proj1")
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

// An unparseable timestamp must leave the zero value rather than substituting
// time.Now(). The Version panel sorts rows by timestamp and, for an unlabelled
// snapshot, renders the timestamp AS the row label — so a fabricated "now" would
// float a stale snapshot above genuinely newer ones and label it with today's
// date. The zero value sorts last and reads as obviously wrong, which is the
// honest failure mode. The row itself is still returned: one bad timestamp must
// not drop a snapshot the user can otherwise see and restore.
func TestClient_GetSnapshots_UnparseableTimestampStaysZero(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`[{"id":1,"label":"broken","timestamp":"not-a-timestamp","size":7}]`))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	got, err := client.GetSnapshots(context.Background(), "proj1")
	assert.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, int64(1), got[0].ID)
	assert.Equal(t, "broken", got[0].Label)
	assert.Equal(t, int64(7), got[0].Size)
	assert.True(t, got[0].Timestamp.IsZero(), "timestamp must stay zero, not be fabricated from time.Now()")
}

func TestClient_GetSnapshots_SetsAPISecretHeader(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "my-secret", r.Header.Get("X-API-Secret"))
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`[]`))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL, APISecret: "my-secret"})
	assert.NoError(t, err)

	_, err = client.GetSnapshots(context.Background(), "proj1")
	assert.NoError(t, err)
}

func TestClient_GetSnapshots_NonOKStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		_, _ = w.Write([]byte("boom"))
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	_, err = client.GetSnapshots(context.Background(), "proj1")
	assert.Error(t, err)
}

// TestClient_SaveNamedSnapshot_EnrichesFromList exercises the enrichment path:
// the save endpoint only returns {id, label} (Timestamp/Size zero-valued,
// mirroring the underlying store's SaveSnapshot signature), so the client
// must call the snapshot list to fill in Timestamp and Size before returning.
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
