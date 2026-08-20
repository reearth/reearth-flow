package postgres

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type ProjectAccess struct {
	c *pgxx.Client
}

var _ repo.ProjectAccess = (*ProjectAccess)(nil)

func NewProjectAccess(c *pgxx.Client) *ProjectAccess {
	return &ProjectAccess{c: c}
}

func (r *ProjectAccess) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *ProjectAccess) FindByProjectID(ctx context.Context, pid id.ProjectID) (*projectAccess.ProjectAccess, error) {
	row, err := r.q(ctx).GetProjectAccessByProjectID(ctx, pid.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	return projectAccessFromRow(row)
}

func (r *ProjectAccess) FindByToken(ctx context.Context, token string) (*projectAccess.ProjectAccess, error) {
	row, err := r.q(ctx).GetProjectAccessByToken(ctx, token)
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	return projectAccessFromRow(row)
}

func (r *ProjectAccess) Save(ctx context.Context, pa *projectAccess.ProjectAccess) error {
	if err := r.q(ctx).UpsertProjectAccess(ctx, projectAccessToParams(pa)); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func projectAccessToParams(pa *projectAccess.ProjectAccess) gen.UpsertProjectAccessParams {
	return gen.UpsertProjectAccessParams{
		ID:        pa.ID().String(),
		ProjectID: pa.Project().String(),
		Token:     pa.Token(),
		IsPublic:  pa.IsPublic(),
	}
}

func projectAccessFromRow(row gen.ProjectAccess) (*projectAccess.ProjectAccess, error) {
	paid, err := id.ProjectAccessIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	pid, err := id.ProjectIDFrom(row.ProjectID)
	if err != nil {
		return nil, err
	}
	return projectAccess.New().
		ID(paid).
		Project(pid).
		Token(row.Token).
		IsPublic(row.IsPublic).
		Build()
}
