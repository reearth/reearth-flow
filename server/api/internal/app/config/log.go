package config

import (
	"strings"
)

type RedisLogConfig struct {
	RedisURL string `pp:",omitempty"`
}

func (r RedisLogConfig) IsConfigured() bool {
	if r.RedisURL == "" {
		return false
	}
	if !strings.Contains(r.RedisURL, ":") {
		return false
	}
	return true
}
