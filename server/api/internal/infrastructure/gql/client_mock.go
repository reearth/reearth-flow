package gql

import (
	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	gqluser "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user"
	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
)

type MockClientParam struct {
	UserRepo      gqluser.UserRepo
	WorkspaceRepo gqlworkspace.WorkspaceRepo
}

func NewMockClient(p *MockClientParam) *gqlclient.Client {
	return &gqlclient.Client{
		UserRepo:      p.UserRepo,
		WorkspaceRepo: p.WorkspaceRepo,
	}
}
