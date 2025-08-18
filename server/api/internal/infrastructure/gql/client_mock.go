package gql

import (
	userpkg "github.com/reearth/reearth-flow/api/pkg/user"
)

type MockClientParam struct {
	UserRepo userpkg.Repo
}

func NewMockClient(p *MockClientParam) *Client {
	return &Client{
		UserRepo: p.UserRepo,
	}
}
