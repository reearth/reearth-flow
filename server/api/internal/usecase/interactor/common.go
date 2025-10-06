package interactor

import (
	"context"
	"log"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/websocket"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/project"
)

var skipPermissionCheck bool

type ContainerConfig struct {
	SignupSecret             string
	AuthSrvUIDomain          string
	Host                     string
	SharedPath               string
	WebsocketThriftServerURL string
	SkipPermissionCheck      bool
	WorkerConfig             interface{}
}

func NewContainer(r *repo.Container, g *gateway.Container,
	permissionChecker gateway.PermissionChecker,
	GQLClient *gql.Client,
	job interfaces.Job,
	config ContainerConfig,
) interfaces.Container {
	setSkipPermissionCheck(config.SkipPermissionCheck)

	clientConfig := websocket.Config{
		ServerURL: config.WebsocketThriftServerURL,
	}
	client, err := websocket.NewClient(clientConfig)
	if err != nil {
		log.Fatalf("Failed to init websocket: %+v\n", err)
	}

	return interfaces.Container{
		Asset:         NewAsset(r, g, permissionChecker),
		CMS:           NewCMS(r, g, permissionChecker),
		Job:           job,
		Deployment:    NewDeployment(r, g, job, permissionChecker),
		Edge:          NewEdge(r, permissionChecker),
		EdgeExecution: NewEdgeExecution(r, g, permissionChecker),
		Log:           NewLogInteractor(g.Redis, r.Job, permissionChecker),
		Node:          NewNode(r, permissionChecker),
		NodeExecution: NewNodeExecution(r.NodeExecution, g.Redis, permissionChecker),
		Parameter:     NewParameter(r, permissionChecker),
		Project:       NewProject(r, g, job, permissionChecker, GQLClient.WorkspaceRepo),
		ProjectAccess: NewProjectAccess(r, g, config, permissionChecker),
		Workspace:     NewWorkspace(GQLClient.WorkspaceRepo),
		Trigger:       NewTrigger(r, g, job, permissionChecker),
		User:          NewUser(GQLClient.UserRepo),
		UserFacingLog: NewUserFacingLogInteractor(g.Redis, r.Job, permissionChecker),
		WorkerConfig:  NewWorkerConfig(r, permissionChecker, config.WorkerConfig),
		Websocket:     client,
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

	user := adapter.ReearthxUser(ctx)
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
