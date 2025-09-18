package config

import (
	"github.com/samber/lo"
)

const AuthServerDefaultClientID = "reearth-authsrv-client-default"

type AuthSrvConfig struct {
	Dev      bool             `pp:",omitempty"`
	Disabled bool             `pp:",omitempty"`
	Issuer   string           `pp:",omitempty"`
	Domain   string           `pp:",omitempty"`
	UIDomain string           `pp:",omitempty"`
	Key      string           `pp:",omitempty"`
	DN       *AuthSrvDNConfig `pp:",omitempty"`
}

func (c AuthSrvConfig) AuthConfig(debug bool, host string) *AuthConfig {
	if c.Disabled {
		return nil
	}

	domain := c.Domain
	if domain == "" {
		domain = host
	}

	var aud []string
	if debug && host != "" && c.Domain != "" && c.Domain != host {
		aud = []string{host, c.Domain}
	} else {
		aud = []string{domain}
	}

	return &AuthConfig{
		ISS:      getAuthDomain(domain),
		AUD:      aud,
		ClientID: lo.ToPtr(AuthServerDefaultClientID),
	}
}

type AuthSrvDNConfig struct {
	CN         string   `pp:",omitempty"`
	O          []string `pp:",omitempty"`
	OU         []string `pp:",omitempty"`
	C          []string `pp:",omitempty"`
	L          []string `pp:",omitempty"`
	ST         []string `pp:",omitempty"`
	Street     []string `pp:",omitempty"`
	PostalCode []string `pp:",omitempty"`
}
