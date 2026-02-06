package factory

import (
	faker "github.com/go-faker/faker/v4"
	"github.com/reearth/reearth-accounts/server/pkg/workspace"
)

type WorkspaceOption func(*workspace.Builder)

func NewWorkspace(opts ...WorkspaceOption) *workspace.Workspace {
	p := workspace.New().
		ID(workspace.NewID()).
		Name(faker.Name()).
		Alias(faker.Username())
	for _, opt := range opts {
		opt(p)
	}
	return p.MustBuild()
}
