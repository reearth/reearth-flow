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
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type Deployment struct {
	c *pgxx.Client
	f repo.WorkspaceFilter
}

var _ repo.Deployment = (*Deployment)(nil)

func NewDeployment(c *pgxx.Client) *Deployment {
	return &Deployment{c: c}
}

func (r *Deployment) Filtered(f repo.WorkspaceFilter) repo.Deployment {
	return &Deployment{c: r.c, f: r.f.Merge(f)}
}

func (r *Deployment) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *Deployment) FindByID(ctx context.Context, did id.DeploymentID) (*deployment.Deployment, error) {
	row, err := r.q(ctx).GetDeployment(ctx, did.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	d, err := deploymentFromRow(row)
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(d.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return d, nil
}

func (r *Deployment) FindByIDs(ctx context.Context, ids id.DeploymentIDList) ([]*deployment.Deployment, error) {
	rows, err := r.q(ctx).ListDeploymentsByIDs(ctx, ids.Strings())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	ds := make([]*deployment.Deployment, 0, len(rows))
	for _, row := range rows {
		d, err := deploymentFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(d.Workspace()) {
			ds = append(ds, d)
		}
	}
	return pgxx.OrderByIDs(ids.Strings(), ds, func(d *deployment.Deployment) string { return d.ID().String() }), nil
}

var deploymentOrderByColumns = map[string]string{
	"description": "description",
	"version":     "version",
	"updatedAt":   "updated_at",
	"id":          "id",
}

func (r *Deployment) FindByWorkspace(
	ctx context.Context,
	wid accountsid.WorkspaceID,
	pagination *interfaces.PaginationParam,
	keyword *string,
) ([]*deployment.Deployment, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(wid) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	where := []string{"workspace_id = $1"}
	args := []any{wid.String()}
	if keyword != nil && *keyword != "" {
		args = append(args, "%"+*keyword+"%")
		where = append(where, fmt.Sprintf(
			"(description ILIKE $%d OR version ILIKE $%d OR id ILIKE $%d)",
			len(args), len(args), len(args),
		))
	}
	whereSQL := "WHERE " + strings.Join(where, " AND ")
	db := r.c.DB(ctx)

	if pagination != nil && pagination.Page != nil {
		p := pagination.Page
		orderCol := "updated_at"
		dir := "DESC"
		if p.OrderBy != nil {
			if c, ok := deploymentOrderByColumns[*p.OrderBy]; ok {
				orderCol = c
			}
			dir = "ASC"
			if p.OrderDir != nil && strings.EqualFold(*p.OrderDir, "DESC") {
				dir = "DESC"
			}
		}
		listSQL := fmt.Sprintf("SELECT * FROM deployments %s ORDER BY %s %s", whereSQL, orderCol, dir)
		rows, total, err := pgxx.CountAndList(ctx, db,
			"SELECT count(*) FROM deployments "+whereSQL, listSQL, args,
			int64(p.PageSize), int64((p.Page-1)*p.PageSize),
			pgx.RowToStructByPos[gen.Deployment])
		if err != nil {
			return nil, nil, err
		}
		ds, err := deploymentsFromRows(rows)
		if err != nil {
			return nil, nil, err
		}
		return ds, interfaces.NewPageBasedInfo(total, p.Page, p.PageSize), nil
	}

	rows, err := pgxx.List(ctx, db,
		"SELECT * FROM deployments "+whereSQL+" ORDER BY updated_at DESC", args,
		pgx.RowToStructByPos[gen.Deployment])
	if err != nil {
		return nil, nil, err
	}
	ds, err := deploymentsFromRows(rows)
	if err != nil {
		return nil, nil, err
	}
	return ds, interfaces.NewPageBasedInfo(int64(len(ds)), 1, len(ds)), nil
}

func (r *Deployment) FindByProject(ctx context.Context, pid id.ProjectID) (*deployment.Deployment, error) {
	db := r.c.DB(ctx)
	rows, err := pgxx.List(ctx, db,
		"SELECT * FROM deployments WHERE project_id = $1 AND is_head = true LIMIT 1",
		[]any{pid.String()},
		pgx.RowToStructByPos[gen.Deployment])
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	if len(rows) == 0 {
		return nil, rerror.ErrNotFound
	}
	d, err := deploymentFromRow(rows[0])
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(d.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return d, nil
}

func (r *Deployment) FindByVersion(ctx context.Context, wsID accountsid.WorkspaceID, pID *id.ProjectID, version string) (*deployment.Deployment, error) {
	db := r.c.DB(ctx)
	var rows []gen.Deployment
	var err error
	if pID != nil {
		rows, err = pgxx.List(ctx, db,
			"SELECT * FROM deployments WHERE workspace_id = $1 AND project_id IS NOT DISTINCT FROM $2 AND version = $3 LIMIT 1",
			[]any{wsID.String(), pID.String(), version},
			pgx.RowToStructByPos[gen.Deployment])
	} else {
		rows, err = pgxx.List(ctx, db,
			"SELECT * FROM deployments WHERE workspace_id = $1 AND project_id IS NULL AND version = $2 LIMIT 1",
			[]any{wsID.String(), version},
			pgx.RowToStructByPos[gen.Deployment])
	}
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	if len(rows) == 0 {
		return nil, rerror.ErrNotFound
	}
	d, err := deploymentFromRow(rows[0])
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(d.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return d, nil
}

func (r *Deployment) FindHead(ctx context.Context, wsID accountsid.WorkspaceID, pID *id.ProjectID) (*deployment.Deployment, error) {
	db := r.c.DB(ctx)
	var rows []gen.Deployment
	var err error
	if pID != nil {
		rows, err = pgxx.List(ctx, db,
			"SELECT * FROM deployments WHERE workspace_id = $1 AND project_id IS NOT DISTINCT FROM $2 AND is_head = true LIMIT 1",
			[]any{wsID.String(), pID.String()},
			pgx.RowToStructByPos[gen.Deployment])
	} else {
		rows, err = pgxx.List(ctx, db,
			"SELECT * FROM deployments WHERE workspace_id = $1 AND project_id IS NULL AND is_head = true LIMIT 1",
			[]any{wsID.String()},
			pgx.RowToStructByPos[gen.Deployment])
	}
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	if len(rows) == 0 {
		return nil, rerror.ErrNotFound
	}
	d, err := deploymentFromRow(rows[0])
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(d.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return d, nil
}

func (r *Deployment) FindVersions(ctx context.Context, wsID accountsid.WorkspaceID, pID *id.ProjectID) ([]*deployment.Deployment, error) {
	db := r.c.DB(ctx)
	var rows []gen.Deployment
	var err error
	if pID != nil {
		rows, err = pgxx.List(ctx, db,
			"SELECT * FROM deployments WHERE workspace_id = $1 AND project_id IS NOT DISTINCT FROM $2 ORDER BY version ASC",
			[]any{wsID.String(), pID.String()},
			pgx.RowToStructByPos[gen.Deployment])
	} else {
		rows, err = pgxx.List(ctx, db,
			"SELECT * FROM deployments WHERE workspace_id = $1 AND project_id IS NULL ORDER BY version ASC",
			[]any{wsID.String()},
			pgx.RowToStructByPos[gen.Deployment])
	}
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	ds := make([]*deployment.Deployment, 0, len(rows))
	for _, row := range rows {
		d, err := deploymentFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(d.Workspace()) {
			ds = append(ds, d)
		}
	}
	return ds, nil
}

func (r *Deployment) Save(ctx context.Context, d *deployment.Deployment) error {
	if !r.f.CanWrite(d.Workspace()) {
		return repo.ErrOperationDenied
	}
	if err := r.q(ctx).UpsertDeployment(ctx, deploymentToParams(d)); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Deployment) Remove(ctx context.Context, did id.DeploymentID) error {
	exec := r.c.DB(ctx)
	if r.f.Writable == nil {
		if _, err := exec.Exec(ctx, `DELETE FROM deployments WHERE id = $1`, did.String()); err != nil {
			return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}
		return nil
	}
	if _, err := exec.Exec(ctx,
		`DELETE FROM deployments WHERE id = $1 AND workspace_id = ANY($2::text[])`,
		did.String(), r.f.Writable.Strings(),
	); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func deploymentsFromRows(rows []gen.Deployment) ([]*deployment.Deployment, error) {
	res := make([]*deployment.Deployment, 0, len(rows))
	for _, row := range rows {
		d, err := deploymentFromRow(row)
		if err != nil {
			return nil, err
		}
		res = append(res, d)
	}
	return res, nil
}

func deploymentToParams(d *deployment.Deployment) gen.UpsertDeploymentParams {
	p := gen.UpsertDeploymentParams{
		ID:          d.ID().String(),
		WorkspaceID: d.Workspace().String(),
		WorkflowUrl: d.WorkflowURL(),
		Description: d.Description(),
		Version:     d.Version(),
		UpdatedAt:   d.UpdatedAt(),
		IsHead:      d.IsHead(),
	}
	if pid := d.Project(); pid != nil {
		s := pid.String()
		p.ProjectID = &s
	}
	if hid := d.HeadID(); hid != nil {
		s := hid.String()
		p.HeadID = &s
	}
	return p
}

func deploymentFromRow(row gen.Deployment) (*deployment.Deployment, error) {
	did, err := id.DeploymentIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	wid, err := accountsid.WorkspaceIDFrom(row.WorkspaceID)
	if err != nil {
		return nil, err
	}

	b := deployment.New().
		ID(did).
		Workspace(wid).
		WorkflowURL(row.WorkflowUrl).
		Description(row.Description).
		Version(row.Version).
		UpdatedAt(row.UpdatedAt).
		IsHead(row.IsHead)

	if row.ProjectID != nil {
		pid, err := id.ProjectIDFrom(*row.ProjectID)
		if err != nil {
			return nil, err
		}
		b = b.Project(&pid)
	}
	if row.HeadID != nil {
		hid, err := id.DeploymentIDFrom(*row.HeadID)
		if err != nil {
			return nil, err
		}
		b = b.HeadID(&hid)
	}
	return b.Build()
}
