package interactor

import (
	"context"
	"errors"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type ProjectAccess struct {
	projectRepo       repo.Project
	projectAccessRepo repo.ProjectAccess
	transaction       usecasex.Transaction
	permissionChecker gateway.PermissionChecker
	config            ContainerConfig
}

func NewProjectAccess(r *repo.Container, gr *gateway.Container, config ContainerConfig, permissionChecker gateway.PermissionChecker) interfaces.ProjectAccess {
	return &ProjectAccess{
		projectRepo:       r.Project,
		projectAccessRepo: r.ProjectAccess,
		transaction:       r.Transaction,
		config:            config,
		permissionChecker: permissionChecker,
	}
}

func (i *ProjectAccess) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceProjectAccess, action)
}

func (i *ProjectAccess) Fetch(ctx context.Context, token string) (project *project.Project, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

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

	return i.projectRepo.FindByID(ctx, pa.Project())
}

func (i *ProjectAccess) Share(ctx context.Context, projectID id.ProjectID) (sharingUrl string, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return "", err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return "", err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	prj, err := i.projectRepo.FindByID(ctx, projectID)
	if err != nil {
		return "", err
	}

	var pa *projectAccess.ProjectAccess
	pa, err = i.projectAccessRepo.FindByProjectID(ctx, projectID)
	if err != nil && !errors.Is(err, rerror.ErrNotFound) {
		return "", err
	}

	if pa == nil {
		pa, err = projectAccess.New().
			NewID().
			Project(prj.ID()).
			Build()
		if err != nil {
			return "", err
		}
	}

	err = pa.MakePublic()
	if err != nil {
		return "", err
	}

	err = i.projectAccessRepo.Save(ctx, pa)
	if err != nil {
		return "", err
	}

	sharingToken := pa.Token()
	if err != nil {
		return "", err
	}

	prj.SetSharedToken(&sharingToken)
	err = i.projectRepo.Save(ctx, prj)
	if err != nil {
		return "", fmt.Errorf("failed to update project with sharing URL: %w", err)
	}

	sharingUrl, err = pa.SharingURL(i.config.Host, i.config.SharedPath)
	if err != nil {
		return "", err
	}

	tx.Commit()
	return sharingUrl, nil
}

func (i *ProjectAccess) Unshare(ctx context.Context, projectID id.ProjectID) (err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

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

	err = pa.MakePrivate()
	if err != nil {
		return err
	}

	err = i.projectAccessRepo.Save(ctx, pa)
	if err != nil {
		return err
	}

	prj.SetSharedToken(nil)
	err = i.projectRepo.Save(ctx, prj)
	if err != nil {
		return fmt.Errorf("failed to update project to remove sharing URL: %w", err)
	}

	tx.Commit()
	return nil
}
