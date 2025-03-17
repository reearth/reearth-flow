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
	AssetBaseURL       string `envconfig:"ASSET_BASE_URL" default:"http://localhost:8080/assets"`
	DB                 string `default:"mongodb://localhost"`
	Dev                bool   `pp:",omitempty"`
	EdgeSubscriptionID string `envconfig:"EDGE_SUBSCRIPTION_ID" default:"flow-edge-pass-through-main"`
	GCPProject         string `envconfig:"GOOGLE_CLOUD_PROJECT" pp:",omitempty"`
	GCSBucket          string `envconfig:"GCS_BUCKET" pp:",omitempty"`
	LogSubscriptionID  string `envconfig:"LOG_SUBSCRIPTION_ID" default:"flow-log-stream-main"`
	Port               string `envconfig:"PORT" default:"8080"`
	RedisURL           string `envconfig:"REDIS_URL" default:"redis://localhost:6379"`
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
