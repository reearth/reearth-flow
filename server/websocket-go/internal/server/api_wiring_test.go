package server

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	flowhttp "github.com/reearth/reearth-flow/websocket-go/internal/http"
)

// TestHandlerWithAPIGuardsOnlyAPI verifies /api/* is behind X-API-Secret while
// /health stays unguarded, and that /api/* routes reach the API router not WS.
func TestHandlerWithAPIGuardsOnlyAPI(t *testing.T) {
	s := New(&config.Config{Origins: []string{"*"}, MaxConnections: 10, MaxPeersPerRoom: 10, MaxRooms: 10})
	s.SetHealthChecks(nil, nil) // 503 unconfigured but reachable

	mw, err := flowhttp.RequireAPISecret(flowhttp.APISecretConfig{Secret: "sekret", AppEnv: "development"})
	if err != nil {
		t.Fatalf("secret guard: %v", err)
	}
	api := flowhttp.NewRouter(flowhttp.Deps{Store: nopStore{}})
	h := s.HandlerWithAPI(mw(api))

	// /api/* without the secret => 401.
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest("GET", "/api/document/proj1", nil))
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("/api/* unguarded? status = %d, want 401", rec.Code)
	}

	// /api/* with the secret => reaches the handler (200).
	req := httptest.NewRequest("GET", "/api/document/proj1", nil)
	req.Header.Set("X-API-Secret", "sekret")
	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("/api/* with secret: status = %d, want 200", rec.Code)
	}

	// /health is NOT behind the secret (reachable without the header).
	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest("GET", "/health", nil))
	if rec.Code == http.StatusUnauthorized {
		t.Fatalf("/health must not require X-API-Secret")
	}
}

// nopStore is a DocStore that returns empty results, enough to exercise routing.
type nopStore struct{}

func (nopStore) Load(ctx context.Context, room string) (flowhttp.LoadResult, error) {
	return flowhttp.LoadResult{}, nil
}
func (nopStore) ListVersions(ctx context.Context, room string) ([]flowhttp.VersionInfo, error) {
	return nil, nil
}
func (nopStore) GetUpdate(ctx context.Context, room string, v uint64) ([]byte, flowhttp.VersionInfo, bool, error) {
	return nil, flowhttp.VersionInfo{}, false, nil
}
func (nopStore) MaterializeAt(ctx context.Context, room string, v uint64) ([]byte, error) {
	return nil, nil
}
func (nopStore) PruneAfter(ctx context.Context, room string, target uint64, rolledBack []byte) error {
	return nil
}
func (nopStore) Flush(ctx context.Context, room string) error { return nil }
func (nopStore) CaptureSnapshot(ctx context.Context, room, name string, state []byte) (uint64, error) {
	return 0, nil
}
func (nopStore) Copy(ctx context.Context, dst, src string) error                      { return nil }
func (nopStore) Import(ctx context.Context, room string, data []byte) (uint64, error) { return 0, nil }
func (nopStore) Compact(ctx context.Context, room string, keep int) (int, error)      { return 0, nil }
func (nopStore) Delete(ctx context.Context, room string) error                        { return nil }
func (nopStore) CleanupAll(ctx context.Context, keep int) (int, error)                { return 0, nil }
