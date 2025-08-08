package gql

import (
	"net/http"
	"strings"

	"github.com/hasura/go-graphql-client"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/user"
	userpkg "github.com/reearth/reearth-flow/api/pkg/user"
)

type Client struct {
	UserRepo userpkg.Repo
}

func NewClient(host string, transport http.RoundTripper) *Client {
	httpClient := &http.Client{
		Transport: transport,
	}

	normalizedHost := strings.TrimRight(host, "/")
	fullEndpoint := normalizedHost + "/api/graphql"
	gqlClient := graphql.NewClient(fullEndpoint, httpClient)

	return &Client{
		UserRepo: user.NewRepo(gqlClient),
	}
}
