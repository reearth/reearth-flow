package config

import (
	"strings"
	"testing"
)

// pp dumps every config field, so secrets() has to list each credential-bearing
// one. DB_PG was missing, which put the Postgres DSN into Cloud Logging in
// cleartext once DB_Driver was switched to postgres.
func TestConfig_Print_MasksCredentials(t *testing.T) {
	c := &Config{}
	c.DB = "mongodb+srv://muser:mpass@cluster.example.net/db"
	c.DB_PG = "postgres://reearth_flow:pgpass123@10.46.0.3:5432/reearth_flow?sslmode=disable"
	c.Auth0.ClientSecret = "auth0secret"
	c.HealthCheck.Password = "hcpass"
	c.SignupSecret = "signupsecret"
	c.WebsocketAPISecret = "wssecret"
	c.CMS_Token = "cmstoken"
	c.Redis_URL = "redis://:redispass@10.0.0.1:6379"
	c.AssetBaseURL = "http://localhost:8080/assets"

	out := c.Print()

	for _, leaked := range []string{
		"mpass", "pgpass123", "auth0secret", "hcpass",
		"signupsecret", "wssecret", "cmstoken", "redispass",
	} {
		if strings.Contains(out, leaked) {
			t.Errorf("Print leaked %q", leaked)
		}
	}
	if !strings.Contains(out, "http://localhost:8080/assets") {
		t.Error("Print masked a non-secret field")
	}
}
