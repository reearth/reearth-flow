package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
)

type Project interface {
	Filtered(WorkspaceFilter) Project
	CountByWorkspace(context.Context, id.WorkspaceID) (int, error)
	CountPublicByWorkspace(context.Context, id.WorkspaceID) (int, error)
	FindByID(context.Context, id.ProjectID) (*project.Project, error)
	FindByIDs(context.Context, id.ProjectIDList) ([]*project.Project, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *interfaces.PaginationParam, *string) ([]*project.Project, *interfaces.PageBasedInfo, error)
	Remove(context.Context, id.ProjectID) error
	Save(context.Context, *project.Project) error
}

func IterateProjectsByWorkspace(repo Project, ctx context.Context, tid id.WorkspaceID, batch int64, callback func([]*project.Project) error) error {
	page := 1
	for {
		pagination := &interfaces.PaginationParam{
			Page: &interfaces.PageBasedPaginationParam{
				Page:     page,
				PageSize: int(batch),
			},
		}

		projects, info, err := repo.FindByWorkspace(ctx, tid, pagination, nil)
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
