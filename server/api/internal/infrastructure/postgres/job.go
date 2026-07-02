package postgres

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/jackc/pgx/v5"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type Job struct {
	c *pgxx.Client
	f repo.WorkspaceFilter
}

var _ repo.Job = (*Job)(nil)

func NewJob(c *pgxx.Client) *Job {
	return &Job{c: c}
}

func (r *Job) Filtered(f repo.WorkspaceFilter) repo.Job {
	return &Job{c: r.c, f: r.f.Merge(f)}
}

func (r *Job) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *Job) FindByID(ctx context.Context, jid id.JobID) (*job.Job, error) {
	row, err := r.q(ctx).GetJob(ctx, jid.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	j, err := jobFromRow(row)
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(j.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return j, nil
}

func (r *Job) FindByIDs(ctx context.Context, ids id.JobIDList) ([]*job.Job, error) {
	strs := jobIDListStrings(ids)
	rows, err := r.q(ctx).ListJobsByIDs(ctx, strs)
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	js := make([]*job.Job, 0, len(rows))
	for _, row := range rows {
		j, err := jobFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(j.Workspace()) {
			js = append(js, j)
		}
	}
	return pgxx.OrderByIDs(strs, js, func(j *job.Job) string { return j.ID().String() }), nil
}

func jobIDListStrings(ids id.JobIDList) []string {
	s := make([]string, len(ids))
	for i, v := range ids {
		s[i] = v.String()
	}
	return s
}

var jobOrderByColumns = map[string]string{
	"startedAt":   "started_at",
	"completedAt": "completed_at",
	"status":      "status",
	"id":          "id",
}

// FindByWorkspace returns non-debug jobs only (mirrors Mongo: debug IS NOT TRUE).
func (r *Job) FindByWorkspace(
	ctx context.Context,
	wid accountsid.WorkspaceID,
	pagination *interfaces.PaginationParam,
	keyword *string,
) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(wid) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	where := []string{"workspace_id = $1", "(debug IS NULL OR debug = false)"}
	args := []any{wid.String()}
	if keyword != nil && *keyword != "" {
		args = append(args, "%"+*keyword+"%")
		where = append(where, fmt.Sprintf(
			"(id ILIKE $%d OR status ILIKE $%d)",
			len(args), len(args),
		))
	}
	whereSQL := "WHERE " + strings.Join(where, " AND ")
	db := r.c.DB(ctx)

	if pagination != nil && pagination.Page != nil {
		p := pagination.Page
		orderCol := "started_at"
		dir := "DESC"
		if p.OrderBy != nil {
			if c, ok := jobOrderByColumns[*p.OrderBy]; ok {
				orderCol = c
			}
			dir = "ASC"
			if p.OrderDir != nil && strings.EqualFold(*p.OrderDir, "DESC") {
				dir = "DESC"
			}
		}
		listSQL := fmt.Sprintf("SELECT * FROM jobs %s ORDER BY %s %s", whereSQL, orderCol, dir)
		rows, total, err := pgxx.CountAndList(ctx, db,
			"SELECT count(*) FROM jobs "+whereSQL, listSQL, args,
			int64(p.PageSize), int64((p.Page-1)*p.PageSize),
			pgx.RowToStructByPos[gen.Job])
		if err != nil {
			return nil, nil, err
		}
		js, err := jobsFromRows(rows)
		if err != nil {
			return nil, nil, err
		}
		return js, interfaces.NewPageBasedInfo(total, p.Page, p.PageSize), nil
	}

	rows, err := pgxx.List(ctx, db,
		"SELECT * FROM jobs "+whereSQL+" ORDER BY started_at DESC", args,
		pgx.RowToStructByPos[gen.Job])
	if err != nil {
		return nil, nil, err
	}
	js, err := jobsFromRows(rows)
	if err != nil {
		return nil, nil, err
	}
	return js, interfaces.NewPageBasedInfo(int64(len(js)), 1, len(js)), nil
}

// FindByProject returns debug jobs for a project (mirrors Mongo: {projectid, debug:true}).
func (r *Job) FindByProject(ctx context.Context, pid id.ProjectID) ([]*job.Job, error) {
	db := r.c.DB(ctx)
	// Exclude preview-schema jobs from run history: they are persisted (debug=true)
	// but are not real runs. Legacy rows predate the mode column (default ''), so
	// `mode <> 'preview-schema'` keeps them. Mirrors the Mongo FindByProject filter.
	rows, err := pgxx.List(ctx, db,
		"SELECT * FROM jobs WHERE project_id = $1 AND debug = true AND mode <> 'preview-schema'",
		[]any{pid.String()},
		pgx.RowToStructByPos[gen.Job])
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	js := make([]*job.Job, 0, len(rows))
	for _, row := range rows {
		j, err := jobFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(j.Workspace()) {
			js = append(js, j)
		}
	}
	return js, nil
}

// RemoveByProject deletes debug jobs for a project (mirrors Mongo: {projectid, debug:true}).
func (r *Job) RemoveByProject(ctx context.Context, pid id.ProjectID) error {
	s := pid.String()
	if err := r.q(ctx).DeleteJobsByProject(ctx, &s); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Job) Save(ctx context.Context, j *job.Job) error {
	if !r.f.CanWrite(j.Workspace()) {
		return repo.ErrOperationDenied
	}
	params, err := jobToParams(j)
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	if err := r.q(ctx).UpsertJob(ctx, params); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Job) Remove(ctx context.Context, jid id.JobID) error {
	exec := r.c.DB(ctx)
	if r.f.Writable == nil {
		if _, err := exec.Exec(ctx, `DELETE FROM jobs WHERE id = $1`, jid.String()); err != nil {
			return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}
		return nil
	}
	if _, err := exec.Exec(ctx,
		`DELETE FROM jobs WHERE id = $1 AND workspace_id = ANY($2::text[])`,
		jid.String(), r.f.Writable.Strings(),
	); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

// jobParameterJSON is the wire format for parameters stored in JSONB.
type jobParameterJSON struct {
	DefaultValue interface{} `json:"default_value,omitempty"`
	Config       interface{} `json:"config,omitempty"`
	ID           string      `json:"id"`
	Name         string      `json:"name"`
	Type         string      `json:"type"`
	ProjectID    string      `json:"project_id"`
	Index        int         `json:"index"`
	Required     bool        `json:"required"`
	Public       bool        `json:"public"`
}

func jobsFromRows(rows []gen.Job) ([]*job.Job, error) {
	res := make([]*job.Job, 0, len(rows))
	for _, row := range rows {
		j, err := jobFromRow(row)
		if err != nil {
			return nil, err
		}
		res = append(res, j)
	}
	return res, nil
}

func jobToParams(j *job.Job) (gen.UpsertJobParams, error) {
	p := gen.UpsertJobParams{
		ID:                j.ID().String(),
		WorkspaceID:       j.Workspace().String(),
		GcpJobID:          j.GCPJobID(),
		LogsUrl:           j.LogsURL(),
		WorkerLogsUrl:     j.WorkerLogsURL(),
		UserFacingLogsUrl: j.UserFacingLogsURL(),
		Status:            string(j.Status()),
		StartedAt:         j.StartedAt(),
		CompletedAt:       j.CompletedAt(),
		MetadataUrl:       j.MetadataURL(),
		Debug:             j.Debug(),
		Mode:              string(j.Mode()),
	}
	if did := j.Deployment(); did != nil {
		s := did.String()
		p.DeploymentID = &s
	}
	if pid := j.ProjectID(); pid != nil {
		s := pid.String()
		p.ProjectID = &s
	}
	if pv := j.ProjectVersion(); pv != nil {
		v := int32(*pv)
		p.ProjectVersion = &v
	}
	if bs := j.BatchStatus(); bs != nil {
		s := string(*bs)
		p.BatchStatus = &s
	}
	if ws := j.WorkerStatus(); ws != nil {
		s := string(*ws)
		p.WorkerStatus = &s
	}
	if urls := j.OutputURLs(); len(urls) > 0 {
		b, err := json.Marshal(urls)
		if err != nil {
			return gen.UpsertJobParams{}, err
		}
		p.OutputUrls = b
	}
	if params := j.Parameters(); len(params) > 0 {
		docs := make([]jobParameterJSON, 0, len(params))
		for _, param := range params {
			if param == nil {
				continue
			}
			docs = append(docs, jobParameterJSON{
				ID:           param.ID().String(),
				ProjectID:    param.ProjectID().String(),
				Name:         param.Name(),
				Type:         string(param.Type()),
				Index:        param.Index(),
				Required:     param.Required(),
				Public:       param.Public(),
				DefaultValue: param.DefaultValue(),
				Config:       param.Config(),
			})
		}
		b, err := json.Marshal(docs)
		if err != nil {
			return gen.UpsertJobParams{}, err
		}
		p.Parameters = b
	}
	return p, nil
}

func jobFromRow(row gen.Job) (*job.Job, error) {
	jid, err := id.JobIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	wid, err := accountsid.WorkspaceIDFrom(row.WorkspaceID)
	if err != nil {
		return nil, err
	}

	b := job.New().
		ID(jid).
		Workspace(wid).
		GCPJobID(row.GcpJobID).
		LogsURL(row.LogsUrl).
		WorkerLogsURL(row.WorkerLogsUrl).
		UserFacingLogsURL(row.UserFacingLogsUrl).
		Status(job.Status(row.Status)).
		StartedAt(row.StartedAt).
		CompletedAt(row.CompletedAt).
		MetadataURL(row.MetadataUrl).
		Debug(row.Debug)

	if row.DeploymentID != nil {
		did, err := id.DeploymentIDFrom(*row.DeploymentID)
		if err != nil {
			return nil, err
		}
		b = b.Deployment(&did)
	}
	if row.ProjectID != nil {
		pid, err := id.ProjectIDFrom(*row.ProjectID)
		if err != nil {
			return nil, err
		}
		b = b.ProjectID(&pid)
	}
	if row.ProjectVersion != nil {
		v := int(*row.ProjectVersion)
		b = b.ProjectVersion(&v)
	}
	if row.BatchStatus != nil {
		s := job.Status(*row.BatchStatus)
		b = b.BatchStatus(&s)
	}
	if row.WorkerStatus != nil {
		s := job.Status(*row.WorkerStatus)
		b = b.WorkerStatus(&s)
	}
	if len(row.OutputUrls) > 0 {
		var urls []string
		if err := json.Unmarshal(row.OutputUrls, &urls); err != nil {
			return nil, err
		}
		b = b.OutputURLs(urls)
	}
	if len(row.Parameters) > 0 {
		var docs []jobParameterJSON
		if err := json.Unmarshal(row.Parameters, &docs); err != nil {
			return nil, err
		}
		params := make([]*parameter.Parameter, 0, len(docs))
		for _, doc := range docs {
			pid, err := id.ParameterIDFrom(doc.ID)
			if err != nil {
				return nil, err
			}
			projID, err := id.ProjectIDFrom(doc.ProjectID)
			if err != nil {
				return nil, err
			}
			p, err := parameter.New().
				ID(pid).
				ProjectID(projID).
				Name(doc.Name).
				Type(parameter.Type(doc.Type)).
				Index(doc.Index).
				Required(doc.Required).
				Public(doc.Public).
				DefaultValue(doc.DefaultValue).
				Config(doc.Config).
				Build()
			if err != nil {
				return nil, err
			}
			params = append(params, p)
		}
		b = b.Parameters(params)
	}
	if row.Mode != "" {
		b = b.Mode(job.Mode(row.Mode))
	}
	return b.Build()
}
