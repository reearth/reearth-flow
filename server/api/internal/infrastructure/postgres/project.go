package postgres

import (
	"context"
	"fmt"
	"strings"

	"github.com/jackc/pgx/v5"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type Project struct {
	c *pgxx.Client
	f repo.WorkspaceFilter
}

var _ repo.Project = (*Project)(nil)

func NewProject(c *pgxx.Client) *Project {
	return &Project{c: c}
}

func (r *Project) Filtered(f repo.WorkspaceFilter) repo.Project {
	return &Project{c: r.c, f: r.f.Merge(f)}
}

func (r *Project) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *Project) FindByID(ctx context.Context, pid id.ProjectID) (*project.Project, error) {
	row, err := r.q(ctx).GetProject(ctx, pid.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	p, err := projectFromRow(row)
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(p.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return p, nil
}

func (r *Project) FindByIDs(ctx context.Context, ids id.ProjectIDList) ([]*project.Project, error) {
	rows, err := r.q(ctx).ListProjectsByIDs(ctx, ids.Strings())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	ps := make([]*project.Project, 0, len(rows))
	for _, row := range rows {
		p, err := projectFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(p.Workspace()) {
			ps = append(ps, p)
		}
	}
	return pgxx.OrderByIDs(ids.Strings(), ps, func(p *project.Project) string { return p.ID().String() }), nil
}

var projectOrderByColumns = map[string]string{
	"name":      "name",
	"updatedAt": "updated_at",
	"id":        "id",
}

// FindByWorkspace lists projects for a workspace.
// includeArchived: nil or false → exclude archived; true → include archived.
func (r *Project) FindByWorkspace(
	ctx context.Context,
	wid accountsid.WorkspaceID,
	pagination *interfaces.PaginationParam,
	keyword *string,
	includeArchived *bool,
) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(wid) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	where := []string{"workspace_id = $1"}
	args := []any{wid.String()}

	if includeArchived == nil || !*includeArchived {
		where = append(where, "is_archived = false")
	}

	if keyword != nil && *keyword != "" {
		args = append(args, "%"+*keyword+"%")
		where = append(where, fmt.Sprintf("name ILIKE $%d", len(args)))
	}

	whereSQL := "WHERE " + strings.Join(where, " AND ")
	db := r.c.DB(ctx)

	if pagination != nil && pagination.Page != nil {
		p := pagination.Page
		orderCol := "updated_at"
		dir := "DESC"
		if p.OrderBy != nil {
			if c, ok := projectOrderByColumns[*p.OrderBy]; ok {
				orderCol = c
			}
			if p.OrderDir != nil && strings.EqualFold(*p.OrderDir, "ASC") {
				dir = "ASC"
			}
		}
		listSQL := fmt.Sprintf("SELECT * FROM projects %s ORDER BY %s %s", whereSQL, orderCol, dir)
		rows, total, err := pgxx.CountAndList(ctx, db,
			"SELECT count(*) FROM projects "+whereSQL, listSQL, args,
			int64(p.PageSize), int64((p.Page-1)*p.PageSize),
			pgx.RowToStructByPos[gen.Project])
		if err != nil {
			return nil, nil, err
		}
		ps, err := projectsFromRows(rows)
		if err != nil {
			return nil, nil, err
		}
		return ps, interfaces.NewPageBasedInfo(total, p.Page, p.PageSize), nil
	}

	rows, err := pgxx.List(ctx, db,
		"SELECT * FROM projects "+whereSQL+" ORDER BY updated_at DESC", args,
		pgx.RowToStructByPos[gen.Project])
	if err != nil {
		return nil, nil, err
	}
	ps, err := projectsFromRows(rows)
	if err != nil {
		return nil, nil, err
	}
	return ps, interfaces.NewPageBasedInfo(int64(len(ps)), 1, len(ps)), nil
}

func (r *Project) CountByWorkspace(ctx context.Context, ws accountsid.WorkspaceID) (int, error) {
	if !r.f.CanRead(ws) {
		return 0, repo.ErrOperationDenied
	}
	n, err := r.q(ctx).CountProjectsByWorkspace(ctx, ws.String())
	if err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return int(n), nil
}

func (r *Project) CountPublicByWorkspace(ctx context.Context, ws accountsid.WorkspaceID) (int, error) {
	if !r.f.CanRead(ws) {
		return 0, repo.ErrOperationDenied
	}
	n, err := r.q(ctx).CountPublicProjectsByWorkspace(ctx, ws.String())
	if err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return int(n), nil
}

func (r *Project) Save(ctx context.Context, p *project.Project) error {
	if !r.f.CanWrite(p.Workspace()) {
		return repo.ErrOperationDenied
	}
	if err := r.q(ctx).UpsertProject(ctx, projectToParams(p)); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Project) Remove(ctx context.Context, pid id.ProjectID) error {
	exec := r.c.DB(ctx)
	if r.f.Writable == nil {
		if _, err := exec.Exec(ctx, `DELETE FROM projects WHERE id = $1`, pid.String()); err != nil {
			return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}
		return nil
	}
	if _, err := exec.Exec(ctx,
		`DELETE FROM projects WHERE id = $1 AND workspace_id = ANY($2::text[])`,
		pid.String(), r.f.Writable.Strings(),
	); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func projectsFromRows(rows []gen.Project) ([]*project.Project, error) {
	res := make([]*project.Project, 0, len(rows))
	for _, row := range rows {
		p, err := projectFromRow(row)
		if err != nil {
			return nil, err
		}
		res = append(res, p)
	}
	return res, nil
}

func projectToParams(p *project.Project) gen.UpsertProjectParams {
	return gen.UpsertProjectParams{
		ID:                p.ID().String(),
		WorkspaceID:       p.Workspace().String(),
		WorkflowID:        p.Workflow().String(),
		Name:              p.Name(),
		Description:       p.Description(),
		IsArchived:        p.IsArchived(),
		IsBasicAuthActive: p.IsBasicAuthActive(),
		BasicAuthUsername: p.BasicAuthUsername(),
		BasicAuthPassword: p.BasicAuthPassword(),
		SharedToken:       p.SharedToken(),
		UpdatedAt:         p.UpdatedAt(),
		IsLocked:          p.IsLocked(),
	}
}

func projectFromRow(row gen.Project) (*project.Project, error) {
	pid, err := id.ProjectIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	wid, err := accountsid.WorkspaceIDFrom(row.WorkspaceID)
	if err != nil {
		return nil, err
	}
	wfid, _ := id.WorkflowIDFrom(row.WorkflowID)

	return project.New().
		ID(pid).
		Workspace(wid).
		Workflow(wfid).
		Name(row.Name).
		Description(row.Description).
		IsArchived(row.IsArchived).
		IsBasicAuthActive(row.IsBasicAuthActive).
		BasicAuthUsername(row.BasicAuthUsername).
		BasicAuthPassword(row.BasicAuthPassword).
		SharedToken(row.SharedToken).
		UpdatedAt(row.UpdatedAt).
		IsLocked(row.IsLocked).
		Build()
}
