// Package server wires the ygo WebSocket provider into the reearth-flow HTTP
// surface: the y-websocket transport at GET /{doc_id} and a /health probe.
package server

import (
	"context"
	"errors"
	"log/slog"
	"net/http"
	"sync"
	"sync/atomic"
	"time"

	"github.com/reearth/ygo/crdt"
	ygws "github.com/reearth/ygo/provider/websocket"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	"github.com/reearth/reearth-flow/websocket-go/internal/docid"
	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
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

	// locks and presence back the project lock: see project_lock.go.
	locks    *projectLocks
	presence Presence

	// Versioning: see versions.go. Nil until EnableVersions.
	changes      versions.Store
	tracker      *versions.Tracker
	versions     *versions.Saver
	stored       func(ctx context.Context, room string) ([]byte, error)
	catchUp      CatchUp
	versionEvery time.Duration
	// leaveSaves counts the versions lastPeerLeft is saving; see WaitVersions.
	leaveMu    sync.Mutex
	leaveSaves int
	leaveIdle  chan struct{}

	health healthDeps

	// shuttingDown refuses new WebSocket connections once Shutdown has begun.
	shuttingDown atomic.Bool
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
	s.ws.RoomIdleTimeout = cfg.RoomIdleTimeout
	if cfg.SlowPeerResync {
		s.ws.SlowPeerPolicy = ygws.SlowPeerResync
	}
	s.enableProjectLock()
	return s
}

// closeRoomPasses bounds how often Shutdown closes rooms that a relay entry
// loaded again while it closed them.
const closeRoomPasses = 3

// Shutdown closes every room, so each is let go as when its last peer leaves,
// then shuts the provider down. ygo's own Shutdown leaves rooms with no peers
// loaded, and the relay would never hear that this instance let them go.
//
// New connections are refused from the start: ygo's CloseRoom does not count
// peers again before removing a room, so a peer joining meanwhile would be left
// on a room with no relay and no persistence.
func (s *Server) Shutdown(ctx context.Context) error {
	s.shuttingDown.Store(true)
	s.closeRooms(ctx)
	return s.ws.Shutdown(ctx)
}

// closeRooms closes every loaded room in parallel until none is left, ctx ends
// or closeRoomPasses runs out. A relay entry can load a room again while it is
// closed.
func (s *Server) closeRooms(ctx context.Context) {
	for range closeRoomPasses {
		rooms := s.ws.Rooms()
		if len(rooms) == 0 {
			return
		}
		var wg sync.WaitGroup
		for _, room := range rooms {
			wg.Go(func() {
				if err := s.ws.CloseRoom(room, true); err != nil && !errors.Is(err, ygws.ErrRoomNotFound) {
					s.log.Warn("closing a room at shutdown failed", "room", room, "err", err)
				}
			})
		}
		done := make(chan struct{})
		go func() {
			wg.Wait()
			close(done)
		}()
		select {
		case <-done:
		case <-ctx.Done():
			return
		}
	}
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
		// Reject a doc_id that normalizes to empty (a bare ":main", or whitespace)
		// before it reaches ygo, matching the Rust server's non-empty validation and
		// preventing distinct invalid URLs from collapsing into a single room "".
		if room == "" {
			http.Error(w, "doc_id required", http.StatusBadRequest)
			return
		}
		if s.shuttingDown.Load() {
			http.Error(w, "server is shutting down", http.StatusServiceUnavailable)
			return
		}
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
