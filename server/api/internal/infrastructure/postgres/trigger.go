package postgres

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgtype"
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
	pool pgxx.DBTX
	f    repo.WorkspaceFilter
}

func NewTrigger(pool pgxx.DBTX) *Trigger {
	return &Trigger{pool: pool}
}

func (r *Trigger) Filtered(f repo.WorkspaceFilter) repo.Trigger {
	return &Trigger{pool: r.pool, f: r.f.Merge(f)}
}

func (r *Trigger) q(ctx context.Context) *gen.Queries {
	return gen.New(pgxx.Executor(ctx, r.pool))
}

func (r *Trigger) FindByID(ctx context.Context, tid id.TriggerID) (*trigger.Trigger, error) {
	row, err := r.q(ctx).GetTrigger(ctx, tid.String())
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, rerror.ErrNotFound
		}
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
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
	byID := make(map[string]*trigger.Trigger, len(rows))
	for _, row := range rows {
		t, err := triggerFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(t.Workspace()) {
			byID[t.ID().String()] = t
		}
	}
	res := make([]*trigger.Trigger, 0, len(ids))
	for _, tid := range ids {
		res = append(res, byID[tid.String()])
	}
	return res, nil
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
	exec := pgxx.Executor(ctx, r.pool)

	if pagination != nil && pagination.Page != nil {
		p := pagination.Page
		var total int64
		if err := exec.QueryRow(ctx, "SELECT count(*) FROM triggers "+whereSQL, args...).Scan(&total); err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}

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

		query := fmt.Sprintf("SELECT * FROM triggers %s ORDER BY %s %s LIMIT $%d OFFSET $%d",
			whereSQL, orderCol, dir, len(args)+1, len(args)+2)
		args = append(args, p.PageSize, (p.Page-1)*p.PageSize)

		ts, err := r.queryTriggers(ctx, exec, query, args)
		if err != nil {
			return nil, nil, err
		}
		return ts, interfaces.NewPageBasedInfo(total, p.Page, p.PageSize), nil
	}

	ts, err := r.queryTriggers(ctx, exec, "SELECT * FROM triggers "+whereSQL+" ORDER BY updated_at DESC", args)
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
	exec := pgxx.Executor(ctx, r.pool)
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

func (r *Trigger) queryTriggers(ctx context.Context, exec pgxx.DBTX, query string, args []any) ([]*trigger.Trigger, error) {
	rows, err := exec.Query(ctx, query, args...)
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	defer rows.Close()

	genRows, err := pgx.CollectRows(rows, pgx.RowToStructByPos[gen.Trigger])
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	res := make([]*trigger.Trigger, 0, len(genRows))
	for _, row := range genRows {
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
		CreatedAt:    pgtype.Timestamptz{Time: t.CreatedAt(), Valid: true},
		UpdatedAt:    pgtype.Timestamptz{Time: t.UpdatedAt(), Valid: true},
	}
	if ti := t.TimeInterval(); ti != nil {
		p.TimeInterval = pgtype.Text{String: string(*ti), Valid: true}
	}
	if at := t.AuthToken(); at != nil {
		p.AuthToken = pgtype.Text{String: *at, Valid: true}
	}
	if lt := t.LastTriggered(); lt != nil {
		p.LastTriggered = pgtype.Timestamptz{Time: *lt, Valid: true}
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
		CreatedAt(row.CreatedAt.Time).
		UpdatedAt(row.UpdatedAt.Time)

	if row.TimeInterval.Valid {
		b = b.TimeInterval(trigger.TimeInterval(row.TimeInterval.String))
	}
	if row.AuthToken.Valid {
		b = b.AuthToken(row.AuthToken.String)
	}
	if row.LastTriggered.Valid {
		b = b.LastTriggered(row.LastTriggered.Time)
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
