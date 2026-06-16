package postgres

import (
	"context"
	"encoding/json"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type Parameter struct {
	c *pgxx.Client
}

var _ repo.Parameter = (*Parameter)(nil)

func NewParameter(c *pgxx.Client) *Parameter {
	return &Parameter{c: c}
}

func (r *Parameter) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *Parameter) FindByID(ctx context.Context, pid id.ParameterID) (*parameter.Parameter, error) {
	row, err := r.q(ctx).GetParameter(ctx, pid.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	return parameterFromRow(row)
}

func (r *Parameter) FindByIDs(ctx context.Context, ids id.ParameterIDList) (*parameter.ParameterList, error) {
	if len(ids) == 0 {
		return nil, nil
	}
	rows, err := r.q(ctx).ListParametersByIDs(ctx, ids.Strings())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	ps := make([]*parameter.Parameter, 0, len(rows))
	for _, row := range rows {
		p, err := parameterFromRow(row)
		if err != nil {
			return nil, err
		}
		ps = append(ps, p)
	}
	ordered := pgxx.OrderByIDs(ids.Strings(), ps, func(p *parameter.Parameter) string { return p.ID().String() })
	return parameter.NewParameterList(ordered), nil
}

func (r *Parameter) FindByProject(ctx context.Context, pid id.ProjectID) (*parameter.ParameterList, error) {
	rows, err := r.q(ctx).ListParametersByProject(ctx, pid.String())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	if len(rows) == 0 {
		return nil, nil
	}
	res := make([]*parameter.Parameter, 0, len(rows))
	for _, row := range rows {
		p, err := parameterFromRow(row)
		if err != nil {
			return nil, err
		}
		res = append(res, p)
	}
	return parameter.NewParameterList(res), nil
}

func (r *Parameter) Save(ctx context.Context, p *parameter.Parameter) error {
	params, err := parameterToParams(p)
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	if err := r.q(ctx).UpsertParameter(ctx, params); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Parameter) Remove(ctx context.Context, pid id.ParameterID) error {
	if err := r.q(ctx).DeleteParameter(ctx, pid.String()); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Parameter) RemoveAll(ctx context.Context, ids id.ParameterIDList) error {
	if len(ids) == 0 {
		return nil
	}
	if err := r.q(ctx).DeleteParametersByIDs(ctx, ids.Strings()); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Parameter) RemoveAllByProject(ctx context.Context, pid id.ProjectID) error {
	if err := r.q(ctx).DeleteParametersByProject(ctx, pid.String()); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func parameterToParams(p *parameter.Parameter) (gen.UpsertParameterParams, error) {
	var defaultValue []byte
	if dv := p.DefaultValue(); dv != nil {
		b, err := json.Marshal(dv)
		if err != nil {
			return gen.UpsertParameterParams{}, err
		}
		defaultValue = b
	}
	var config []byte
	if cfg := p.Config(); cfg != nil {
		b, err := json.Marshal(cfg)
		if err != nil {
			return gen.UpsertParameterParams{}, err
		}
		config = b
	}
	return gen.UpsertParameterParams{
		ID:           p.ID().String(),
		ProjectID:    p.ProjectID().String(),
		Name:         p.Name(),
		Type:         string(p.Type()),
		Index:        int32(p.Index()),
		Required:     p.Required(),
		Public:       p.Public(),
		DefaultValue: defaultValue,
		Config:       config,
		CreatedAt:    p.CreatedAt(),
		UpdatedAt:    p.UpdatedAt(),
	}, nil
}

func parameterFromRow(row gen.Parameter) (*parameter.Parameter, error) {
	pid, err := id.ParameterIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	projID, err := id.ProjectIDFrom(row.ProjectID)
	if err != nil {
		return nil, err
	}

	var defaultValue interface{}
	if len(row.DefaultValue) > 0 {
		if err := json.Unmarshal(row.DefaultValue, &defaultValue); err != nil {
			return nil, err
		}
	}
	var config interface{}
	if len(row.Config) > 0 {
		if err := json.Unmarshal(row.Config, &config); err != nil {
			return nil, err
		}
	}

	return parameter.New().
		ID(pid).
		ProjectID(projID).
		Name(row.Name).
		Type(parameter.Type(row.Type)).
		Index(int(row.Index)).
		Required(row.Required).
		Public(row.Public).
		DefaultValue(defaultValue).
		Config(config).
		CreatedAt(row.CreatedAt).
		UpdatedAt(row.UpdatedAt).
		Build()
}
