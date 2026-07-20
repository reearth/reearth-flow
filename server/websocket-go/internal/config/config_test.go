package config

import (
	"reflect"
	"strings"
	"testing"
)

// TestDefaultOriginsHaveNoDeadGlobs guards against shipping a default origin that
// ygo can never match: the provider compares AllowedOrigins entries by exact
// (case-insensitive) equality or the literal "*", with no glob/suffix support, so
// an entry like "https://*.reearth.dev" is a dead, misleading allow-list line.
func TestDefaultOriginsHaveNoDeadGlobs(t *testing.T) {
	for _, o := range defaultOrigins {
		if o != "*" && strings.Contains(o, "*") {
			t.Errorf("default origin %q contains a glob but ygo matches origins exactly; it can never match", o)
		}
	}
}

func TestLoadDefaults(t *testing.T) {
	clearEnv(t)
	c := Load()

	if c.RedisURL != "redis://127.0.0.1:6379" {
		t.Errorf("RedisURL = %q", c.RedisURL)
	}
	if c.GCSBucketName != "yrs-dev" {
		t.Errorf("GCSBucketName = %q", c.GCSBucketName)
	}
	if c.GCSEndpoint != "" {
		t.Errorf("GCSEndpoint = %q, want empty (real GCS)", c.GCSEndpoint)
	}
	if c.ThriftAuthURL != "http://localhost:8080" {
		t.Errorf("ThriftAuthURL = %q", c.ThriftAuthURL)
	}
	if c.AppEnv != "development" {
		t.Errorf("AppEnv = %q", c.AppEnv)
	}
	if c.WSPort != 8000 {
		t.Errorf("WSPort = %d, want 8000", c.WSPort)
	}
	if c.APISecret != "" {
		t.Errorf("APISecret = %q, want empty", c.APISecret)
	}
	wantOrigins := []string{
		"http://localhost:3000",
		"https://api.flow.test",
		"http://localhost:8000",
		"http://localhost:8080",
	}
	if !reflect.DeepEqual(c.Origins, wantOrigins) {
		t.Errorf("Origins = %#v\n want %#v", c.Origins, wantOrigins)
	}
	if c.MaxConnections != 10000 {
		t.Errorf("MaxConnections = %d, want 10000", c.MaxConnections)
	}
	if c.MaxPeersPerRoom != 256 {
		t.Errorf("MaxPeersPerRoom = %d, want 256", c.MaxPeersPerRoom)
	}
	if c.MaxRooms != 50000 {
		t.Errorf("MaxRooms = %d, want 50000", c.MaxRooms)
	}
}

func TestLoadOverrides(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_REDIS_URL", "redis://redis:6380")
	t.Setenv("REEARTH_FLOW_GCS_BUCKET_NAME", "prod-bucket")
	t.Setenv("REEARTH_FLOW_GCS_ENDPOINT", "http://fake-gcs:4443")
	t.Setenv("REEARTH_FLOW_THRIFT_AUTH_URL", "https://auth.example.com")
	t.Setenv("REEARTH_FLOW_APP_ENV", "production")
	t.Setenv("REEARTH_FLOW_WS_PORT", "9000")
	t.Setenv("REEARTH_FLOW_API_SECRET", "s3cret")
	t.Setenv("REEARTH_FLOW_ORIGINS", "https://a.com, https://b.com ,, https://c.com")
	t.Setenv("REEARTH_FLOW_MAX_CONNECTIONS", "500")
	t.Setenv("REEARTH_FLOW_MAX_PEERS_PER_ROOM", "32")
	t.Setenv("REEARTH_FLOW_MAX_ROOMS", "1000")

	c := Load()
	if c.RedisURL != "redis://redis:6380" {
		t.Errorf("RedisURL = %q", c.RedisURL)
	}
	if c.GCSBucketName != "prod-bucket" {
		t.Errorf("GCSBucketName = %q", c.GCSBucketName)
	}
	if c.GCSEndpoint != "http://fake-gcs:4443" {
		t.Errorf("GCSEndpoint = %q", c.GCSEndpoint)
	}
	if c.ThriftAuthURL != "https://auth.example.com" {
		t.Errorf("ThriftAuthURL = %q", c.ThriftAuthURL)
	}
	if c.AppEnv != "production" {
		t.Errorf("AppEnv = %q", c.AppEnv)
	}
	if c.WSPort != 9000 {
		t.Errorf("WSPort = %d", c.WSPort)
	}
	if c.APISecret != "s3cret" {
		t.Errorf("APISecret = %q", c.APISecret)
	}
	want := []string{"https://a.com", "https://b.com", "https://c.com"}
	if !reflect.DeepEqual(c.Origins, want) {
		t.Errorf("Origins = %#v, want %#v", c.Origins, want)
	}
	if c.MaxConnections != 500 {
		t.Errorf("MaxConnections = %d, want 500", c.MaxConnections)
	}
	if c.MaxPeersPerRoom != 32 {
		t.Errorf("MaxPeersPerRoom = %d, want 32", c.MaxPeersPerRoom)
	}
	if c.MaxRooms != 1000 {
		t.Errorf("MaxRooms = %d, want 1000", c.MaxRooms)
	}
}

func TestEmptyAPISecretTreatedAsUnset(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_API_SECRET", "")
	if got := Load().APISecret; got != "" {
		t.Errorf("APISecret = %q, want empty", got)
	}
}

func TestInvalidWSPortFallsBackToDefault(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_WS_PORT", "not-a-number")
	if got := Load().WSPort; got != 8000 {
		t.Errorf("WSPort = %d, want 8000 fallback", got)
	}
}

func TestOutOfRangeWSPortFallsBackToDefault(t *testing.T) {
	for _, v := range []string{"0", "-1", "99999", "65536"} {
		t.Run(v, func(t *testing.T) {
			clearEnv(t)
			t.Setenv("REEARTH_FLOW_WS_PORT", v)
			if got := Load().WSPort; got != 8000 {
				t.Errorf("WSPort(%q) = %d, want 8000 fallback", v, got)
			}
		})
	}
}

func TestValidWSPortBoundaries(t *testing.T) {
	for _, tc := range []struct {
		in   string
		want int
	}{
		{"1", 1},
		{"65535", 65535},
	} {
		clearEnv(t)
		t.Setenv("REEARTH_FLOW_WS_PORT", tc.in)
		if got := Load().WSPort; got != tc.want {
			t.Errorf("WSPort(%q) = %d, want %d", tc.in, got, tc.want)
		}
	}
}

func TestInvalidCapsFallBackToDefault(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_MAX_CONNECTIONS", "0")
	t.Setenv("REEARTH_FLOW_MAX_PEERS_PER_ROOM", "-5")
	t.Setenv("REEARTH_FLOW_MAX_ROOMS", "nope")
	c := Load()
	if c.MaxConnections != 10000 {
		t.Errorf("MaxConnections = %d, want 10000", c.MaxConnections)
	}
	if c.MaxPeersPerRoom != 256 {
		t.Errorf("MaxPeersPerRoom = %d, want 256", c.MaxPeersPerRoom)
	}
	if c.MaxRooms != 50000 {
		t.Errorf("MaxRooms = %d, want 50000", c.MaxRooms)
	}
}

func TestOTLPAndAuthDefaults(t *testing.T) {
	clearEnv(t)
	c := Load()
	if c.OTLPEnabled {
		t.Errorf("OTLPEnabled default = true, want false")
	}
	if c.WSAuthEnabled {
		t.Errorf("WSAuthEnabled default = true, want false (parity with auth-OFF Rust prod)")
	}
	if c.OTLPExporterType != "otlp" {
		t.Errorf("OTLPExporterType = %q, want otlp", c.OTLPExporterType)
	}
	if c.OTLPSamplingRatio != 1.0 {
		t.Errorf("OTLPSamplingRatio = %v, want 1.0", c.OTLPSamplingRatio)
	}
	if c.OTLPServiceName != "reearth-flow-websocket" {
		t.Errorf("OTLPServiceName = %q", c.OTLPServiceName)
	}
}

func TestOTLPAndAuthOverrides(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_ENABLE_OTLP", "true")
	t.Setenv("REEARTH_FLOW_OTLP_ENDPOINT", "collector:4317")
	t.Setenv("REEARTH_FLOW_GCP_PROJECT_ID", "my-proj")
	t.Setenv("REEARTH_FLOW_SERVICE_NAME", "ws-svc")
	t.Setenv("REEARTH_FLOW_OTEL_EXPORTER_TYPE", "gcp")
	t.Setenv("REEARTH_FLOW_OTEL_SAMPLING_RATIO", "0.25")
	t.Setenv("REEARTH_FLOW_OTEL_BATCH_TIMEOUT", "2s")
	t.Setenv("REEARTH_FLOW_OTEL_MAX_EXPORT_BATCH_SIZE", "256")
	t.Setenv("REEARTH_FLOW_OTEL_MAX_QUEUE_SIZE", "1024")
	t.Setenv("REEARTH_FLOW_WS_PROTECTED", "true")

	c := Load()
	if !c.OTLPEnabled {
		t.Errorf("OTLPEnabled = false")
	}
	if c.OTLPEndpoint != "collector:4317" {
		t.Errorf("OTLPEndpoint = %q", c.OTLPEndpoint)
	}
	if c.GCPProjectID != "my-proj" {
		t.Errorf("GCPProjectID = %q", c.GCPProjectID)
	}
	if c.OTLPServiceName != "ws-svc" {
		t.Errorf("OTLPServiceName = %q", c.OTLPServiceName)
	}
	if c.OTLPExporterType != "gcp" {
		t.Errorf("OTLPExporterType = %q", c.OTLPExporterType)
	}
	if c.OTLPSamplingRatio != 0.25 {
		t.Errorf("OTLPSamplingRatio = %v", c.OTLPSamplingRatio)
	}
	if c.OTLPBatchTimeout.String() != "2s" {
		t.Errorf("OTLPBatchTimeout = %v", c.OTLPBatchTimeout)
	}
	if c.OTLPMaxExportBatchSize != 256 {
		t.Errorf("OTLPMaxExportBatchSize = %d", c.OTLPMaxExportBatchSize)
	}
	if c.OTLPMaxQueueSize != 1024 {
		t.Errorf("OTLPMaxQueueSize = %d", c.OTLPMaxQueueSize)
	}
	if !c.WSAuthEnabled {
		t.Errorf("WSAuthEnabled = false, want true when REEARTH_FLOW_WS_PROTECTED=true")
	}
}

func TestLoadLoggingDefaults(t *testing.T) {
	clearEnv(t) // app env unset ⇒ development ⇒ text
	c := Load()
	if c.LogLevel != "info" {
		t.Errorf("LogLevel = %q, want info", c.LogLevel)
	}
	if c.LogFormat != "text" {
		t.Errorf("LogFormat = %q, want text (dev default)", c.LogFormat)
	}
}

func TestLoadLoggingProductionDefaultsToJSON(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_APP_ENV", "production")
	if got := Load().LogFormat; got != "json" {
		t.Errorf("LogFormat = %q, want json for a non-dev environment", got)
	}
}

func TestLoadLoggingOverrides(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_LOG_LEVEL", "debug")
	t.Setenv("REEARTH_FLOW_LOG_FORMAT", "text")
	t.Setenv("REEARTH_FLOW_APP_ENV", "production")
	c := Load()
	if c.LogLevel != "debug" {
		t.Errorf("LogLevel = %q, want debug", c.LogLevel)
	}
	if c.LogFormat != "text" {
		t.Errorf("LogFormat = %q, want explicit text override even in production", c.LogFormat)
	}
}

// TestWSProtectedAcceptsCommonSpellings: an operator who writes a reasonable
// truthy value (on/yes/enabled) to enable protected mode must actually get it
// enabled, never a silent fall-back to the insecure default.
func TestWSProtectedAcceptsCommonSpellings(t *testing.T) {
	for _, v := range []string{"on", "yes", "enabled", "ON", "True", "1"} {
		t.Run(v, func(t *testing.T) {
			clearEnv(t)
			t.Setenv("REEARTH_FLOW_WS_PROTECTED", v)
			if !Load().WSAuthEnabled {
				t.Errorf("WSAuthEnabled(%q) = false, want true", v)
			}
		})
	}
	for _, v := range []string{"off", "no", "disabled", "false", "0"} {
		t.Run(v, func(t *testing.T) {
			clearEnv(t)
			t.Setenv("REEARTH_FLOW_WS_PROTECTED", v)
			if Load().WSAuthEnabled {
				t.Errorf("WSAuthEnabled(%q) = true, want false", v)
			}
		})
	}
}

// TestValidateRejectsUnparseableWSProtected: a genuinely unparseable value for a
// security-gating toggle must fail startup rather than silently disable the
// control (fail-open on misconfig).
func TestValidateRejectsUnparseableWSProtected(t *testing.T) {
	clearEnv(t)
	t.Setenv("REEARTH_FLOW_WS_PROTECTED", "maybe")
	if err := Load().Validate(); err == nil {
		t.Fatalf("Validate() = nil, want error for unparseable REEARTH_FLOW_WS_PROTECTED")
	}
}

// TestValidateAcceptsValidAndEmpty: valid and unset values must pass validation.
func TestValidateAcceptsValidAndEmpty(t *testing.T) {
	for _, v := range []string{"", "true", "false", "on", "no"} {
		clearEnv(t)
		if v != "" {
			t.Setenv("REEARTH_FLOW_WS_PROTECTED", v)
		}
		if err := Load().Validate(); err != nil {
			t.Errorf("Validate() with WS_PROTECTED=%q = %v, want nil", v, err)
		}
	}
}

// clearEnv unsets every env var Load reads so a test starts from a clean slate.
func clearEnv(t *testing.T) {
	t.Helper()
	for _, k := range []string{
		"REEARTH_FLOW_LOG_LEVEL",
		"REEARTH_FLOW_LOG_FORMAT",
		"REEARTH_FLOW_REDIS_URL",
		"REEARTH_FLOW_GCS_BUCKET_NAME",
		"REEARTH_FLOW_GCS_ENDPOINT",
		"REEARTH_FLOW_THRIFT_AUTH_URL",
		"REEARTH_FLOW_APP_ENV",
		"REEARTH_FLOW_ORIGINS",
		"REEARTH_FLOW_WS_PORT",
		"REEARTH_FLOW_API_SECRET",
		"REEARTH_FLOW_MAX_CONNECTIONS",
		"REEARTH_FLOW_MAX_PEERS_PER_ROOM",
		"REEARTH_FLOW_MAX_ROOMS",
		"REEARTH_FLOW_PEER_WRITE_QUEUE_SIZE",
		"REEARTH_FLOW_ENABLE_OTLP",
		"REEARTH_FLOW_OTLP_ENDPOINT",
		"REEARTH_FLOW_GCP_PROJECT_ID",
		"REEARTH_FLOW_SERVICE_NAME",
		"REEARTH_FLOW_OTEL_EXPORTER_TYPE",
		"REEARTH_FLOW_OTEL_SAMPLING_RATIO",
		"REEARTH_FLOW_OTEL_BATCH_TIMEOUT",
		"REEARTH_FLOW_OTEL_MAX_EXPORT_BATCH_SIZE",
		"REEARTH_FLOW_OTEL_MAX_QUEUE_SIZE",
		"REEARTH_FLOW_WS_PROTECTED",
	} {
		t.Setenv(k, "")
	}
}

func TestLoad_PeerWriteQueueSize(t *testing.T) {
	t.Run("default is 512", func(t *testing.T) {
		clearEnv(t)
		if c := Load(); c.PeerWriteQueueSize != 512 {
			t.Errorf("PeerWriteQueueSize = %d, want 512", c.PeerWriteQueueSize)
		}
	})
	t.Run("env override", func(t *testing.T) {
		clearEnv(t)
		t.Setenv("REEARTH_FLOW_PEER_WRITE_QUEUE_SIZE", "1024")
		if c := Load(); c.PeerWriteQueueSize != 1024 {
			t.Errorf("PeerWriteQueueSize = %d, want 1024", c.PeerWriteQueueSize)
		}
	})
	t.Run("non-positive falls back to default", func(t *testing.T) {
		clearEnv(t)
		t.Setenv("REEARTH_FLOW_PEER_WRITE_QUEUE_SIZE", "-5")
		if c := Load(); c.PeerWriteQueueSize != 512 {
			t.Errorf("PeerWriteQueueSize = %d, want 512", c.PeerWriteQueueSize)
		}
	})
}
