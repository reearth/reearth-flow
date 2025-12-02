package repo

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Deployment interface {
	Filtered(WorkspaceFilter) Deployment
	FindByIDs(context.Context, id.DeploymentIDList) ([]*deployment.Deployment, error)
	FindByID(context.Context, id.DeploymentID) (*deployment.Deployment, error)
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *interfaces.PaginationParam, *string) ([]*deployment.Deployment, *interfaces.PageBasedInfo, error)
	FindByProject(context.Context, id.ProjectID) (*deployment.Deployment, error)
	FindByVersion(context.Context, accountsid.WorkspaceID, *id.ProjectID, string) (*deployment.Deployment, error)
	FindHead(context.Context, accountsid.WorkspaceID, *id.ProjectID) (*deployment.Deployment, error)
	FindVersions(context.Context, accountsid.WorkspaceID, *id.ProjectID) ([]*deployment.Deployment, error)
	Save(context.Context, *deployment.Deployment) error
	Remove(context.Context, id.DeploymentID) error
}

func IterateDeploymentsByWorkspace(repo Deployment, ctx context.Context, tid accountsid.WorkspaceID, batch int64, callback func([]*deployment.Deployment) error) error {
	page := 1
	for {
		pagination := &interfaces.PaginationParam{
			Page: &interfaces.PageBasedPaginationParam{
				Page:     page,
				PageSize: int(batch),
			},
		}

		deployments, info, err := repo.FindByWorkspace(ctx, tid, pagination, nil)
		if err != nil {
			return err
		}
		if len(deployments) == 0 {
			break
		}

		if err := callback(deployments); err != nil {
			return err
		}

		if info.TotalCount <= int64(page*int(batch)) {
			break
		}

		page++
	}
	return nil
}
