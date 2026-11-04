package interactor

import (
	"context"
	"errors"
	"fmt"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type ProjectAccess struct {
	projectRepo       repo.Project
	projectAccessRepo repo.ProjectAccess
	parameterRepo     repo.Parameter
	transaction       usecasex.Transactor
	permissionChecker gateway.PermissionChecker
	config            ContainerConfig
}

func NewProjectAccess(r *repo.Container, gr *gateway.Container, config ContainerConfig, permissionChecker gateway.PermissionChecker) interfaces.ProjectAccess {
	return &ProjectAccess{
		projectRepo:       r.Project,
		projectAccessRepo: r.ProjectAccess,
		parameterRepo:     r.Parameter,
		transaction:       r.Transaction,
		config:            config,
		permissionChecker: permissionChecker,
	}
}

func (i *ProjectAccess) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceProjectAccess, action, workspaceID...)
}

func (i *ProjectAccess) Fetch(ctx context.Context, token string) (project *project.Project, err error) {
	pa, err := i.projectAccessRepo.FindByToken(ctx, token)
	if err != nil {
		return nil, err
	}
	if pa == nil {
		return nil, errors.New("invalid sharing token")
	}

	if !pa.IsPublic() {
		return nil, errors.New("project access is not public")
	}

	prj, err := i.projectRepo.FindByID(ctx, pa.Project())
	if err != nil {
		return nil, err
	}
	if prj == nil {
		return nil, rerror.ErrNotFound
	}

	return prj, nil
}

// FetchParameters is the shared-link counterpart of Parameter.FetchByProject: the
// viewer holds a sharing token, not a workspace role, so the token is the only
// authorization. Non-public parameters keep their definition (so an import still
// gets the variable) but never their value.
func (i *ProjectAccess) FetchParameters(ctx context.Context, token string) (*parameter.ParameterList, error) {
	prj, err := i.Fetch(ctx, token)
	if err != nil {
		return nil, err
	}

	params, err := i.parameterRepo.FindByProject(ctx, prj.ID())
	if err != nil {
		return nil, err
	}
	// Non-nil even when empty: the field is a non-null list.
	res := make([]*parameter.Parameter, 0, params.Len())
	if params == nil {
		return parameter.NewParameterList(res), nil
	}
	for _, p := range *params {
		if p == nil {
			continue
		}
		if !p.Public() {
			p = p.WithoutValue()
		}
		res = append(res, p)
	}
	return parameter.NewParameterList(res), nil
}

func (i *ProjectAccess) Share(ctx context.Context, projectID id.ProjectID) (sharingUrl string, err error) {
	target, err := i.projectRepo.FindByID(ctx, projectID)
	if err != nil {
		return "", err
	}
	if target == nil {
		return "", rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, target.Workspace()); err != nil {
		return "", err
	}

	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		prj, err := i.projectRepo.FindByID(ctx, projectID)
		if err != nil {
			return err
		}

		pa, err := i.projectAccessRepo.FindByProjectID(ctx, projectID)
		if err != nil && !errors.Is(err, rerror.ErrNotFound) {
			return err
		}

		if pa == nil {
			pa, err = projectAccess.New().
				NewID().
				Project(prj.ID()).
				Build()
			if err != nil {
				return err
			}
		}

		if err := pa.MakePublic(); err != nil {
			return err
		}

		if err := i.projectAccessRepo.Save(ctx, pa); err != nil {
			return err
		}

		sharingToken := pa.Token()
		prj.SetSharedToken(&sharingToken)
		stampUpdatedBy(ctx, prj)
		if err := i.projectRepo.Save(ctx, prj); err != nil {
			return fmt.Errorf("failed to update project with sharing URL: %w", err)
		}

		sharingUrl, err = pa.SharingURL(i.config.Host, i.config.SharedPath)
		return err
	}); err != nil {
		return "", err
	}
	return sharingUrl, nil
}

func (i *ProjectAccess) Unshare(ctx context.Context, projectID id.ProjectID) (err error) {
	target, err := i.projectRepo.FindByID(ctx, projectID)
	if err != nil {
		return err
	}
	if target == nil {
		return rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, target.Workspace()); err != nil {
		return err
	}

	return i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		prj, err := i.projectRepo.FindByID(ctx, projectID)
		if err != nil {
			return err
		}

		pa, err := i.projectAccessRepo.FindByProjectID(ctx, projectID)
		if err != nil {
			return fmt.Errorf("failed to find project access: %w", err)
		}
		if pa == nil {
			return errors.New("project access not found")
		}

		if err := pa.MakePrivate(); err != nil {
			return err
		}

		if err := i.projectAccessRepo.Save(ctx, pa); err != nil {
			return err
		}

		prj.SetSharedToken(nil)
		stampUpdatedBy(ctx, prj)
		if err := i.projectRepo.Save(ctx, prj); err != nil {
			return fmt.Errorf("failed to update project to remove sharing URL: %w", err)
		}

		return nil
	})
}
