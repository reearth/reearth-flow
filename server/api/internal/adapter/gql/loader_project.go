package gql

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/util"
)

type ProjectLoader struct {
	usecase interfaces.Project
}

func NewProjectLoader(usecase interfaces.Project) *ProjectLoader {
	return &ProjectLoader{usecase: usecase}
}

func (c *ProjectLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.Project, []error) {
	ids2, err := util.TryMap(ids, gqlmodel.ToID[id.Project])
	if err != nil {
		return nil, []error{err}
	}

	res, err := c.usecase.Fetch(ctx, ids2)
	if err != nil {
		return nil, []error{err}
	}

	projects := make([]*gqlmodel.Project, 0, len(res))
	for _, project := range res {
		projects = append(projects, gqlmodel.ToProject(project))
	}

	return projects, nil
}

func (c *ProjectLoader) FindByWorkspacePage(ctx context.Context, wsID gqlmodel.ID, keyword *string, includeArchived *bool, pagination gqlmodel.PageBasedPagination) (*gqlmodel.ProjectConnection, error) {
	tid, err := gqlmodel.ToID[accountsid.Workspace](wsID)
	if err != nil {
		return nil, err
	}

	paginationParam := gqlmodel.ToPageBasedPagination(pagination)

	res, pi, err := c.usecase.FindByWorkspace(ctx, tid, paginationParam, keyword, includeArchived)
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.Project, 0, len(res))
	for _, p := range res {
		nodes = append(nodes, gqlmodel.ToProject(p))
	}

	pageInfo := gqlmodel.ToPageInfo(pi)
	if pageInfo.CurrentPage == nil {
		cp := pagination.Page
		pageInfo.CurrentPage = &cp
	}
	if pageInfo.TotalPages == nil {
		tp := (int(pi.TotalCount) + pagination.PageSize - 1) / pagination.PageSize
		pageInfo.TotalPages = &tp
	}

	return &gqlmodel.ProjectConnection{
		Nodes:      nodes,
		PageInfo:   pageInfo,
		TotalCount: int(pi.TotalCount),
	}, nil
}

// data loaders

type ProjectDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.Project, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.Project, []error)
}

func (c *ProjectLoader) DataLoader(ctx context.Context) ProjectDataLoader {
	return gqldataloader.NewProjectLoader(gqldataloader.ProjectLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Project, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *ProjectLoader) OrdinaryDataLoader(ctx context.Context) ProjectDataLoader {
	return &ordinaryProjectLoader{
		fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Project, []error) {
			return c.Fetch(ctx, keys)
		},
	}
}

type ordinaryProjectLoader struct {
	fetch func(keys []gqlmodel.ID) ([]*gqlmodel.Project, []error)
}

func (l *ordinaryProjectLoader) Load(key gqlmodel.ID) (*gqlmodel.Project, error) {
	res, errs := l.fetch([]gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryProjectLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.Project, []error) {
	return l.fetch(keys)
}
