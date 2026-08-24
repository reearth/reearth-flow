package interactor

import (
	"context"
	"log"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/websocket"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
)

// tracerName identifies spans emitted by this package in the OpenTelemetry backend.
const tracerName = "github.com/reearth/reearth-flow/api/internal/usecase/interactor"

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
		Asset:           NewAsset(r, g, permissionChecker, GQLClient.WorkspaceRepo),
		CMS:             NewCMS(r, g, permissionChecker),
		Job:             job,
		Deployment:      NewDeployment(r, g, job, permissionChecker),
		Log:             NewLogInteractor(g.Redis, r.Job, permissionChecker),
		NodeDiagnostics: NewNodeDiagnostics(r.NodeDiagnostics, r.Job, g.Redis, permissionChecker),
		NodeExecution:   NewNodeExecution(r.NodeExecution, r.Job, g.Redis, permissionChecker),
		Parameter:       NewParameter(r, permissionChecker),
		Project:         NewProject(r, g, job, permissionChecker, GQLClient.WorkspaceRepo, client),
		ProjectAccess:   NewProjectAccess(r, g, config, permissionChecker),
		Workspace:       NewWorkspace(GQLClient.WorkspaceRepo),
		Trigger:         NewTrigger(r, g, job, permissionChecker),
		User:            NewUser(GQLClient.UserRepo),
		UserFacingLog:   NewUserFacingLogInteractor(g.Redis, r.Job, permissionChecker),
		Websocket:       NewWebsocket(client, r.Project, permissionChecker),
		WorkerConfig:    NewWorkerConfig(r, permissionChecker),
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

	if err := d.Project.Remove(ctx, prj.ID()); err != nil {
		return err
	}

	return nil
}

func setSkipPermissionCheck(isSkipPermissionCheck bool) {
	skipPermissionCheck = isSkipPermissionCheck
}

func checkPermission(ctx context.Context, permissionChecker gateway.PermissionChecker, resource string, action string, workspaceID ...accountsid.WorkspaceID) error {
	ctx, span := otel.Tracer(tracerName).Start(ctx, "interactor.checkPermission")
	defer span.End()
	span.SetAttributes(
		attribute.String("permission.resource", resource),
		attribute.String("permission.action", action),
		attribute.Int("permission.workspace_count", len(workspaceID)),
	)

	// At most one workspace is meaningful; reject misuse and fail closed rather
	// than silently evaluating against workspaceID[0] and ignoring the rest.
	if len(workspaceID) > 1 {
		log.Printf("ERROR: checkPermission called with %d workspace ids for resource=%s action=%s; expected at most one", len(workspaceID), resource, action)
		return interfaces.ErrOperationDenied
	}
	if skipPermissionCheck {
		log.Printf("INFO: SkipPermissionCheck enabled, skipping permission check for resource=%s action=%s", resource, action)
		return nil
	}

	hasPermission, err := permissionChecker.CheckPermission(ctx, resource, action, workspaceID...)
	if err != nil {
		log.Printf("WARNING: Permission check error for resource=%s action=%s: %v", resource, action, err)
		span.RecordError(err)
		return err
	}

	if !hasPermission {
		log.Printf("WARNING: Permission denied for resource=%s action=%s", resource, action)
		return interfaces.ErrOperationDenied
	}

	log.Printf("DEBUG: Permission granted for resource=%s action=%s", resource, action)

	return nil
}
