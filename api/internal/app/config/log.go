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
	// Basic format validation for Redis address
	// Example: localhost:6379 or redis://localhost:6379
	if !strings.Contains(r.Addr, ":") {
		return false
	}
	return true
}

type GCSLogConfig struct {
	BucketName              string `pp:",omitempty"`
	PublicationCacheControl string `pp:",omitempty"`
}

func (g GCSLogConfig) IsConfigured() bool {
	return g.BucketName != ""
}
