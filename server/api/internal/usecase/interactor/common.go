package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/reearth/reearthx/account/accountusecase/accountgateway"
	"github.com/reearth/reearthx/account/accountusecase/accountinteractor"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
)

var ErrPermissionDenied = fmt.Errorf("permission denied")

type ContainerConfig struct {
	SignupSecret    string
	AuthSrvUIDomain string
	Host            string
	SharedPath      string
}

func NewContainer(r *repo.Container, g *gateway.Container,
	ar *accountrepo.Container, ag *accountgateway.Container,
	permissionChecker gateway.PermissionChecker,
	config ContainerConfig,
) interfaces.Container {
	job := NewJob(r, g, permissionChecker)

	return interfaces.Container{
		Asset:         NewAsset(r, g, permissionChecker),
		Job:           job,
		Deployment:    NewDeployment(r, g, job, permissionChecker),
		Log:           NewLogInteractor(g.LogRedis, permissionChecker),
		Parameter:     NewParameter(r, permissionChecker),
		Project:       NewProject(r, g, job, permissionChecker),
		ProjectAccess: NewProjectAccess(r, g, config, permissionChecker),
		Workspace:     accountinteractor.NewWorkspace(ar, workspaceMemberCountEnforcer(r)),
		Trigger:       NewTrigger(r, g, job, permissionChecker),
		User:          accountinteractor.NewMultiUser(ar, ag, config.SignupSecret, config.AuthSrvUIDomain, ar.Users),
	}
}

type ProjectDeleter struct {
	File    gateway.File
	Project repo.Project
}

func (d ProjectDeleter) Delete(ctx context.Context, prj *project.Project, force bool) error {
	if prj == nil {
		return nil
	}

	// Delete project
	if err := d.Project.Remove(ctx, prj.ID()); err != nil {
		return err
	}

	return nil
}

func workspaceMemberCountEnforcer(_ *repo.Container) accountinteractor.WorkspaceMemberCountEnforcer {
	return func(ctx context.Context, ws *workspace.Workspace, _ user.List, op *accountusecase.Operator) error {
		return nil
	}
}

func checkPermission(ctx context.Context, permissionChecker gateway.PermissionChecker, resource string, action string) error {
	authInfo := adapter.GetAuthInfo(ctx)
	if authInfo == nil {
		return fmt.Errorf("auth info not found")
	}

	user := adapter.User(ctx)
	if user == nil {
		return fmt.Errorf("user not found")
	}

	hasPermission, err := permissionChecker.CheckPermission(ctx, authInfo, user.ID().String(), resource, action)
	if err != nil {
		return fmt.Errorf("failed to check permission: %w", err)
	}
	if !hasPermission {
		return ErrPermissionDenied
	}
	return nil
}
