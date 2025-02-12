package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type Parameter struct {
	common
	paramRepo   repo.Parameter
	projectRepo repo.Project
	transaction usecasex.Transaction
}

func NewParameter(r *repo.Container) interfaces.Parameter {
	return &Parameter{
		paramRepo:   r.Parameter,
		projectRepo: r.Project,
		transaction: r.Transaction,
	}
}

func (i *Parameter) DeclareParameter(ctx context.Context, param interfaces.DeclareParameterParam, operator *usecase.Operator) (*parameter.Parameter, error) {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	// Check if project exists
	proj, err := i.projectRepo.FindByID(ctx, param.ProjectID)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}

	// Get next index if not specified
	var index int
	if param.Index == nil {
		params, err := i.paramRepo.FindByProject(ctx, param.ProjectID)
		if err != nil {
			return nil, err
		}
		if params != nil {
			index = params.MaxIndex() + 1
		}
	} else {
		index = *param.Index
	}

	// Create parameter
	p, err := parameter.New().
		ProjectID(param.ProjectID).
		Name(param.Name).
		Type(param.Type).
		Required(param.Required).
		Value(param.Value).
		Index(index).
		Build()
	if err != nil {
		return nil, err
	}

	if err := i.paramRepo.Save(ctx, p); err != nil {
		return nil, err
	}

	tx.Commit()
	return p, nil
}

func (i *Parameter) Fetch(ctx context.Context, ids id.ParameterIDList, operator *usecase.Operator) (*parameter.ParameterList, error) {
	return i.paramRepo.FindByIDs(ctx, ids)
}

func (i *Parameter) FetchByProject(ctx context.Context, pid id.ProjectID, operator *usecase.Operator) (*parameter.ParameterList, error) {
	params, err := i.paramRepo.FindByProject(ctx, pid)
	return params, err
}

func (i *Parameter) RemoveParameter(ctx context.Context, pid id.ParameterID, operator *usecase.Operator) (id.ParameterID, error) {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return pid, err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	p, err := i.paramRepo.FindByID(ctx, pid)
	if err != nil {
		return pid, err
	}
	if p == nil {
		return pid, rerror.ErrNotFound
	}

	if err := i.paramRepo.Remove(ctx, pid); err != nil {
		return pid, err
	}

	// Update indices of remaining parameters
	params, err := i.paramRepo.FindByProject(ctx, p.ProjectID())
	if err != nil {
		return pid, err
	}

	for _, param := range *params {
		if param.Index() > p.Index() {
			param.SetIndex(param.Index() - 1)
			if err := i.paramRepo.Save(ctx, param); err != nil {
				return pid, err
			}
		}
	}

	tx.Commit()
	return pid, nil
}

func (i *Parameter) UpdateParameterOrder(ctx context.Context, param interfaces.UpdateParameterOrderParam, operator *usecase.Operator) (*parameter.ParameterList, error) {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	params, err := i.paramRepo.FindByProject(ctx, param.ProjectID)
	if err != nil {
		return nil, err
	}
	if params == nil {
		return nil, rerror.ErrNotFound
	}

	targetParam := params.FindByID(param.ParamID)
	if targetParam == nil {
		return nil, rerror.ErrNotFound
	}

	// Update indices
	currentIndex := targetParam.Index()
	newIndex := param.NewIndex

	// Reorder parameters
	for _, p := range *params {
		if p.ID() == param.ParamID {
			p.SetIndex(newIndex)
		} else if currentIndex < newIndex && p.Index() > currentIndex && p.Index() <= newIndex {
			p.SetIndex(p.Index() - 1)
		} else if currentIndex > newIndex && p.Index() >= newIndex && p.Index() < currentIndex {
			p.SetIndex(p.Index() + 1)
		}

		if err := i.paramRepo.Save(ctx, p); err != nil {
			return nil, err
		}
	}

	tx.Commit()
	return params, nil
}

func (i *Parameter) UpdateParameterValue(ctx context.Context, param interfaces.UpdateParameterValueParam, operator *usecase.Operator) (*parameter.Parameter, error) {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	p, err := i.paramRepo.FindByID(ctx, param.ParamID)
	if err != nil {
		return nil, err
	}
	if p == nil {
		return nil, rerror.ErrNotFound
	}

	p.SetValue(param.Value)

	if err := i.paramRepo.Save(ctx, p); err != nil {
		return nil, err
	}

	tx.Commit()
	return p, nil
}
