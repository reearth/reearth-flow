package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
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

	res, err := c.usecase.Fetch(ctx, ids2, getOperator(ctx))
	if err != nil {
		return nil, []error{err}
	}

	deployments := make([]*gqlmodel.Deployment, 0, len(res))
	for _, deployment := range res {
		deployments = append(deployments, gqlmodel.ToDeployment(deployment))
	}

	return deployments, nil
}

func (c *DeploymentLoader) FindByWorkspace(ctx context.Context, wsID gqlmodel.ID, pagination *gqlmodel.Pagination) (*gqlmodel.DeploymentConnection, error) {
	tid, err := gqlmodel.ToID[accountdomain.Workspace](wsID)
	if err != nil {
		return nil, err
	}

	res, pi, err := c.usecase.FindByWorkspace(ctx, tid, gqlmodel.ToPagination(pagination), getOperator(ctx))
	if err != nil {
		return nil, err
	}

	edges := make([]*gqlmodel.DeploymentEdge, 0, len(res))
	nodes := make([]*gqlmodel.Deployment, 0, len(res))
	for _, d := range res {
		dep := gqlmodel.ToDeployment(d)
		edges = append(edges, &gqlmodel.DeploymentEdge{
			Node:   dep,
			Cursor: usecasex.Cursor(dep.ID),
		})
		nodes = append(nodes, dep)
	}

	return &gqlmodel.DeploymentConnection{
		Edges:      edges,
		Nodes:      nodes,
		PageInfo:   gqlmodel.ToPageInfo(pi),
		TotalCount: int(pi.TotalCount),
	}, nil
}

func (c *DeploymentLoader) FindByProject(ctx context.Context, pID gqlmodel.ID) (*gqlmodel.Deployment, error) {
	pid, err := gqlmodel.ToID[id.Project](pID)
	if err != nil {
		return nil, err
	}

	res, _ := c.usecase.FindByProject(ctx, pid, getOperator(ctx))

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
