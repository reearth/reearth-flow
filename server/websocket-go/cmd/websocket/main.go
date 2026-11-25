// Command websocket is the reearth-flow Y-WebSocket server entrypoint: it loads
// config, builds the WS transport plus HTTP surface, and serves with graceful
// shutdown on SIGINT/SIGTERM.
package main

import (
	"context"
	crand "crypto/rand"
	"errors"
	"fmt"
	"log/slog"
	"net"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"cloud.google.com/go/storage"
	"google.golang.org/api/option"

	"github.com/reearth/reearth-flow/websocket-go/internal/auth"
	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	"github.com/reearth/reearth-flow/websocket-go/internal/coord"
	"github.com/reearth/reearth-flow/websocket-go/internal/gcs"
	"github.com/reearth/reearth-flow/websocket-go/internal/health"
	flowhttp "github.com/reearth/reearth-flow/websocket-go/internal/http"
	"github.com/reearth/reearth-flow/websocket-go/internal/logging"
	flowotel "github.com/reearth/reearth-flow/websocket-go/internal/otel"
	"github.com/reearth/reearth-flow/websocket-go/internal/server"
	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

func main() {
	if err := run(); err != nil {
		slog.Error("server exited with error", "err", err)
		os.Exit(1)
	}
}

func run() error {
	cfg := config.Load()
	// Build the configured logger and make it the process default so even
	// components that fall back to slog.Default() honor the level/format.
	log := logging.New(cfg.LogLevel, cfg.LogFormat, os.Stderr)
	slog.SetDefault(log)
	log.Info("logger configured", "level", cfg.LogLevel, "format", cfg.LogFormat)

	// Fail fast on a misconfigured security toggle rather than silently
	// disabling it.
	if err := cfg.Validate(); err != nil {
		return fmt.Errorf("invalid configuration: %w", err)
	}

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	log.Info("coordination backend selected", "backend", coord.Describe(cfg))

	// Build GCS persistence and the coordination lock store. Fatal on failure:
	// persistence is load-bearing for durability across restarts.
	p, err := buildPersistence(ctx, cfg)
	if err != nil {
		return fmt.Errorf("build persistence: %w", err)
	}
	defer p.close()
	srv, adapter, flusher := p.srv, p.adapter, p.flusher

	// OTLP tracing; disabled by default (noop provider).
	tp, err := flowotel.InitTracer(ctx, otelConfig(cfg))
	if err != nil {
		return fmt.Errorf("init tracer: %w", err)
	}
	defer func() {
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := tp.Shutdown(shutdownCtx); err != nil {
			log.Warn("otel shutdown", "err", err)
		}
	}()

	// Every connection is verified against the flow API before the upgrade.
	srv.WSProvider().Authorize = auth.NewAuthorize(auth.Config{
		AuthURL: cfg.ThriftAuthURL,
	})

	// HTTP document API behind X-API-Secret; fails startup if a non-dev env has no secret.
	apiSecret, err := flowhttp.RequireAPISecret(flowhttp.APISecretConfig{
		Secret: cfg.APISecret,
		AppEnv: cfg.AppEnv,
	})
	if err != nil {
		return fmt.Errorf("api secret guard: %w", err)
	}
	store := flowhttp.NewStoreAdapter(flowhttp.StoreAdapterDeps{
		P:         adapter,
		FlushFn:   flusher.FlushRoom,
		ListRooms: srv.WSProvider().Rooms,
		Logger:    log,
	})
	apiRouter := flowhttp.NewRouter(flowhttp.Deps{
		Store:     store,
		Signaler:  srv,
		Versioner: srv,
		Logger:    log,
	})

	// Wire the real /health probes. A client-construction failure is non-fatal:
	// /health reports that component unconfigured (503) instead.
	wireHealth(ctx, srv, cfg, p.locks, log)

	// Versions are saved from user edits, recorded in the coordination store so
	// every instance shares one record per room, and the API hears of each.
	notifier, err := enableVersions(srv, cfg, p, log)
	if err != nil {
		return err
	}

	// Attach the cluster relay so instances share rooms. The GCS Flusher is the
	// relay's last-instance persistence seam; wrapped, it first saves the room
	// as a version when users changed it. Fatal on failure.
	relay, err := coord.NewRelay(cfg, p.locks, srv.VersionedFlusher(flusher), log)
	if err != nil {
		return fmt.Errorf("build cluster relay: %w", err)
	}
	defer func() { _ = relay.Close() }()
	if err := srv.AttachRedisRelay(ctx, relay); err != nil {
		return fmt.Errorf("attach cluster relay: %w", err)
	}
	// A project lock change or a version request must reach a room open only on
	// another instance.
	if p := coord.Presence(relay); p != nil {
		srv.SetPresence(p)
	}
	if c := coord.CatchUp(relay); c != nil {
		srv.SetCatchUp(c)
	}

	go srv.StartPeriodicSync(ctx, 0) // 0 => default 30s
	go srv.StartPeriodicVersions(ctx)

	// Retention sweep for backends without key TTLs; a no-op for the others.
	coord.StartJanitor(ctx, cfg, p.locks, log)

	// Compose WS + /health + /api/*, then wrap for request spans. Span attributes
	// must never carry tokens, secrets, or payloads.
	mux := srv.HandlerWithAPI(apiSecret(apiRouter))
	handler := flowotel.WrapHandler(mux, flowotel.WrapOptions{
		TracerProvider: tp,
		SpanName:       "ws.http",
	})
	// Outermost: structured access logging + panic recovery across every route
	// (WS upgrade, /api/*, /health), so no request failure is silent. WS-safe.
	handler = flowhttp.ObserveRequests(log)(handler)

	// Bind the configured WS port (8000), not $PORT.
	addr := net.JoinHostPort("0.0.0.0", fmt.Sprintf("%d", cfg.WSPort))
	httpSrv := &http.Server{
		Addr:    addr,
		Handler: handler,
	}

	errCh := make(chan error, 1)
	go func() {
		log.Info("websocket server listening", "addr", addr, "env", cfg.AppEnv)
		if err := httpSrv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			errCh <- err
		}
	}()

	select {
	case err := <-errCh:
		return err
	case <-ctx.Done():
		log.Info("shutdown signal received, draining")
	}

	shutdownCtx, cancel := context.WithTimeout(context.Background(), shutdownTimeout)
	defer cancel()
	// The saves get the drain's budget less what the notices need, so the
	// notices of versions saved late in the drain still go out.
	savesCtx, cancelSaves := context.WithTimeout(shutdownCtx, shutdownTimeout-noticeReserve)
	defer cancelSaves()
	// Drain WebSocket peers first, then stop accepting HTTP. Rooms the drain
	// releases are saved as versions on the way out, so let those saves, and
	// any other save in progress, finish, and then their API notices.
	if err := srv.Shutdown(savesCtx); err != nil {
		log.Warn("ws provider shutdown", "err", err)
	}
	err = httpSrv.Shutdown(savesCtx)
	if werr := srv.WaitVersions(savesCtx); werr != nil {
		log.Warn("versions still saving at shutdown", "err", werr)
	}
	if werr := notifier.Shutdown(shutdownCtx); werr != nil {
		log.Warn("version notices abandoned at shutdown", "err", werr)
	}
	return err
}

const (
	// shutdownTimeout is the drain's whole budget.
	shutdownTimeout = 30 * time.Second
	// noticeReserve is kept from the drain's budget for the API notices of the
	// versions saved during it.
	noticeReserve = 5 * time.Second
	// versionLockWait bounds the wait for another save of the same room to
	// finish. A save can wait catchUpTimeout (10s) for the relay and then read
	// and write whole documents in GCS, so the lock store's own 5s budget is too
	// short.
	versionLockWait = time.Minute
)

// enableVersions wires versioning into srv and returns the notifier that tells
// the API about each version.
func enableVersions(srv *server.Server, cfg *config.Config, p *persistence, log *slog.Logger) (*versions.Notifier, error) {
	changes, err := coord.NewChangeStore(cfg, p.locks)
	if err != nil {
		return nil, fmt.Errorf("version change store: %w", err)
	}
	notifier := versions.NewNotifier(versions.NotifierConfig{
		APIURL: cfg.ThriftAuthURL,
		Secret: cfg.APISecret,
		Log:    log,
	})
	srv.EnableVersions(server.VersionsConfig{
		Store:     changes,
		Snapshots: p.adapter.Versions(cfg.KeepSnapshots),
		// One save per room at a time across instances, so a slower save can
		// never store an older state after a newer one.
		Lock: func(ctx context.Context, room string, fn func(context.Context) error) error {
			return lockWithin(ctx, p.locks.Locker, versionLockKey(room), versionLockWait, fn)
		},
		// The checkpoint is written from the room's current state, never older
		// than what a concurrent flush already wrote.
		Checkpoint: p.flusher.FlushRoom,
		Stored: func(ctx context.Context, room string) ([]byte, error) {
			res, err := p.adapter.Load(ctx, room)
			return res.Update, err
		},
		Notify: notifier.Notify,
		Every:  cfg.AutoVersionEvery,
	})
	return notifier, nil
}

// versionLockKey is the lock serializing a room's version saves.
func versionLockKey(room string) string { return "lock:version:" + room }

// lockWithin runs fn under key, retrying the lock store's bounded acquisition
// until wait has passed. It then returns versions.ErrBusy. An error fn returns
// is returned as is, never retried, even when it is a lock timeout of fn's own.
func lockWithin(ctx context.Context, l gcs.Locker, key string, wait time.Duration, fn func(context.Context) error) error {
	deadline := time.Now().Add(wait)
	for {
		ran := false
		err := l.WithLock(ctx, key, func(ctx context.Context) error {
			ran = true
			return fn(ctx)
		})
		if ran || !coord.IsLockTimeout(err) {
			return err
		}
		if ctx.Err() != nil || time.Now().After(deadline) {
			return fmt.Errorf("%w: %w", versions.ErrBusy, err)
		}
	}
}

// newInstanceOwner returns a per-process lock-owner token unique across
// instances even when they share a PID (e.g. PID 1 in separate containers).
// The crypto/rand suffix guarantees uniqueness; the hostname prefix is only for
// operator-facing readability of lock values.
func newInstanceOwner() (string, error) {
	var b [8]byte
	if _, err := crand.Read(b[:]); err != nil {
		return "", fmt.Errorf("generate instance owner: %w", err)
	}
	host, _ := os.Hostname()
	return fmt.Sprintf("instance-%s-%x", host, b[:]), nil
}

// persistence groups what buildPersistence returns. A struct rather than a tuple
// because the lock store joined the list and six unnamed returns stop being
// readable at the call site.
type persistence struct {
	srv     *server.Server
	adapter *gcs.Adapter
	flusher *gcs.Flusher
	locks   *coord.Locks
	close   func()
}

// buildPersistence constructs the coordination lock store, the GCS adapter, the
// persistence-wired Server and the last-instance Flusher, plus a close that
// releases the GCS and lock-store clients. Phase-2 is opt-in via
// REEARTH_FLOW_GCS_PHASE2 (default OFF).
func buildPersistence(ctx context.Context, cfg *config.Config) (*persistence, error) {
	// GCS client (anonymous against fake-gcs when REEARTH_FLOW_GCS_ENDPOINT is set).
	var gcsOpts []option.ClientOption
	if cfg.GCSEndpoint != "" {
		gcsOpts = append(gcsOpts, option.WithEndpoint(cfg.GCSEndpoint), option.WithoutAuthentication())
	}
	stClient, err := storage.NewClient(ctx, gcsOpts...)
	if err != nil {
		return nil, fmt.Errorf("gcs client: %w", err)
	}

	owner, err := newInstanceOwner()
	if err != nil {
		_ = stClient.Close()
		return nil, fmt.Errorf("instance owner: %w", err)
	}

	// The configured backend's lock store, shared by OID allocation and the
	// flusher's read:lock.
	locks, err := coord.NewLocker(cfg, owner)
	if err != nil {
		_ = stClient.Close()
		return nil, fmt.Errorf("coordination locks: %w", err)
	}

	adapter, err := gcs.New(gcs.Options{
		Client:           stClient,
		Bucket:           cfg.GCSBucketName,
		Locker:           locks.Locker,
		Phase2:           os.Getenv("REEARTH_FLOW_GCS_PHASE2") == "true",
		TransientMapKeys: server.TransientMapKeys(),
		KeepSnapshots:    cfg.KeepSnapshots,
	})
	if err != nil {
		_ = stClient.Close()
		_ = locks.Close()
		return nil, fmt.Errorf("gcs adapter: %w", err)
	}

	srv := server.NewWithPersistence(ctx, cfg, adapter)

	// The flusher persists the room's live doc state on last-instance eviction.
	flusher := gcs.NewFlusher(gcs.FlusherOptions{
		Adapter: adapter,
		Locker:  locks.Locker,
		StateOf: srv.CheckpointState,
	})

	return &persistence{
		srv:     srv,
		adapter: adapter,
		flusher: flusher,
		locks:   locks,
		close:   func() { _ = stClient.Close(); _ = locks.Close() },
	}, nil
}

// wireHealth attaches the coordination-backend + GCS probes; a nil probe is
// reported unconfigured (503). The backend probe comes from the lock store rather
// than being rebuilt here, so /health cannot end up probing a different store
// than the service coordinates through.
func wireHealth(ctx context.Context, srv *server.Server, cfg *config.Config, locks *coord.Locks, log *slog.Logger) {
	var p server.PingerFunc
	if locks.Probe != nil {
		p = server.PingerFunc(locks.Probe)
	} else {
		log.Warn("coordination health probe unavailable", "backend", coord.Describe(cfg))
	}

	var l server.ListerFunc
	if lister, err := health.NewGCSLister(ctx, cfg.GCSBucketName, cfg.GCSEndpoint); err != nil {
		log.Warn("gcs health probe unavailable", "err", err)
	} else {
		l = lister.List
	}

	srv.SetHealthChecks(p, l, coord.Describe(cfg))
}

// otelConfig maps the service config onto the otel package's Config.
func otelConfig(cfg *config.Config) flowotel.Config {
	return flowotel.Config{
		Enabled:            cfg.OTLPEnabled,
		Endpoint:           cfg.OTLPEndpoint,
		ExporterType:       flowotel.ExporterType(cfg.OTLPExporterType),
		GCPProjectID:       cfg.GCPProjectID,
		ServiceName:        cfg.OTLPServiceName,
		MaxExportBatchSize: cfg.OTLPMaxExportBatchSize,
		BatchTimeout:       cfg.OTLPBatchTimeout,
		MaxQueueSize:       cfg.OTLPMaxQueueSize,
		SamplingRatio:      cfg.OTLPSamplingRatio,
	}
}
