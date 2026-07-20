package main

import (
	"os"

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
	Dev          bool   `pp:",omitempty"`
	// DiagnosticSubscriptionID has NO default (unlike the sibling
	// subscription IDs above): the diagnostics subscription does not exist
	// in any deployed environment yet. Defaulting it to a name would defeat
	// the "if conf.DiagnosticSubscriptionID != ''" gate below and the one in
	// main.go — the subscriber would always try to open a subscriber for a
	// subscription that was never provisioned, and since a listener error
	// calls cancel() on the root context (main.go), that crash-loops the
	// ENTIRE subscriber, taking down log/node/job ingestion with it. Leave
	// this empty until the subscription is explicitly provisioned and wired
	// per environment (see README.md's "Diagnostics ingestion" section).
	DiagnosticSubscriptionID    string `envconfig:"DIAGNOSTIC_SUBSCRIPTION_ID" default:""`
	GCPProject                  string `envconfig:"GOOGLE_CLOUD_PROJECT" pp:",omitempty"`
	GCSBucket                   string `envconfig:"GCS_BUCKET" pp:",omitempty"`
	JobCompleteSubscriptionID   string `envconfig:"JOB_COMPLETE_SUBSCRIPTION_ID" default:"flow-job-complete-main"`
	LogSubscriptionID           string `envconfig:"LOG_SUBSCRIPTION_ID" default:"flow-log-stream-main"`
	NodeSubscriptionID          string `envconfig:"NODE_STATUS_SUBSCRIPTION_ID" default:"flow-node-status-main"`
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
	err := envconfig.Process(configPrefix, &c)

	if debug {
		c.Dev = true
	}

	return &c, err
}

func (c *Config) Print() string {
	s := pp.Sprint(c)
	return s
}
