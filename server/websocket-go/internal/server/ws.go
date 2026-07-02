// Package server wires the ygo WebSocket provider into the reearth-flow HTTP
// surface: the y-websocket transport at GET /{doc_id} and a /health probe.
package server

import (
	"context"
	"log/slog"
	"net/http"
	"time"

	"github.com/reearth/ygo/crdt"
	ygws "github.com/reearth/ygo/provider/websocket"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	"github.com/reearth/reearth-flow/websocket-go/internal/docid"
)

// defaultResyncInterval is the period of the server-side re-sync loop.
const defaultResyncInterval = 30 * time.Second

// Server hosts the WebSocket transport and the HTTP surface around it.
type Server struct {
	cfg *config.Config
	ws  *ygws.Server
	log *slog.Logger

	// onPeriodicSync, if set, is invoked once per re-sync tick (tests only).
	onPeriodicSync func()

	health healthDeps
}

// New builds a Server from config using the in-memory provider.
func New(cfg *config.Config) *Server {
	s := &Server{
		cfg: cfg,
		ws:  ygws.NewServer(),
		log: slog.Default(),
	}
	s.ws.AllowedOrigins = cfg.Origins
	s.ws.Logger = s.log
	// DoS caps: ygo treats 0 as unlimited, so these must be set (and replicated
	// in NewWithPersistence). doc_id is client-supplied.
	s.ws.MaxConnections = cfg.MaxConnections
	s.ws.MaxPeersPerRoom = cfg.MaxPeersPerRoom
	s.ws.MaxRooms = cfg.MaxRooms
	return s
}

// WSProvider exposes the underlying ygo provider for cluster/persistence hooks.
func (s *Server) WSProvider() *ygws.Server { return s.ws }

// Handler returns the HTTP handler: the WS transport at GET /{doc_id} plus /health.
func (s *Server) Handler() http.Handler {
	return s.HandlerWithAPI(nil)
}

// HandlerWithAPI returns the full HTTP handler, additionally mounting the
// secret-guarded /api/* routes when apiHandler is non-nil. /health and the WS
// upgrade stay unguarded; apiHandler must carry its own X-API-Secret middleware.
func (s *Server) HandlerWithAPI(apiHandler http.Handler) http.Handler {
	mux := http.NewServeMux()
	mux.Handle("GET /{doc_id}", s.wsHandler())
	s.registerHealth(mux)
	if apiHandler != nil {
		mux.Handle("/api/", apiHandler)
	}
	return mux
}

// wsHandler normalizes the {doc_id} path param into the "room" path value the
// ygo provider reads, so "/{uuid}:main" and "/{uuid}" resolve to one room.
func (s *Server) wsHandler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		room := docid.Normalize(r.PathValue("doc_id"))
		r.SetPathValue("room", room)
		s.ws.ServeHTTP(w, r)
	})
}

// resyncInterval returns the configured interval or the default.
func resyncInterval(d time.Duration) time.Duration {
	if d > 0 {
		return d
	}
	return defaultResyncInterval
}

// StartPeriodicSync re-broadcasts each active room's full state on every tick
// until ctx is cancelled, keeping connected peers converged.
func (s *Server) StartPeriodicSync(ctx context.Context, interval time.Duration) {
	t := time.NewTicker(resyncInterval(interval))
	defer t.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			s.resyncRooms(ctx)
			if s.onPeriodicSync != nil {
				s.onPeriodicSync()
			}
		}
	}
}

// resyncRooms re-broadcasts the current full state of every active room.
func (s *Server) resyncRooms(ctx context.Context) {
	for _, room := range s.ws.Rooms() {
		doc := s.ws.GetDoc(room)
		if doc == nil {
			continue
		}
		update := encodeFullState(doc)
		if len(update) == 0 {
			continue
		}
		if err := s.ws.BroadcastUpdate(ctx, room, update); err != nil {
			s.log.Debug("periodic resync broadcast failed", "room", room, "err", err)
		}
	}
}

// encodeFullState encodes the document's complete state as a V1 update.
func encodeFullState(doc *crdt.Doc) []byte {
	return crdt.EncodeStateAsUpdateV1(doc, nil)
}
