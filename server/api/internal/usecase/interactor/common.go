package interactor

import (
	"context"
	"log"

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

var skipPermissionCheck bool

type ContainerConfig struct {
	SignupSecret        string
	AuthSrvUIDomain     string
	Host                string
	SharedPath          string
	SkipPermissionCheck bool
}

func NewContainer(r *repo.Container, g *gateway.Container,
	ar *accountrepo.Container, ag *accountgateway.Container,
	permissionChecker gateway.PermissionChecker,
	config ContainerConfig,
) interfaces.Container {
	setSkipPermissionCheck(config.SkipPermissionCheck)

	job := NewJob(r, g, permissionChecker)

	return interfaces.Container{
		Asset:         NewAsset(r, g, permissionChecker),
		Job:           job,
		Deployment:    NewDeployment(r, g, job, permissionChecker),
		EdgeExecution: NewEdgeExecution(r, g, permissionChecker),
		Log:           NewLogInteractor(g.Redis, permissionChecker),
		NodeExecution: NewNodeExecution(r.NodeExecution, g.Redis, permissionChecker),
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

func setSkipPermissionCheck(isSkipPermissionCheck bool) {
	skipPermissionCheck = isSkipPermissionCheck
}

func checkPermission(ctx context.Context, permissionChecker gateway.PermissionChecker, resource string, action string) error {
	if skipPermissionCheck {
		log.Printf("INFO: SkipPermissionCheck enabled, skipping permission check for resource=%s action=%s", resource, action)
		return nil
	}

	authInfo := adapter.GetAuthInfo(ctx)
	if authInfo == nil {
		log.Printf("WARNING: AuthInfo not found for resource=%s action=%s", resource, action)
		return nil
	}

	user := adapter.User(ctx)
	if user == nil {
		log.Printf("WARNING: User not found for resource=%s action=%s", resource, action)
		return nil
	}

	// Once the operation check in the oss environment is completed, delete the log output and
	hasPermission, err := permissionChecker.CheckPermission(ctx, authInfo, user.ID().String(), resource, action)
	if err != nil {
		log.Printf("WARNING: Permission check error for user=%s resource=%s action=%s: %v", user.ID().String(), resource, action, err)
		return nil
	}

	if !hasPermission {
		log.Printf("WARNING: Permission denied for user=%s resource=%s action=%s", user.ID().String(), resource, action)
		return nil
	}

	log.Printf("DEBUG: Permission granted for user=%s resource=%s action=%s", user.ID().String(), resource, action)

	return nil
}
