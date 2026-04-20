package interactor

import (
	"context"
	"log"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/websocket"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/appx"
)

var skipPermissionCheck bool

type ContainerConfig struct {
	SignupSecret             string
	AuthSrvUIDomain          string
	Host                     string
	SharedPath               string
	WebsocketThriftServerURL string
	WebsocketAPISecret       string
	SkipPermissionCheck      bool
}

func NewContainer(r *repo.Container, g *gateway.Container,
	permissionChecker gateway.PermissionChecker,
	GQLClient *gqlclient.Client,
	job interfaces.Job,
	config ContainerConfig,
) interfaces.Container {
	setSkipPermissionCheck(config.SkipPermissionCheck)

	clientConfig := websocket.Config{
		ServerURL: config.WebsocketThriftServerURL,
		APISecret: config.WebsocketAPISecret,
	}
	client, err := websocket.NewClient(clientConfig)
	if err != nil {
		log.Fatalf("Failed to init websocket: %+v\n", err)
	}

	return interfaces.Container{
		Asset:         NewAsset(r, g, permissionChecker, GQLClient.WorkspaceRepo),
		CMS:           NewCMS(r, g, permissionChecker),
		Job:           job,
		Deployment:    NewDeployment(r, g, job, permissionChecker),
		EdgeExecution: NewEdgeExecution(r, g, permissionChecker),
		Log:           NewLogInteractor(g.Redis, r.Job, permissionChecker),
		NodeExecution: NewNodeExecution(r.NodeExecution, g.Redis, permissionChecker),
		Parameter:     NewParameter(r, permissionChecker),
		Project:       NewProject(r, g, job, permissionChecker, GQLClient.WorkspaceRepo, client),
		ProjectAccess: NewProjectAccess(r, g, config, permissionChecker),
		Workspace:     NewWorkspace(GQLClient.WorkspaceRepo),
		Trigger:       NewTrigger(r, g, job, permissionChecker),
		User:          NewUser(GQLClient.UserRepo),
		UserFacingLog: NewUserFacingLogInteractor(g.Redis, r.Job, permissionChecker),
		Websocket:     client,
		WorkerConfig:  NewWorkerConfig(r, permissionChecker),
	}
}

type ProjectDeleter struct {
	File      gateway.File
	Project   repo.Project
	Websocket interfaces.WebsocketClient
}

func (d ProjectDeleter) Delete(ctx context.Context, prj *project.Project, force bool) error {
	if prj == nil {
		return nil
	}

	// Delete collaborative document data (GCS snapshots + Redis stream)
	if d.Websocket != nil {
		if err := d.Websocket.DeleteDocument(ctx, prj.ID().String()); err != nil {
			log.Printf("WARNING: Failed to delete websocket document for project %s: %v", prj.ID(), err)
			// Non-fatal: project deletion should proceed even if WS cleanup fails
		}
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
		if token := adapter.JWT(ctx); token != "" {
			authInfo = &appx.AuthInfo{Token: token}
		} else if tmp := adapter.TempAuthInfo(ctx); tmp != nil {
			authInfo = tmp
		}
	}

	var userIDStr string
	if u := adapter.User(ctx); u != nil {
		userIDStr = u.ID().String()
	} else if u := adapter.ReearthxUser(ctx); u != nil {
		userIDStr = u.ID().String()
	} else {
		log.Printf("WARNING: User not found for resource=%s action=%s", resource, action)
		return interfaces.ErrOperationDenied
	}

	hasPermission, err := permissionChecker.CheckPermission(ctx, authInfo, userIDStr, resource, action)
	if err != nil {
		log.Printf("WARNING: Permission check error for user=%s resource=%s action=%s: %v", userIDStr, resource, action, err)
		return err
	}

	if !hasPermission {
		log.Printf("WARNING: Permission denied for user=%s resource=%s action=%s", userIDStr, resource, action)
		return interfaces.ErrOperationDenied
	}

	log.Printf("DEBUG: Permission granted for user=%s resource=%s action=%s", userIDStr, resource, action)

	return nil
}
