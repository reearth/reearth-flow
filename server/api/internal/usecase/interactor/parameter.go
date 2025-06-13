package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type Parameter struct {
	paramRepo         repo.Parameter
	projectRepo       repo.Project
	transaction       usecasex.Transaction
	permissionChecker gateway.PermissionChecker
}

func NewParameter(r *repo.Container, permissionChecker gateway.PermissionChecker) interfaces.Parameter {
	return &Parameter{
		paramRepo:         r.Parameter,
		projectRepo:       r.Project,
		transaction:       r.Transaction,
		permissionChecker: permissionChecker,
	}
}

func (i *Parameter) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceParameter, action)
}

func (i *Parameter) DeclareParameter(ctx context.Context, param interfaces.DeclareParameterParam) (*parameter.Parameter, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

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
		Public(param.Public).
		DefaultValue(param.DefaultValue).
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

func (i *Parameter) Fetch(ctx context.Context, ids id.ParameterIDList) (*parameter.ParameterList, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.paramRepo.FindByIDs(ctx, ids)
}

func (i *Parameter) FetchByProject(ctx context.Context, pid id.ProjectID) (*parameter.ParameterList, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	params, err := i.paramRepo.FindByProject(ctx, pid)
	return params, err
}

func (i *Parameter) RemoveParameter(ctx context.Context, pid id.ParameterID) (id.ParameterID, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return pid, err
	}

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

func (i *Parameter) UpdateParameterOrder(ctx context.Context, param interfaces.UpdateParameterOrderParam) (*parameter.ParameterList, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

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
		switch {
		case p.ID() == param.ParamID:
			p.SetIndex(newIndex)
		case currentIndex < newIndex && p.Index() > currentIndex && p.Index() <= newIndex:
			p.SetIndex(p.Index() - 1)
		case currentIndex > newIndex && p.Index() >= newIndex && p.Index() < currentIndex:
			p.SetIndex(p.Index() + 1)
		}

		if err := i.paramRepo.Save(ctx, p); err != nil {
			return nil, err
		}
	}

	tx.Commit()
	return params, nil
}

func (i *Parameter) UpdateParameter(ctx context.Context, param interfaces.UpdateParameterParam) (*parameter.Parameter, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

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

	p.SetDefaultValue(param.DefaultValue)
	p.SetName(param.NameValue)
	p.SetType(param.TypeValue)
	p.SetRequired(param.RequiredValue)
	p.SetPublic(param.PublicValue)

	if err := i.paramRepo.Save(ctx, p); err != nil {
		return nil, err
	}

	tx.Commit()
	return p, nil
}
