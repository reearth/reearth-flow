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
	Dev                    bool   `pp:",omitempty"`
	GCPProject             string `envconfig:"GOOGLE_CLOUD_PROJECT" pp:",omitempty"`
	LogSubscriptionID      string `envconfig:"LOG_SUBSCRIPTION_ID" default:"flow-log-stream-main"`
	EdgePassSubscriptionID string `envconfig:"EDGE_PASS_SUBSCRIPTION_ID" default:"flow-edge-pass-through-main"`
	Port                   string `envconfig:"PORT" default:"8080"`
	RedisURL               string `envconfig:"REDIS_URL" default:"redis://localhost:6379"`
	MongoURI               string `envconfig:"MONGO_URI" default:"mongodb://localhost:27017"`
	MongoDatabaseName      string `envconfig:"MONGO_DATABASE_NAME" default:"reearth-flow"`
	MongoJobCollection     string `envconfig:"MONGO_JOB_COLLECTION" default:"jobs"`
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
