package config

import (
	"strings"
)

type RedisLogConfig struct {
	Addr     string `envconfig:"REEARTH_FLOW_REDIS_ADDR" pp:",omitempty"`
	Password string `envconfig:"REEARTH_FLOW_REDIS_PASSWORD" pp:",omitempty"`
	DB       int    `envconfig:"REEARTH_FLOW_REDIS_DB" pp:",omitempty"`
}

func (r RedisLogConfig) IsConfigured() bool {
	if r.Addr == "" {
		return false
	}
	if r.DB < 0 || r.DB > 15 {
		return false
	}
	if !strings.Contains(r.Addr, ":") {
		return false
	}
	return true
}
