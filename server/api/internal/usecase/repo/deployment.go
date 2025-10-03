package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Deployment interface {
	Filtered(WorkspaceFilter) Deployment
	FindByIDs(context.Context, id.DeploymentIDList) ([]*deployment.Deployment, error)
	FindByID(context.Context, id.DeploymentID) (*deployment.Deployment, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *interfaces.PaginationParam) ([]*deployment.Deployment, *interfaces.PageBasedInfo, error)
	FindByProject(context.Context, id.ProjectID) (*deployment.Deployment, error)
	FindByVersion(context.Context, id.WorkspaceID, *id.ProjectID, string) (*deployment.Deployment, error)
	FindHead(context.Context, id.WorkspaceID, *id.ProjectID) (*deployment.Deployment, error)
	FindVersions(context.Context, id.WorkspaceID, *id.ProjectID) ([]*deployment.Deployment, error)
	Save(context.Context, *deployment.Deployment) error
	Remove(context.Context, id.DeploymentID) error
}

func IterateDeploymentsByWorkspace(repo Deployment, ctx context.Context, tid id.WorkspaceID, batch int64, callback func([]*deployment.Deployment) error) error {
	page := 1
	for {
		pagination := &interfaces.PaginationParam{
			Page: &interfaces.PageBasedPaginationParam{
				Page:     page,
				PageSize: int(batch),
			},
		}

		deployments, info, err := repo.FindByWorkspace(ctx, tid, pagination)
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
