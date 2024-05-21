package memory

import (
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/usecasex"
)

func New() *repo.Container {
	return &repo.Container{
		Asset:       NewAsset(),
		Config:      NewConfig(),
		Project:     NewProject(),
		Workspace:   accountmemory.NewWorkspace(),
		Lock:        NewLock(),
		Transaction: &usecasex.NopTransaction{},
	}
}
