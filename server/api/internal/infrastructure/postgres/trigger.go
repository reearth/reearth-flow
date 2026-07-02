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
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type Trigger struct {
	c *pgxx.Client
	f repo.WorkspaceFilter
}

var _ repo.Trigger = (*Trigger)(nil)

func NewTrigger(c *pgxx.Client) *Trigger {
	return &Trigger{c: c}
}

func (r *Trigger) Filtered(f repo.WorkspaceFilter) repo.Trigger {
	return &Trigger{c: r.c, f: r.f.Merge(f)}
}

func (r *Trigger) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *Trigger) FindByID(ctx context.Context, tid id.TriggerID) (*trigger.Trigger, error) {
	row, err := r.q(ctx).GetTrigger(ctx, tid.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	t, err := triggerFromRow(row)
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(t.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return t, nil
}

func (r *Trigger) FindByIDs(ctx context.Context, ids id.TriggerIDList) ([]*trigger.Trigger, error) {
	rows, err := r.q(ctx).ListTriggersByIDs(ctx, ids.Strings())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	ts := make([]*trigger.Trigger, 0, len(rows))
	for _, row := range rows {
		t, err := triggerFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(t.Workspace()) {
			ts = append(ts, t)
		}
	}
	return pgxx.OrderByIDs(ids.Strings(), ts, func(t *trigger.Trigger) string { return t.ID().String() }), nil
}

func (r *Trigger) FindByDeployment(ctx context.Context, did id.DeploymentID) ([]*trigger.Trigger, error) {
	rows, err := r.q(ctx).ListTriggersByDeployment(ctx, did.String())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	res := make([]*trigger.Trigger, 0, len(rows))
	for _, row := range rows {
		t, err := triggerFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(t.Workspace()) {
			res = append(res, t)
		}
	}
	return res, nil
}

var triggerOrderByColumns = map[string]string{
	"description": "description",
	"createdAt":   "created_at",
	"updatedAt":   "updated_at",
	"id":          "id",
}

func (r *Trigger) FindByWorkspace(
	ctx context.Context,
	wid accountsid.WorkspaceID,
	pagination *interfaces.PaginationParam,
	keyword *string,
) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(wid) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	where := []string{"workspace_id = $1"}
	args := []any{wid.String()}
	if keyword != nil && *keyword != "" {
		args = append(args, "%"+*keyword+"%")
		where = append(where, fmt.Sprintf("(description ILIKE $%d OR id ILIKE $%d)", len(args), len(args)))
	}
	whereSQL := "WHERE " + strings.Join(where, " AND ")
	db := r.c.DB(ctx)

	if pagination != nil && pagination.Page != nil {
		p := pagination.Page
		orderCol := "updated_at"
		dir := "DESC"
		if p.OrderBy != nil {
			if c, ok := triggerOrderByColumns[*p.OrderBy]; ok {
				orderCol = c
			}
			dir = "ASC"
			if p.OrderDir != nil && strings.EqualFold(*p.OrderDir, "DESC") {
				dir = "DESC"
			}
		}

		// RowToStructByPos is valid here because SELECT * returns columns in
		// table-definition order, matching gen.Trigger's field order.
		listSQL := fmt.Sprintf("SELECT * FROM triggers %s ORDER BY %s %s", whereSQL, orderCol, dir)
		rows, total, err := pgxx.CountAndList(ctx, db,
			"SELECT count(*) FROM triggers "+whereSQL, listSQL, args,
			int64(p.PageSize), int64((p.Page-1)*p.PageSize),
			pgx.RowToStructByPos[gen.Trigger])
		if err != nil {
			return nil, nil, err
		}
		ts, err := triggersFromRows(rows)
		if err != nil {
			return nil, nil, err
		}
		return ts, interfaces.NewPageBasedInfo(total, p.Page, p.PageSize), nil
	}

	rows, err := pgxx.List(ctx, db,
		"SELECT * FROM triggers "+whereSQL+" ORDER BY updated_at DESC", args,
		pgx.RowToStructByPos[gen.Trigger])
	if err != nil {
		return nil, nil, err
	}
	ts, err := triggersFromRows(rows)
	if err != nil {
		return nil, nil, err
	}
	return ts, interfaces.NewPageBasedInfo(int64(len(ts)), 1, len(ts)), nil
}

func (r *Trigger) Save(ctx context.Context, t *trigger.Trigger) error {
	if !r.f.CanWrite(t.Workspace()) {
		return repo.ErrOperationDenied
	}
	params, err := triggerToParams(t)
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	if err := r.q(ctx).UpsertTrigger(ctx, params); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Trigger) Remove(ctx context.Context, tid id.TriggerID) error {
	exec := r.c.DB(ctx)
	if r.f.Writable == nil {
		if _, err := exec.Exec(ctx, `DELETE FROM triggers WHERE id = $1`, tid.String()); err != nil {
			return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}
		return nil
	}
	if _, err := exec.Exec(ctx,
		`DELETE FROM triggers WHERE id = $1 AND workspace_id = ANY($2::text[])`,
		tid.String(), r.f.Writable.Strings(),
	); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func triggersFromRows(rows []gen.Trigger) ([]*trigger.Trigger, error) {
	res := make([]*trigger.Trigger, 0, len(rows))
	for _, row := range rows {
		t, err := triggerFromRow(row)
		if err != nil {
			return nil, err
		}
		res = append(res, t)
	}
	return res, nil
}

func triggerToParams(t *trigger.Trigger) (gen.UpsertTriggerParams, error) {
	vars, err := variablesToJSON(t.Variables())
	if err != nil {
		return gen.UpsertTriggerParams{}, err
	}
	p := gen.UpsertTriggerParams{
		ID:           t.ID().String(),
		WorkspaceID:  t.Workspace().String(),
		DeploymentID: t.Deployment().String(),
		Description:  t.Description(),
		EventSource:  string(t.EventSource()),
		Enabled:      t.Enabled(),
		Variables:    vars,
		CreatedAt:    t.CreatedAt(),
		UpdatedAt:    t.UpdatedAt(),
	}
	if ti := t.TimeInterval(); ti != nil {
		s := string(*ti)
		p.TimeInterval = &s
	}
	if at := t.AuthToken(); at != nil {
		p.AuthToken = at
	}
	if lt := t.LastTriggered(); lt != nil {
		p.LastTriggered = lt
	}
	return p, nil
}

func triggerFromRow(row gen.Trigger) (*trigger.Trigger, error) {
	tid, err := id.TriggerIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	wid, err := accountsid.WorkspaceIDFrom(row.WorkspaceID)
	if err != nil {
		return nil, err
	}
	did, err := id.DeploymentIDFrom(row.DeploymentID)
	if err != nil {
		return nil, err
	}

	b := trigger.New().
		ID(tid).
		Workspace(wid).
		Deployment(did).
		Description(row.Description).
		EventSource(trigger.EventSourceType(row.EventSource)).
		Enabled(row.Enabled).
		CreatedAt(row.CreatedAt).
		UpdatedAt(row.UpdatedAt)

	if row.TimeInterval != nil {
		b = b.TimeInterval(trigger.TimeInterval(*row.TimeInterval))
	}
	if row.AuthToken != nil {
		b = b.AuthToken(*row.AuthToken)
	}
	if row.LastTriggered != nil {
		b = b.LastTriggered(*row.LastTriggered)
	}

	vars, err := variablesFromJSON(row.Variables)
	if err != nil {
		return nil, err
	}
	if len(vars) > 0 {
		b = b.Variables(vars)
	}

	return b.Build()
}
