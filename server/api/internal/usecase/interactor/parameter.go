package interactor

import (
	"context"
	"sort"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"

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

func (i *Parameter) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceParameter, action, workspaceID...)
}

func (i *Parameter) DeclareParameter(ctx context.Context, param interfaces.DeclareParameterParam) (*parameter.Parameter, error) {
	proj, err := i.projectRepo.FindByID(ctx, param.ProjectID)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, proj.Workspace()); err != nil {
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
	proj, err = i.projectRepo.FindByID(ctx, param.ProjectID)
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
		Config(param.Config).
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

func (i *Parameter) UpdateParameters(ctx context.Context, param interfaces.UpdateParametersParam) (*parameter.ParameterList, error) {
	proj, err := i.projectRepo.FindByID(ctx, param.ProjectID)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, proj.Workspace()); err != nil {
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

	proj, err = i.projectRepo.FindByID(ctx, param.ProjectID)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}

	if len(param.Deletes) > 0 {
		_, err = i.RemoveParameters(ctx, param.Deletes)
		if err != nil {
			return nil, err
		}
	}

	for _, createParam := range param.Creates {
		createParam.ProjectID = param.ProjectID
		_, err = i.DeclareParameter(ctx, createParam)
		if err != nil {
			return nil, err
		}
	}

	for _, updateParam := range param.Updates {
		_, err = i.UpdateParameter(ctx, interfaces.UpdateParameterParam{
			DefaultValue:  updateParam.DefaultValue,
			Config:        updateParam.Config,
			NameValue:     updateParam.NameValue,
			RequiredValue: updateParam.RequiredValue,
			PublicValue:   updateParam.PublicValue,
			TypeValue:     updateParam.TypeValue,
			ParamID:       updateParam.ParamID,
		})
		if err != nil {
			return nil, err
		}
	}

	for _, reorderParam := range param.Reorders {
		reorderParam.ProjectID = param.ProjectID
		_, err = i.UpdateParameterOrder(ctx, reorderParam)
		if err != nil {
			return nil, err
		}
	}

	finalParams, err := i.paramRepo.FindByProject(ctx, param.ProjectID)
	if err != nil {
		return nil, err
	}

	tx.Commit()
	return finalParams, nil
}

func (i *Parameter) Fetch(ctx context.Context, ids id.ParameterIDList) (*parameter.ParameterList, error) {
	params, err := i.paramRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}

	if params == nil || len(*params) == 0 {
		if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
			return nil, err
		}
		return params, nil
	}

	// single-workspace batch assumption
	proj, err := i.projectRepo.FindByID(ctx, (*params)[0].ProjectID())
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, proj.Workspace()); err != nil {
		return nil, err
	}

	return params, nil
}

func (i *Parameter) FetchByProject(ctx context.Context, pid id.ProjectID) (*parameter.ParameterList, error) {
	proj, err := i.projectRepo.FindByID(ctx, pid)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, proj.Workspace()); err != nil {
		return nil, err
	}

	params, err := i.paramRepo.FindByProject(ctx, pid)
	return params, err
}

func (i *Parameter) RemoveParameter(ctx context.Context, pid id.ParameterID) (id.ParameterID, error) {
	// Use the batch delete method for consistency
	removedIDs, err := i.RemoveParameters(ctx, id.ParameterIDList{pid})
	if err != nil {
		return pid, err
	}
	if len(removedIDs) == 0 {
		return pid, rerror.ErrNotFound
	}
	return removedIDs[0], nil
}

func (i *Parameter) RemoveParameters(ctx context.Context, pids id.ParameterIDList) (id.ParameterIDList, error) {
	if len(pids) == 0 {
		if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
			return nil, err
		}
		return id.ParameterIDList{}, nil
	}

	preParams, err := i.paramRepo.FindByIDs(ctx, pids)
	if err != nil {
		return nil, err
	}
	if preParams == nil || len(*preParams) == 0 {
		return nil, rerror.ErrNotFound
	}
	// single-workspace batch assumption
	proj, err := i.projectRepo.FindByID(ctx, (*preParams)[0].ProjectID())
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, proj.Workspace()); err != nil {
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

	// Fetch all parameters to be deleted to validate they exist and get project info
	paramsToDelete, err := i.paramRepo.FindByIDs(ctx, pids)
	if err != nil {
		return nil, err
	}
	if paramsToDelete == nil || len(*paramsToDelete) == 0 {
		return nil, rerror.ErrNotFound
	}

	// Validate all parameters belong to the same project
	var projectID id.ProjectID
	deleteIndexes := make(map[int]bool)
	for i, param := range *paramsToDelete {
		if i == 0 {
			projectID = param.ProjectID()
		} else if param.ProjectID() != projectID {
			return nil, rerror.ErrNotFound
		}
		deleteIndexes[param.Index()] = true
	}

	// Remove all specified parameters
	if err := i.paramRepo.RemoveAll(ctx, pids); err != nil {
		return nil, err
	}

	// Fetch remaining parameters for the project to recalculate indexes
	remainingParams, err := i.paramRepo.FindByProject(ctx, projectID)
	if err != nil {
		return nil, err
	}

	// Recalculate indexes for remaining parameters
	if remainingParams != nil && len(*remainingParams) > 0 {
		// Sort remaining parameters by current index
		sortedParams := make([]*parameter.Parameter, len(*remainingParams))
		copy(sortedParams, *remainingParams)

		// Sort remaining parameters by current index using sort.Slice
		sort.Slice(sortedParams, func(i, j int) bool {
			return sortedParams[i].Index() < sortedParams[j].Index()
		})

		// Reassign sequential indexes starting from 0
		for newIndex, param := range sortedParams {
			if param.Index() != newIndex {
				param.SetIndex(newIndex)
				if err := i.paramRepo.Save(ctx, param); err != nil {
					return nil, err
				}
			}
		}
	}

	tx.Commit()
	return pids, nil
}

func (i *Parameter) UpdateParameterOrder(ctx context.Context, param interfaces.UpdateParameterOrderParam) (*parameter.ParameterList, error) {
	proj, err := i.projectRepo.FindByID(ctx, param.ProjectID)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, proj.Workspace()); err != nil {
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
	target, err := i.paramRepo.FindByID(ctx, param.ParamID)
	if err != nil {
		return nil, err
	}
	if target == nil {
		return nil, rerror.ErrNotFound
	}
	proj, err := i.projectRepo.FindByID(ctx, target.ProjectID())
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, proj.Workspace()); err != nil {
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
	p.SetConfig(param.Config)

	if err := i.paramRepo.Save(ctx, p); err != nil {
		return nil, err
	}

	tx.Commit()
	return p, nil
}
