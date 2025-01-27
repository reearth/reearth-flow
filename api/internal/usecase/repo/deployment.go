package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
	"github.com/samber/lo"
)

type Deployment interface {
	Filtered(WorkspaceFilter) Deployment
	FindByIDs(context.Context, id.DeploymentIDList) ([]*deployment.Deployment, error)
	FindByID(context.Context, id.DeploymentID) (*deployment.Deployment, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *interfaces.PaginationParam) ([]*deployment.Deployment, *usecasex.PageInfo, error)
	FindByProject(context.Context, id.ProjectID) (*deployment.Deployment, error)
	FindByVersion(context.Context, accountdomain.WorkspaceID, *id.ProjectID, string) (*deployment.Deployment, error)
	FindHead(context.Context, accountdomain.WorkspaceID, *id.ProjectID) (*deployment.Deployment, error)
	FindVersions(context.Context, accountdomain.WorkspaceID, *id.ProjectID) ([]*deployment.Deployment, error)
	Save(context.Context, *deployment.Deployment) error
	Remove(context.Context, id.DeploymentID) error
}

func IterateDeploymentsByWorkspace(repo Deployment, ctx context.Context, tid accountdomain.WorkspaceID, batch int64, callback func([]*deployment.Deployment) error) error {
	cursorPagination := usecasex.CursorPagination{
		First: lo.ToPtr(batch),
	}.Wrap()

	for {
		pagination := &interfaces.PaginationParam{
			Cursor: cursorPagination,
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

		if !info.HasNextPage {
			break
		}

		c := usecasex.Cursor(deployments[len(deployments)-1].ID().String())
		cursorPagination = usecasex.CursorPagination{
			First: lo.ToPtr(batch),
			After: &c,
		}.Wrap()
	}

	return nil
}
