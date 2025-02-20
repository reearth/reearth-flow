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
	Dev            bool   `pp:",omitempty"`
	Port           string `default:"8080" envconfig:"PORT"`
	ProjectID      string `envconfig:"PROJECT_ID" default:"local-project"`
	RedisURL       string `envconfig:"REDIS_URL" default:"redis://localhost:6379"`
	SubscriptionID string `envconfig:"SUBSCRIPTION_ID" default:"flow-log-stream-topic"`
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
