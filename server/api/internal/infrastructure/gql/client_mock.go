package gql

import (
	userpkg "github.com/reearth/reearth-flow/api/pkg/user"
	workspacepkg "github.com/reearth/reearth-flow/api/pkg/workspace"
)

type MockClientParam struct {
	UserRepo      userpkg.Repo
	WorkspaceRepo workspacepkg.Repo
}

func NewMockClient(p *MockClientParam) *Client {
	return &Client{
		UserRepo:      p.UserRepo,
		WorkspaceRepo: p.WorkspaceRepo,
	}
}
