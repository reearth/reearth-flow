package postgres

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
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

// projectColumns lists the projects table's columns in gen.Project's field
// order, for the hand-written queries below. SELECT * relies on the table's
// physical column order matching that struct, which silently breaks (with a
// confusing "number of field descriptions must equal number of destinations"
// error) the moment the two drift — naming the columns here turns that into
// the same clear "column ... does not exist" error the generated queries give.
const projectColumns = `id, workspace_id, workflow_id, name, description,
	is_archived, is_basic_auth_active, basic_auth_username, basic_auth_password,
	shared_token, updated_at, is_locked,
	alias, visibility, readme, license, topics, starred_by,
	created_by, updated_by`

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

func (r *Project) FindByAlias(ctx context.Context, wid accountsid.WorkspaceID, alias string) (*project.Project, error) {
	if !r.f.CanRead(wid) {
		return nil, rerror.ErrNotFound
	}
	row, err := r.q(ctx).GetProjectByAlias(ctx, gen.GetProjectByAliasParams{
		WorkspaceID: wid.String(),
		Alias:       alias,
	})
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	return projectFromRow(row)
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
	createdBy accountsid.UserIDList,
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

	if len(createdBy) > 0 {
		args = append(args, createdBy.Strings())
		where = append(where, fmt.Sprintf("created_by = ANY($%d::text[])", len(args)))
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
		listSQL := fmt.Sprintf("SELECT %s FROM projects %s ORDER BY %s %s", projectColumns, whereSQL, orderCol, dir)
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
		"SELECT "+projectColumns+" FROM projects "+whereSQL+" ORDER BY updated_at DESC", args,
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

// projectSearchOrder maps a sort field to its ORDER BY expression. Star counts
// are read from the starred_by list.
var projectSearchOrder = map[interfaces.ProjectSortField]string{
	interfaces.ProjectSortUpdatedAt: "updated_at",
	interfaces.ProjectSortName:      "name",
	interfaces.ProjectSortStarCount: "jsonb_array_length(COALESCE(starred_by, '[]'::jsonb))",
}

func (r *Project) Search(ctx context.Context, p interfaces.ProjectSearchParam) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	var where []string
	var args []any
	arg := func(v any) string {
		args = append(args, v)
		return fmt.Sprintf("$%d", len(args))
	}

	if p.Workspace != nil {
		where = append(where, "workspace_id = "+arg(p.Workspace.String()))
	}
	if r.f.Readable != nil {
		where = append(where, "workspace_id = ANY("+arg(r.f.Readable.Strings())+"::text[])")
	}
	if p.Visibility != nil {
		where = append(where, "visibility = "+arg(string(*p.Visibility)))
	}
	if !p.IncludeArchived {
		where = append(where, "is_archived = false")
	}
	if p.Keyword != "" {
		where = append(where, "name ILIKE "+arg("%"+escapeLike(p.Keyword)+"%"))
	}
	if len(p.Topics) > 0 {
		topics, err := pgxx.MarshalJSONBSlice(p.Topics)
		if err != nil {
			return nil, nil, err
		}
		where = append(where, "COALESCE(topics, '[]'::jsonb) @> "+arg(topics)+"::jsonb")
	}
	whereSQL := ""
	if len(where) > 0 {
		whereSQL = "WHERE " + strings.Join(where, " AND ")
	}

	order, ok := projectSearchOrder[p.SortBy]
	if !ok {
		order = projectSearchOrder[interfaces.ProjectSortUpdatedAt]
	}
	dir := "DESC"
	if p.SortAscending {
		dir = "ASC"
	}
	listSQL := fmt.Sprintf("SELECT %s FROM projects %s ORDER BY %s %s, id %s", projectColumns, whereSQL, order, dir, dir)

	rows, total, err := pgxx.CountAndList(ctx, r.c.DB(ctx),
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

// escapeLike makes s match literally inside a LIKE pattern.
func escapeLike(s string) string {
	return strings.NewReplacer(`\`, `\\`, `%`, `\%`, `_`, `\_`).Replace(s)
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
	params, err := projectToParams(p)
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	if err := r.q(ctx).UpsertProject(ctx, params); err != nil {
		if isAliasConflict(err) {
			return interfaces.ErrProjectAliasAlreadyUsed
		}
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

// SetStar edits starred_by in place, so concurrent stars on one project cannot
// overwrite each other the way a read-modify-write would.
func (r *Project) SetStar(ctx context.Context, pid id.ProjectID, uid accountsid.UserID, starred bool) error {
	set := `CASE WHEN jsonb_exists(COALESCE(starred_by, '[]'::jsonb), $2::text)
		THEN starred_by
		ELSE COALESCE(starred_by, '[]'::jsonb) || jsonb_build_array($2::text) END`
	if !starred {
		set = `COALESCE(starred_by, '[]'::jsonb) - $2::text`
	}
	query := `UPDATE projects SET starred_by = ` + set + ` WHERE id = $1`
	args := []any{pid.String(), uid.String()}
	if r.f.Readable != nil {
		query += ` AND workspace_id = ANY($3::text[])`
		args = append(args, r.f.Readable.Strings())
	}

	tag, err := r.c.DB(ctx).Exec(ctx, query, args...)
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	if tag.RowsAffected() == 0 {
		return rerror.ErrNotFound
	}
	return nil
}

const projectAliasIndex = "projects_workspace_id_alias_idx"

func isAliasConflict(err error) bool {
	var pgErr *pgconn.PgError
	return pgxx.IsUniqueViolation(err) && errors.As(err, &pgErr) && pgErr.ConstraintName == projectAliasIndex
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

func projectToParams(p *project.Project) (gen.UpsertProjectParams, error) {
	topics, err := pgxx.MarshalJSONBSlice(p.Topics())
	if err != nil {
		return gen.UpsertProjectParams{}, err
	}
	starredBy, err := pgxx.MarshalJSONBSlice(p.StarredBy())
	if err != nil {
		return gen.UpsertProjectParams{}, err
	}
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
		Alias:             p.Alias(),
		Visibility:        string(p.Visibility()),
		Readme:            p.Readme(),
		License:           p.License(),
		Topics:            topics,
		StarredBy:         starredBy,
		CreatedBy:         userIDString(p.CreatedBy()),
		UpdatedBy:         userIDString(p.UpdatedBy()),
	}, nil
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
	topics, err := pgxx.UnmarshalJSONBSlice[string](row.Topics)
	if err != nil {
		return nil, err
	}
	starredBy, err := pgxx.UnmarshalJSONBSlice[string](row.StarredBy)
	if err != nil {
		return nil, err
	}
	createdBy, err := userIDFrom(row.CreatedBy)
	if err != nil {
		return nil, err
	}
	updatedBy, err := userIDFrom(row.UpdatedBy)
	if err != nil {
		return nil, err
	}

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
		Alias(row.Alias).
		Visibility(project.Visibility(row.Visibility)).
		Readme(row.Readme).
		License(row.License).
		Topics(topics).
		StarredBy(starredBy).
		CreatedBy(createdBy).
		UpdatedBy(updatedBy).
		Build()
}

func userIDString(uid *accountsid.UserID) *string {
	if uid == nil {
		return nil
	}
	s := uid.String()
	return &s
}

func userIDFrom(s *string) (*accountsid.UserID, error) {
	if s == nil || *s == "" {
		return nil, nil
	}
	uid, err := accountsid.UserIDFrom(*s)
	if err != nil {
		return nil, err
	}
	return &uid, nil
}
