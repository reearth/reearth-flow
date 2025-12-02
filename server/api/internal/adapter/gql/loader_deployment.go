package gql

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/idx"
	"github.com/reearth/reearthx/util"
)

type DeploymentLoader struct {
	usecase interfaces.Deployment
}

func NewDeploymentLoader(usecase interfaces.Deployment) *DeploymentLoader {
	return &DeploymentLoader{usecase: usecase}
}

func (c *DeploymentLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.Deployment, []error) {
	ids2, err := util.TryMap(ids, gqlmodel.ToID[id.Deployment])
	if err != nil {
		return nil, []error{err}
	}

	res, err := c.usecase.Fetch(ctx, ids2)
	if err != nil {
		return nil, []error{err}
	}

	deployments := make([]*gqlmodel.Deployment, 0, len(res))
	for _, deployment := range res {
		deployments = append(deployments, gqlmodel.ToDeployment(deployment))
	}

	return deployments, nil
}

func (c *DeploymentLoader) FindByWorkspacePage(ctx context.Context, wsID gqlmodel.ID, keyword *string, pagination gqlmodel.PageBasedPagination) (*gqlmodel.DeploymentConnection, error) {
	wID, err := gqlmodel.ToID[accountsid.Workspace](wsID)
	if err != nil {
		return nil, err
	}

	paginationParam := &interfaces.PaginationParam{
		Page: &interfaces.PageBasedPaginationParam{
			Page:     pagination.Page,
			PageSize: pagination.PageSize,
			OrderBy:  pagination.OrderBy,
			OrderDir: gqlmodel.OrderDirectionToString(pagination.OrderDir),
		},
	}

	res, pageInfo, err := c.usecase.FindByWorkspace(ctx, wID, paginationParam, keyword)
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.Deployment, 0, len(res))
	for _, d := range res {
		nodes = append(nodes, gqlmodel.ToDeployment(d))
	}

	return &gqlmodel.DeploymentConnection{
		Nodes:      nodes,
		PageInfo:   gqlmodel.ToPageInfo(pageInfo),
		TotalCount: len(res),
	}, nil
}

func (c *DeploymentLoader) FindByVersion(ctx context.Context, input *gqlmodel.GetByVersionInput) (*gqlmodel.Deployment, error) {
	wsID, err := gqlmodel.ToID[accountsid.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	var pID *idx.ID[id.Project]
	if input.ProjectID != nil {
		pid, err := gqlmodel.ToID[id.Project](*input.ProjectID)
		if err != nil {
			return nil, err
		}
		pID = &pid
	}

	res, err := c.usecase.FindByVersion(ctx, wsID, pID, input.Version)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToDeployment(res), nil
}

func (c *DeploymentLoader) FindHead(ctx context.Context, input *gqlmodel.GetHeadInput) (*gqlmodel.Deployment, error) {
	wsID, err := gqlmodel.ToID[accountsid.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	var pID *idx.ID[id.Project]
	if input.ProjectID != nil {
		pid, err := gqlmodel.ToID[id.Project](*input.ProjectID)
		if err != nil {
			return nil, err
		}
		pID = &pid
	}

	res, err := c.usecase.FindHead(ctx, wsID, pID)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToDeployment(res), nil
}

func (c *DeploymentLoader) FindVersions(ctx context.Context, wsID gqlmodel.ID, pID *gqlmodel.ID) ([]*gqlmodel.Deployment, error) {
	wID, err := gqlmodel.ToID[accountsid.Workspace](wsID)
	if err != nil {
		return nil, err
	}

	var projectID *idx.ID[id.Project]
	if pID != nil {
		pid, err := gqlmodel.ToID[id.Project](*pID)
		if err != nil {
			return nil, err
		}
		projectID = &pid
	}

	res, err := c.usecase.FindVersions(ctx, wID, projectID)
	if err != nil {
		return nil, err
	}

	deployments := make([]*gqlmodel.Deployment, 0, len(res))
	for _, d := range res {
		deployments = append(deployments, gqlmodel.ToDeployment(d))
	}

	return deployments, nil
}

func (c *DeploymentLoader) FindByProject(ctx context.Context, pID gqlmodel.ID) (*gqlmodel.Deployment, error) {
	pid, err := gqlmodel.ToID[id.Project](pID)
	if err != nil {
		return nil, err
	}

	res, _ := c.usecase.FindByProject(ctx, pid)

	dep := gqlmodel.ToDeployment(res)

	return dep, nil
}

// data loaders

type DeploymentDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.Deployment, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.Deployment, []error)
}

func (c *DeploymentLoader) DataLoader(ctx context.Context) DeploymentDataLoader {
	return gqldataloader.NewDeploymentLoader(gqldataloader.DeploymentLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Deployment, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *DeploymentLoader) OrdinaryDataLoader(ctx context.Context) DeploymentDataLoader {
	return &ordinaryDeploymentLoader{
		fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Deployment, []error) {
			return c.Fetch(ctx, keys)
		},
	}
}

type ordinaryDeploymentLoader struct {
	fetch func(keys []gqlmodel.ID) ([]*gqlmodel.Deployment, []error)
}

func (l *ordinaryDeploymentLoader) Load(key gqlmodel.ID) (*gqlmodel.Deployment, error) {
	res, errs := l.fetch([]gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryDeploymentLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.Deployment, []error) {
	return l.fetch(keys)
}
