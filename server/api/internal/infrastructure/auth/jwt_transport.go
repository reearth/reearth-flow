package auth

import (
	"net/http"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	"github.com/reearth/reearth-flow/api/internal/adapter"
)

type DynamicAuthTransport struct {
	transport *gqlclient.AccountsTransport
}

func NewDynamicAuthTransport() *DynamicAuthTransport {
	return &DynamicAuthTransport{
		transport: gqlclient.NewAccountsTransport(http.DefaultTransport, gqlclient.InternalServiceFlowAPI),
	}
}

func (t DynamicAuthTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	var token string

	// The legacy AuthInfo token is a fallback for the accounts-server migration.
	// TODO: Remove authInfo handling once the migration is complete
	if jwtToken := adapter.JWT(req.Context()); jwtToken != "" {
		token = jwtToken
	} else if authInfo := adapter.TempAuthInfo(req.Context()); authInfo != nil && authInfo.Token != "" {
		token = authInfo.Token
	}

	if token != "" {
		req.Header.Set("Authorization", "Bearer "+token)
	}

	return t.transport.RoundTrip(req)
}
