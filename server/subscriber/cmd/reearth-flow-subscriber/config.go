package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/joho/godotenv"
	"github.com/k0kubun/pp/v3"
	"github.com/kelseyhightower/envconfig"
	"github.com/reearth/reearthx/log"
)

const configPrefix = "REEARTH_FLOW_SUBSCRIBER"

func init() {
	pp.Default.SetColoringEnabled(false)
}

type Config struct {
	AssetBaseURL string `envconfig:"ASSET_BASE_URL" default:"http://localhost:8080/assets"`
	DB           string `default:"mongodb://localhost"`
	// DBPG is the API's Postgres database, required when LiveBackend is postgres.
	DBPG string `envconfig:"DB_PG" pp:",omitempty"`
	Dev  bool   `pp:",omitempty"`
	// No default: a defaulted value would crash-loop the whole subscriber.
	DiagnosticSubscriptionID  string `envconfig:"DIAGNOSTIC_SUBSCRIPTION_ID" default:""`
	GCPProject                string `envconfig:"GOOGLE_CLOUD_PROJECT" pp:",omitempty"`
	GCSBucket                 string `envconfig:"GCS_BUCKET" pp:",omitempty"`
	JobCompleteSubscriptionID string `envconfig:"JOB_COMPLETE_SUBSCRIPTION_ID" default:"flow-job-complete-main"`
	// LiveBackend selects where job logs, user-facing logs, diagnostics and
	// job-complete events are written: "redis" (RedisURL) or "postgres" (DBPG).
	LiveBackend                 string `envconfig:"LIVE_BACKEND" default:"redis"`
	LogSubscriptionID           string `envconfig:"LOG_SUBSCRIPTION_ID" default:"flow-log-stream-main"`
	Port                        string `envconfig:"PORT" default:"8080"`
	RedisURL                    string `envconfig:"REDIS_URL" default:"redis://localhost:6379"`
	UserFacingLogSubscriptionID string `envconfig:"USER_FACING_LOG_SUBSCRIPTION_ID" default:"flow-user-facing-log-main"`

	TracerType       string `envconfig:"OTEL_TRACER_TYPE" default:"" pp:",omitempty"` // "gcp" or "otlp"
	OTLPEndpoint     string `envconfig:"OTEL_EXPORTER_OTLP_ENDPOINT" pp:",omitempty"`
	OTLPInsecure     bool   `envconfig:"OTEL_EXPORTER_OTLP_INSECURE" default:"false"`
	TelemetryEnabled bool   `envconfig:"OTEL_ENABLED" default:"false"`

	HealthCheckUsername string `envconfig:"HEALTH_CHECK_USERNAME" pp:",omitempty"`
	HealthCheckPassword string `envconfig:"HEALTH_CHECK_PASSWORD" pp:",omitempty"`
}

func ReadConfig(debug bool) (*Config, error) {
	if err := godotenv.Load(".env"); err != nil && !os.IsNotExist(err) {
		return nil, err
	} else if err == nil {
		log.Infof("config: .env loaded")
	}

	var c Config
	if err := envconfig.Process(configPrefix, &c); err != nil {
		return &c, err
	}

	if debug {
		c.Dev = true
	}

	return &c, c.validate()
}

// Live backends.
const (
	LiveBackendRedis    = "redis"
	LiveBackendPostgres = "postgres"
)

func (c *Config) validate() error {
	switch c.LiveBackend {
	case LiveBackendRedis:
	case LiveBackendPostgres:
		if strings.TrimSpace(c.DBPG) == "" {
			return fmt.Errorf("%s_LIVE_BACKEND=%s requires %s_DB_PG", configPrefix, LiveBackendPostgres, configPrefix)
		}
	default:
		return fmt.Errorf("%s_LIVE_BACKEND=%q is not one of %s/%s", configPrefix, c.LiveBackend, LiveBackendRedis, LiveBackendPostgres)
	}
	return nil
}

func (c *Config) Print() string {
	s := pp.Sprint(c)
	for _, secret := range c.secrets() {
		if secret == "" {
			continue
		}
		s = strings.ReplaceAll(s, secret, "***")
	}
	return s
}

// secrets lists the values Print must not emit. pp dumps every field, so a
// credential-bearing field added without being listed here is logged verbatim.
func (c *Config) secrets() []string {
	return []string{c.DB, c.DBPG, c.RedisURL, c.HealthCheckPassword}
}
