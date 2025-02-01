package config

import "strings"

type RedisLogConfig struct {
	Addr     string `pp:",omitempty"`
	Password string `pp:",omitempty"`
	DB       int    `pp:",omitempty"`
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
