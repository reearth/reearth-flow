package postgres

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type Workflow struct {
	c *pgxx.Client
	f repo.WorkspaceFilter
}

var _ repo.Workflow = (*Workflow)(nil)

func NewWorkflow(c *pgxx.Client) *Workflow {
	return &Workflow{c: c}
}

func (r *Workflow) Filtered(f repo.WorkspaceFilter) repo.Workflow {
	return &Workflow{c: r.c, f: r.f.Merge(f)}
}

func (r *Workflow) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *Workflow) FindByID(ctx context.Context, wid id.WorkflowID) (*workflow.Workflow, error) {
	row, err := r.q(ctx).GetWorkflow(ctx, wid.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	wf, err := workflowFromRow(row)
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(wf.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return wf, nil
}

func (r *Workflow) Save(ctx context.Context, wf *workflow.Workflow) error {
	if !r.f.CanWrite(wf.Workspace()) {
		return repo.ErrOperationDenied
	}
	if err := r.q(ctx).UpsertWorkflow(ctx, workflowToParams(wf)); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Workflow) Remove(ctx context.Context, wid id.WorkflowID) error {
	exec := r.c.DB(ctx)
	if r.f.Writable == nil {
		if err := r.q(ctx).DeleteWorkflow(ctx, wid.String()); err != nil {
			return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}
		return nil
	}
	if _, err := exec.Exec(ctx,
		`DELETE FROM workflows WHERE id = $1 AND workspace_id = ANY($2::text[])`,
		wid.String(), r.f.Writable.Strings(),
	); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func workflowToParams(wf *workflow.Workflow) gen.UpsertWorkflowParams {
	return gen.UpsertWorkflowParams{
		ID:          wf.ID().String(),
		ProjectID:   wf.Project().String(),
		WorkspaceID: wf.Workspace().String(),
		Url:         wf.URL(),
	}
}

func workflowFromRow(row gen.Workflow) (*workflow.Workflow, error) {
	wid, err := id.WorkflowIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	pid, err := id.ProjectIDFrom(row.ProjectID)
	if err != nil {
		return nil, err
	}
	wsid, err := accountsid.WorkspaceIDFrom(row.WorkspaceID)
	if err != nil {
		return nil, err
	}
	return workflow.NewWorkflow(wid, pid, wsid, row.Url), nil
}
