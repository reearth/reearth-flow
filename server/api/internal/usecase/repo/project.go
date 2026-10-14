package repo

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
)

type Project interface {
	Filtered(WorkspaceFilter) Project
	CountByWorkspace(context.Context, accountsid.WorkspaceID) (int, error)
	CountPublicByWorkspace(context.Context, accountsid.WorkspaceID) (int, error)
	// FindByAlias returns rerror.ErrNotFound when no project in the workspace has the alias.
	FindByAlias(context.Context, accountsid.WorkspaceID, string) (*project.Project, error)
	FindByID(context.Context, id.ProjectID) (*project.Project, error)
	FindByIDs(context.Context, id.ProjectIDList) ([]*project.Project, error)
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *interfaces.PaginationParam, *string, *bool) ([]*project.Project, *interfaces.PageBasedInfo, error)
	Remove(context.Context, id.ProjectID) error
	// Save does not write stars once the project exists; SetStar owns them. It
	// returns interfaces.ErrProjectAliasAlreadyUsed when another project in the
	// workspace holds the alias.
	Save(context.Context, *project.Project) error
	// SetStar adds or removes the user's star in a single atomic write, and
	// returns rerror.ErrNotFound when the project does not exist or is not readable.
	SetStar(ctx context.Context, pid id.ProjectID, uid accountsid.UserID, starred bool) error
}

func IterateProjectsByWorkspace(repo Project, ctx context.Context, tid accountsid.WorkspaceID, batch int64, callback func([]*project.Project) error) error {
	page := 1
	for {
		pagination := &interfaces.PaginationParam{
			Page: &interfaces.PageBasedPaginationParam{
				Page:     page,
				PageSize: int(batch),
			},
		}

		projects, info, err := repo.FindByWorkspace(ctx, tid, pagination, nil, nil)
		if err != nil {
			return err
		}
		if len(projects) == 0 {
			break
		}

		if err := callback(projects); err != nil {
			return err
		}

		if info.TotalCount <= int64(page*int(batch)) {
			break
		}

		page++
	}
	return nil
}
