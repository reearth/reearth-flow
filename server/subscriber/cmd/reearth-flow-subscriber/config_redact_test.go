package main

import (
	"strings"
	"testing"
)

// pp dumps every config field, so Print must mask the credential-bearing ones.
// The Postgres DSN reached Cloud Logging in cleartext because it was not masked.
func TestConfig_Print_MasksCredentials(t *testing.T) {
	c := &Config{
		DB:                  "mongodb+srv://muser:mpass@cluster.example.net/db",
		DBPG:                "postgres://reearth_flow:pgpass123@10.46.0.3:5432/reearth_flow?sslmode=disable",
		RedisURL:            "redis://:redispass@10.0.0.1:6379",
		HealthCheckPassword: "hcpass",
		GCSBucket:           "some-bucket",
	}

	out := c.Print()

	for _, leaked := range []string{"mpass", "pgpass123", "redispass", "hcpass"} {
		if strings.Contains(out, leaked) {
			t.Errorf("Print leaked %q", leaked)
		}
	}
	if !strings.Contains(out, "some-bucket") {
		t.Error("Print masked a non-secret field")
	}
}
