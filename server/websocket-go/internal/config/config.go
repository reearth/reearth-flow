// Package config loads the websocket-go service configuration from the
// REEARTH_FLOW_* environment variables.
package config

import (
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

// Config is the resolved service configuration.
type Config struct {
	// CoordBackend selects the cluster coordination store. See the Backend*
	// constants.
	CoordBackend string
	// PGURL is the postgres:// DSN for the postgres backend. Required when
	// CoordBackend is postgres, ignored otherwise.
	PGURL string
	// PGFanout selects the postgres read strategy. See the Fanout* constants.
	PGFanout string
	// PGPollInterval is the poll period for the poll and hybrid fan-outs. Unused
	// by notify.
	PGPollInterval time.Duration

	// RedisURL is the Redis Streams fan-out / locks / heartbeat endpoint. Unused
	// unless CoordBackend is redis.
	RedisURL string
	// GCSBucketName is the GCS persistence bucket.
	GCSBucketName string
	// GCSEndpoint overrides the GCS endpoint (fake-gcs in dev). Empty ⇒ real GCS.
	GCSEndpoint string
	// ThriftAuthURL is the base URL for POST /auth/verify.
	ThriftAuthURL string
	// AppEnv is the environment label (development/production).
	AppEnv string
	// LogLevel is the slog verbosity: debug/info/warn/error. Default info.
	LogLevel string
	// LogFormat is the slog handler format: json or text. Defaults to json in a
	// non-dev environment (for Cloud Run log ingestion) and text in development.
	LogFormat string
	// Origins is the CORS / WebSocket allow-list (comma-split, trimmed).
	Origins []string
	// WSPort is the listen port (default 8000), not Cloud Run's $PORT.
	WSPort int
	// APISecret is the X-API-Secret shared secret for the HTTP doc API.
	// Empty ⇒ allow-all (treated as unset).
	APISecret string

	// MaxConnections caps simultaneous WebSocket peers server-wide.
	// Finite by default: ygo treats 0 as unlimited, so we never pass 0.
	MaxConnections int
	// MaxPeersPerRoom caps simultaneous WebSocket peers per room.
	MaxPeersPerRoom int
	// MaxRooms caps distinct rooms server-wide (doc_id is client-supplied).
	MaxRooms int

	// WSAuthEnabled gates protected-mode WS token verification. Default OFF;
	// when ON the AuthFunc fails closed. Sourced from REEARTH_FLOW_WS_PROTECTED.
	WSAuthEnabled bool

	// SlowPeerResync keeps slow peers connected and re-syncs them in place on
	// queue overflow instead of disconnecting. Default ON; env REEARTH_FLOW_SLOW_PEER_RESYNC.
	SlowPeerResync bool

	// AutoVersionEvery caps how often ygo snapshots a changed room. 0 disables.
	AutoVersionEvery time.Duration

	// KeepSnapshots bounds retained snapshots per room. Applies on the auto path
	// only and ignores labels, so named snapshots are also evicted: reearth/ygo#212.
	KeepSnapshots int

	// OTLP tracing config.
	OTLPEnabled            bool
	OTLPEndpoint           string
	GCPProjectID           string
	OTLPServiceName        string
	OTLPExporterType       string
	OTLPSamplingRatio      float64
	OTLPBatchTimeout       time.Duration
	OTLPMaxExportBatchSize int
	OTLPMaxQueueSize       int
}

// Defaults.
const (
	// Coordination backends. Memory is single-instance only; Redis is the only one compatible with the Rust server.
	BackendRedis    = "redis"
	BackendPostgres = "postgres"
	BackendMemory   = "memory"

	// Postgres fan-out: how an instance learns another wrote a row. Notify needs a session-pooled or direct connection.
	FanoutPoll   = "poll"
	FanoutNotify = "notify"
	FanoutHybrid = "hybrid"

	defaultCoordBackend   = BackendRedis
	defaultPGFanout       = FanoutHybrid
	defaultPGPollInterval = 50 * time.Millisecond
	defaultPGHybridPoll   = time.Second
	minPGPollInterval     = 5 * time.Millisecond

	defaultRedisURL      = "redis://127.0.0.1:6379"
	defaultGCSBucketName = "yrs-dev"
	defaultThriftAuthURL = "http://localhost:8080"
	defaultAppEnv        = "development"
	defaultLogLevel      = "info"
	defaultWSPort        = 8000

	defaultMaxConnections  = 10000
	defaultMaxPeersPerRoom = 256
	defaultMaxRooms        = 50000

	defaultAutoVersionEvery = 15 * time.Minute
	defaultKeepSnapshots    = 50

	defaultOTLPExporterType       = "otlp"
	defaultOTLPServiceName        = "reearth-flow-websocket"
	defaultOTLPSamplingRatio      = 1.0
	defaultOTLPBatchTimeout       = time.Second
	defaultOTLPMaxExportBatchSize = 512
	defaultOTLPMaxQueueSize       = 2048
)

// defaultOrigins is the default CORS / WebSocket allow-list for local
// development. Entries are matched by the ygo provider case-insensitively and
// EXACTLY (or the literal "*"); it has no glob/suffix support, so every entry
// must be a concrete origin. Production must set REEARTH_FLOW_ORIGINS to the
// exact origins it serves (e.g. https://app.reearth.dev), one per comma.
var defaultOrigins = []string{
	"http://localhost:3000",
	"https://api.flow.test",
	"http://localhost:8000",
	"http://localhost:8080",
}

// Load reads configuration from the environment
func Load() *Config {
	appEnv := envOr("REEARTH_FLOW_APP_ENV", defaultAppEnv)
	fanout := envEnum("REEARTH_FLOW_PG_FANOUT", defaultPGFanout)
	return &Config{
		CoordBackend:   envEnum("REEARTH_FLOW_COORD_BACKEND", defaultCoordBackend),
		PGURL:          os.Getenv("REEARTH_FLOW_PG_URL"),
		PGFanout:       fanout,
		PGPollInterval: envDuration("REEARTH_FLOW_PG_POLL_INTERVAL", defaultPollFor(fanout)),

		RedisURL:      envOr("REEARTH_FLOW_REDIS_URL", defaultRedisURL),
		GCSBucketName: envOr("REEARTH_FLOW_GCS_BUCKET_NAME", defaultGCSBucketName),
		GCSEndpoint:   os.Getenv("REEARTH_FLOW_GCS_ENDPOINT"),
		ThriftAuthURL: envOr("REEARTH_FLOW_THRIFT_AUTH_URL", defaultThriftAuthURL),
		AppEnv:        appEnv,
		LogLevel:      envOr("REEARTH_FLOW_LOG_LEVEL", defaultLogLevel),
		LogFormat:     envOr("REEARTH_FLOW_LOG_FORMAT", defaultLogFormat(appEnv)),
		Origins:       origins(os.Getenv("REEARTH_FLOW_ORIGINS")),
		WSPort:        envPort("REEARTH_FLOW_WS_PORT", defaultWSPort),
		APISecret:     os.Getenv("REEARTH_FLOW_API_SECRET"),

		MaxConnections:  envPositive("REEARTH_FLOW_MAX_CONNECTIONS", defaultMaxConnections),
		MaxPeersPerRoom: envPositive("REEARTH_FLOW_MAX_PEERS_PER_ROOM", defaultMaxPeersPerRoom),
		MaxRooms:        envPositive("REEARTH_FLOW_MAX_ROOMS", defaultMaxRooms),

		WSAuthEnabled: envBool("REEARTH_FLOW_WS_PROTECTED", false),

		SlowPeerResync: envBool("REEARTH_FLOW_SLOW_PEER_RESYNC", true),

		AutoVersionEvery: envDuration("REEARTH_FLOW_AUTO_VERSION_EVERY", defaultAutoVersionEvery),
		KeepSnapshots:    envPositive("REEARTH_FLOW_KEEP_SNAPSHOTS", defaultKeepSnapshots),

		OTLPEnabled:            envBool("REEARTH_FLOW_ENABLE_OTLP", false),
		OTLPEndpoint:           os.Getenv("REEARTH_FLOW_OTLP_ENDPOINT"),
		GCPProjectID:           os.Getenv("REEARTH_FLOW_GCP_PROJECT_ID"),
		OTLPServiceName:        envOr("REEARTH_FLOW_SERVICE_NAME", defaultOTLPServiceName),
		OTLPExporterType:       envOr("REEARTH_FLOW_OTEL_EXPORTER_TYPE", defaultOTLPExporterType),
		OTLPSamplingRatio:      envFloat("REEARTH_FLOW_OTEL_SAMPLING_RATIO", defaultOTLPSamplingRatio),
		OTLPBatchTimeout:       envDuration("REEARTH_FLOW_OTEL_BATCH_TIMEOUT", defaultOTLPBatchTimeout),
		OTLPMaxExportBatchSize: envPositive("REEARTH_FLOW_OTEL_MAX_EXPORT_BATCH_SIZE", defaultOTLPMaxExportBatchSize),
		OTLPMaxQueueSize:       envPositive("REEARTH_FLOW_OTEL_MAX_QUEUE_SIZE", defaultOTLPMaxQueueSize),
	}
}

// Validate reports configuration errors that must fail startup. Security-gating
// toggles must never silently fall back to an insecure default on a typo, so an
// explicitly-set but unparseable value is a hard error rather than a quiet
// disable (fail-open on misconfig).
func (c *Config) Validate() error {
	if raw := os.Getenv("REEARTH_FLOW_WS_PROTECTED"); strings.TrimSpace(raw) != "" {
		if _, ok := parseBool(raw); !ok {
			return fmt.Errorf("REEARTH_FLOW_WS_PROTECTED=%q is not a valid boolean (use true/false, on/off, yes/no); refusing to start with an ambiguous WS auth setting", raw)
		}
	}
	// Auto-versioning is ON by default and envDuration silently falls back, so an
	// unparseable kill switch would leave it running. Use "0" to disable.
	if raw := os.Getenv("REEARTH_FLOW_AUTO_VERSION_EVERY"); strings.TrimSpace(raw) != "" {
		d, err := time.ParseDuration(strings.TrimSpace(raw))
		if err != nil {
			return fmt.Errorf("REEARTH_FLOW_AUTO_VERSION_EVERY=%q is not a valid Go duration (e.g. 15m, 30s; use 0 to disable auto-versioning); refusing to start rather than silently keeping auto-versioning enabled", raw)
		}
		// ygo treats <= 0 as disabled, but "0" is the documented spelling, so a
		// negative value is more likely a typo than an intent.
		if d < 0 {
			return fmt.Errorf("REEARTH_FLOW_AUTO_VERSION_EVERY=%q is negative; use exactly 0 to disable auto-versioning, or a positive duration such as 15m", raw)
		}
	}
	// The coordination backend decides which store carries cross-instance document
	// traffic. A typo must not silently fall back to redis: an operator who meant
	// postgres would get a service that still needs the Redis they were removing,
	// and one who meant memory would get a second store they are paying for.
	switch c.CoordBackend {
	case BackendRedis, BackendMemory:
	case BackendPostgres:
		if strings.TrimSpace(c.PGURL) == "" {
			return fmt.Errorf("REEARTH_FLOW_COORD_BACKEND=%s requires REEARTH_FLOW_PG_URL; refusing to start without a database to coordinate through", BackendPostgres)
		}
		switch c.PGFanout {
		case FanoutPoll, FanoutNotify, FanoutHybrid:
		default:
			return fmt.Errorf("REEARTH_FLOW_PG_FANOUT=%q is not one of %s/%s/%s", c.PGFanout, FanoutPoll, FanoutNotify, FanoutHybrid)
		}
		// Only the polling fan-outs consult the interval, so a bad value is
		// harmless under notify and rejecting it there would be noise.
		if c.PGFanout != FanoutNotify {
			if raw := os.Getenv("REEARTH_FLOW_PG_POLL_INTERVAL"); strings.TrimSpace(raw) != "" {
				if _, err := time.ParseDuration(strings.TrimSpace(raw)); err != nil {
					return fmt.Errorf("REEARTH_FLOW_PG_POLL_INTERVAL=%q is not a valid Go duration (e.g. 50ms, 1s)", raw)
				}
			}
			if c.PGPollInterval < minPGPollInterval {
				return fmt.Errorf("REEARTH_FLOW_PG_POLL_INTERVAL=%s is below the %s floor; a shorter period queries faster than it waits", c.PGPollInterval, minPGPollInterval)
			}
		}
	default:
		return fmt.Errorf("REEARTH_FLOW_COORD_BACKEND=%q is not one of %s/%s/%s", c.CoordBackend, BackendRedis, BackendPostgres, BackendMemory)
	}
	return nil
}

func defaultPollFor(fanout string) time.Duration {
	if fanout == FanoutHybrid {
		return defaultPGHybridPoll
	}
	return defaultPGPollInterval
}

// envEnum reads an enum-valued setting, lowercased and trimmed so "Postgres " and
// "postgres" select the same backend.
func envEnum(key, def string) string {
	return strings.ToLower(strings.TrimSpace(envOr(key, def)))
}

func defaultLogFormat(appEnv string) string {
	if isDevEnv(appEnv) {
		return "text"
	}
	return "json"
}

// isDevEnv reports whether appEnv names a development-like environment.
func isDevEnv(appEnv string) bool {
	switch strings.ToLower(strings.TrimSpace(appEnv)) {
	case "", "development", "dev", "local", "test":
		return true
	default:
		return false
	}
}

func parseBool(v string) (val bool, ok bool) {
	switch strings.ToLower(strings.TrimSpace(v)) {
	case "1", "t", "true", "on", "yes", "y", "enabled":
		return true, true
	case "0", "f", "false", "off", "no", "n", "disabled":
		return false, true
	default:
		return false, false
	}
}

// envBool parses a boolean, falling back to def when unset/empty/unparseable.
func envBool(key string, def bool) bool {
	v := os.Getenv(key)
	if v == "" {
		return def
	}
	if b, ok := parseBool(v); ok {
		return b
	}
	return def
}

// envFloat parses a float, falling back to def when unset/empty/unparseable.
func envFloat(key string, def float64) float64 {
	v := os.Getenv(key)
	if v == "" {
		return def
	}
	f, err := strconv.ParseFloat(strings.TrimSpace(v), 64)
	if err != nil {
		return def
	}
	return f
}

// envDuration parses a Go duration, falling back to def when unset/empty/unparseable.
func envDuration(key string, def time.Duration) time.Duration {
	v := os.Getenv(key)
	if v == "" {
		return def
	}
	d, err := time.ParseDuration(strings.TrimSpace(v))
	if err != nil {
		return def
	}
	return d
}

// envOr returns the env var or def when it is unset/empty.
func envOr(key, def string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return def
}

func envPort(key string, def int) int {
	v := os.Getenv(key)
	if v == "" {
		return def
	}
	n, err := strconv.Atoi(v)
	if err != nil || n < 1 || n > 65535 {
		return def
	}
	return n
}

// envPositive parses a positive integer, falling back to def when unset, empty,
// unparseable, or non-positive (0 must never reach ygo's "unlimited" caps).
func envPositive(key string, def int) int {
	v := os.Getenv(key)
	if v == "" {
		return def
	}
	n, err := strconv.Atoi(v)
	if err != nil || n < 1 {
		return def
	}
	return n
}

// origins parses a comma-separated origin list (trim entries, drop empties)
func origins(raw string) []string {
	if raw == "" {
		out := make([]string, len(defaultOrigins))
		copy(out, defaultOrigins)
		return out
	}
	var out []string
	for _, part := range strings.Split(raw, ",") {
		if p := strings.TrimSpace(part); p != "" {
			out = append(out, p)
		}
	}
	return out
}
