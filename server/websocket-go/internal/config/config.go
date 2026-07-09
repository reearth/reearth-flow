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
	// RedisURL is the Redis Streams fan-out / locks / heartbeat endpoint.
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
	defaultRedisURL      = "redis://127.0.0.1:6379"
	defaultGCSBucketName = "yrs-dev"
	defaultThriftAuthURL = "http://localhost:8080"
	defaultAppEnv        = "development"
	defaultLogLevel      = "info"
	defaultWSPort        = 8000

	defaultMaxConnections  = 10000
	defaultMaxPeersPerRoom = 256
	defaultMaxRooms        = 50000

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

// Load reads configuration from the environment, applying defaults for any
// unset (or empty) variable.
func Load() *Config {
	appEnv := envOr("REEARTH_FLOW_APP_ENV", defaultAppEnv)
	return &Config{
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
	return nil
}

// defaultLogFormat chooses structured JSON for non-dev environments (so Cloud
// Run ingests structured logs) and human-readable text for local development.
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

// parseBool recognizes the strconv.ParseBool set plus the common operator
// spellings on/off, yes/no, y/n, and enabled/disabled. ok is false for a
// non-empty value that matches none of these, so a security-gating toggle can
// detect a typo instead of silently falling back to an insecure default.
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

// envPort parses a TCP port, falling back to def when unset, empty,
// unparseable, or outside 1..65535 (rejecting 0 avoids a random-ephemeral bind).
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

// origins parses a comma-separated origin list (trim entries, drop empties);
// an empty/unset value yields the default list.
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
