package interactor

import (
	"context"
	"errors"
	"log"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
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
		Asset:                NewAsset(r, g, permissionChecker, GQLClient.WorkspaceRepo),
		CMS:                  NewCMS(r, g, permissionChecker),
		Job:                  job,
		Deployment:           NewDeployment(r, g, job, permissionChecker),
		IntermediateDataView: NewIntermediateDataView(r, g, permissionChecker),
		Log:                  NewLogInteractor(g.Redis, r.Job, permissionChecker),
		NodeDiagnostics:      NewNodeDiagnostics(r.NodeDiagnostics, r.Job, g.Redis, g.File, permissionChecker),
		Parameter:            NewParameter(r, permissionChecker),
		Project:              NewProject(r, g, job, permissionChecker, GQLClient.WorkspaceRepo, client),
		ProjectAccess:        NewProjectAccess(r, g, config, permissionChecker),
		Workspace:            NewWorkspace(GQLClient.WorkspaceRepo, permissionChecker),
		Trigger:              NewTrigger(r, g, job, permissionChecker),
		User:                 NewUser(GQLClient.UserRepo),
		UserFacingLog:        NewUserFacingLogInteractor(g.Redis, r.Job, permissionChecker),
		Websocket:            NewWebsocket(client, r.Project, permissionChecker),
		WorkerConfig:         NewWorkerConfig(r, permissionChecker),
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

	// Fail closed on misuse rather than silently checking only workspaceID[0].
	if len(workspaceID) > 1 {
		log.Printf("ERROR: checkPermission called with %d workspace ids for resource=%s action=%s; expected at most one", len(workspaceID), resource, action)
		return interfaces.ErrOperationDenied
	}
	if skipPermissionCheck {
		log.Printf("INFO: SkipPermissionCheck enabled, skipping permission check for resource=%s action=%s", resource, action)
		return nil
	}

	memoUser := ""
	if u := adapter.User(ctx); u != nil {
		memoUser = u.ID().String()
	}
	memoWorkspace := ""
	if len(workspaceID) == 1 {
		memoWorkspace = workspaceID[0].String()
	}
	memo := adapter.PermissionVerdicts(ctx)
	if hasPermission, ok := memo.Get(memoUser, resource, action, memoWorkspace); ok {
		if !hasPermission {
			return interfaces.ErrOperationDenied
		}
		return nil
	}

	hasPermission, err := permissionChecker.CheckPermission(ctx, resource, action, workspaceID...)
	if err != nil {
		log.Printf("WARNING: Permission check error for resource=%s action=%s: %v", resource, action, err)
		span.RecordError(err)
		return err // never memoize an error result
	}

	memo.Set(memoUser, resource, action, memoWorkspace, hasPermission)

	if !hasPermission {
		log.Printf("WARNING: Permission denied for resource=%s action=%s", resource, action)
		return interfaces.ErrOperationDenied
	}

	log.Printf("DEBUG: Permission granted for resource=%s action=%s", resource, action)

	return nil
}

// alignToRequestedIDs reassembles a FindByIDs result into one slot per
// requested id, in request order, nil for any id not returned. Callers must
// not assume FindByIDs itself does this: Mongo's filterX helpers pad with nil,
// but pgxx.OrderByIDs (used by the Postgres repos) silently drops not-found
// ids and compacts the slice, and the in-memory fakes are inconsistent with
// each other too. Anything that treats the result as positionally aligned
// with the request - the GraphQL dataloaders consuming Fetch, and
// authorizeFetchByWorkspace's in-place nil-ing below - needs this first.
func alignToRequestedIDs[ID comparable, T comparable](ids []ID, got []T, zero T, idOf func(T) ID) []T {
	byID := make(map[ID]T, len(got))
	for _, it := range got {
		if it == zero {
			continue // some backends already pad with zero values; don't call idOf on one
		}
		byID[idOf(it)] = it
	}
	aligned := make([]T, len(ids))
	for i, id := range ids {
		aligned[i] = byID[id]
	}
	return aligned
}

// authorizeFetchByWorkspace authorizes an id-aligned batch fetch result (see
// alignToRequestedIDs; nil marks an absent id) per item's own workspace,
// zeroing out items whose workspace is denied instead of trusting one item's
// workspace for the whole batch. checkPermission memoizes per workspace, so a
// mixed-tenant batch costs one check per distinct workspace. An all-nil batch
// falls back to a single no-workspace check.
func authorizeFetchByWorkspace[T comparable](
	ctx context.Context,
	check func(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error,
	action string,
	items []T,
	zero T,
	workspaceOf func(T) accountsid.WorkspaceID,
) error {
	verdicts := map[accountsid.WorkspaceID]bool{}
	haveItem := false

	for idx, it := range items {
		if it == zero {
			continue
		}
		haveItem = true

		ws := workspaceOf(it)
		allowed, checked := verdicts[ws]
		if !checked {
			err := check(ctx, action, ws)
			if err != nil && !errors.Is(err, interfaces.ErrOperationDenied) {
				return err
			}
			allowed = err == nil
			verdicts[ws] = allowed
		}
		if !allowed {
			items[idx] = zero
		}
	}

	if !haveItem {
		return check(ctx, action)
	}
	return nil
}
