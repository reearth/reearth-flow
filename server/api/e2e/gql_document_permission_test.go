package e2e

import (
	"context"
	"net/http"
	"testing"

	"github.com/gavv/httpexpect/v2"
	usermockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user/mockrepo"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/stretchr/testify/require"
	"go.uber.org/mock/gomock"
)

// The document/websocket operations used to be the one GraphQL surface with no
// authorization: interfaces.Container.Websocket held the bare HTTP client, so any
// authenticated user could act on any project in any workspace. rollbackProject
// was the worst of it, because it prunes a project's update log.
//
// These tests pin the WIRING, which the interactor unit tests cannot reach. Those
// prove the interactor denies; only booting the real server proves
// Container.Websocket actually points at the interactor and not the raw client.
// Re-pointing it would leave every unit test green.
//
// Memory-backed on purpose (useMongo = false) so this runs anywhere rather than
// only where a database is available.
//
// Note the pre-existing TestDocumentOperations is not comparable coverage: its
// interceptor answers the GraphQL request itself with canned JSON, so it never
// reaches the resolvers, the container, or the client.

// startDocumentPermissionServer boots the real server with permissions either
// enforced or granted, and returns the expect client plus the project repo.
func startDocumentPermissionServer(t *testing.T, allowPermission bool) (*httpexpect.Expect, repo.Project) {
	t.Helper()
	ctrl := gomock.NewController(t)

	workspace := factory.NewWorkspace()
	userEntity := factory.NewUser(func(b *accountsuser.Builder) {
		b.Workspace(workspace.ID())
		b.Auths([]accountsuser.Auth{{Provider: "auth0", Sub: "auth0|doc-perm-test"}})
	})

	mockUserRepo := usermockrepo.NewMockRepo(ctrl)
	// AnyTimes: the table below issues one request per operation.
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(userEntity, nil).AnyTimes()

	cfg := &config.Config{
		Origins:         []string{"https://example.com"},
		AuthSrv:         config.AuthSrvConfig{Disabled: true},
		AccountsApiHost: "http://localhost:8080",
	}
	exp, repos, _ := StartServerAndRepos(t, cfg, false, allowPermission, &TestMocks{UserRepo: mockUserRepo})
	return exp, repos.Project
}

// documentOps is every operation the schema exposes. Table-driven so a new
// operation added without authorization shows up as a missing row rather than
// passing unnoticed.
var documentOps = []struct {
	// vars first: keeping the pointer-bearing field at the front shortens the
	// range the GC has to scan (govet fieldalignment).
	vars  func(projectID string) map[string]any
	name  string
	query string
}{
	{
		name:  "latestProjectSnapshot",
		query: `query($projectId: ID!) { latestProjectSnapshot(projectId: $projectId) { version } }`,
		vars:  func(p string) map[string]any { return map[string]any{"projectId": p} },
	},
	{
		name:  "projectHistory",
		query: `query($projectId: ID!) { projectHistory(projectId: $projectId) { version } }`,
		vars:  func(p string) map[string]any { return map[string]any{"projectId": p} },
	},
	{
		name:  "projectSnapshot",
		query: `query($projectId: ID!, $version: Int!) { projectSnapshot(projectId: $projectId, version: $version) { version } }`,
		vars: func(p string) map[string]any {
			return map[string]any{"projectId": p, "version": 1}
		},
	},
	{
		name:  "rollbackProject",
		query: `mutation($projectId: ID!, $version: Int!) { rollbackProject(projectId: $projectId, version: $version) { version } }`,
		vars: func(p string) map[string]any {
			return map[string]any{"projectId": p, "version": 1}
		},
	},
	{
		name:  "saveSnapshot",
		query: `mutation($projectId: ID!) { saveSnapshot(projectId: $projectId) }`,
		vars:  func(p string) map[string]any { return map[string]any{"projectId": p} },
	},
	{
		name:  "previewSnapshot",
		query: `mutation($projectId: ID!, $version: Int!) { previewSnapshot(projectId: $projectId, version: $version) { version } }`,
		vars: func(p string) map[string]any {
			return map[string]any{"projectId": p, "version": 1}
		},
	},
	{
		name:  "importProject",
		query: `mutation($projectId: ID!, $data: Bytes!) { importProject(projectId: $projectId, data: $data) }`,
		vars: func(p string) map[string]any {
			// Bytes is a number array (Uint8Array), NOT base64: a string here fails
			// scalar coercion before the resolver runs, so the row would assert nothing.
			return map[string]any{"projectId": p, "data": []int{1, 2, 3}}
		},
	},
}

// TestDocumentOperations_DeniedWithoutPermission: with the permission checker
// refusing, every operation must return a GraphQL error.
//
// This is what pins the authorizing wrapper in place. If Container.Websocket were
// re-pointed at the raw client, the request would instead reach a websocket server
// that is not running, which is a different failure and a real bypass.
func TestDocumentOperations_DeniedWithoutPermission(t *testing.T) {
	e, projectRepo := startDocumentPermissionServer(t, false)

	prj := project.New().NewID().Workspace(project.NewWorkspaceID()).MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), prj))

	for _, op := range documentOps {
		t.Run(op.name, func(t *testing.T) {
			res := e.POST("/api/graphql").
				WithHeader("Origin", "https://example.com").
				WithHeader("authorization", "Bearer test").
				WithHeader("Content-Type", "application/json").
				WithJSON(map[string]any{
					"query":     op.query,
					"variables": op.vars(prj.ID().String()),
				}).
				Expect().
				Status(http.StatusOK).
				JSON().Object()

			// Assert the SPECIFIC error. "an error occurred" is not enough: with the
			// bare client wired in, the request reaches a websocket server that is not
			// running and fails with `server returned non-200 status: 404`, so a test
			// that only checks for the presence of errors passes under the very bypass
			// it is meant to catch. (Verified: it does.) Only the message distinguishes
			// authorization from a transport failure.
			res.ContainsKey("errors")
			msg := res.Value("errors").Array().Value(0).Object().Value("message").String()
			msg.Contains("operation denied")
			msg.NotContains("server returned")
		})
	}
}

// TestDocumentOperations_UnknownProjectIsDenied: authorization resolves the
// project to a workspace, so an id that does not resolve must fail closed rather
// than skip the check. Permissions are GRANTED here, so the only thing that can
// produce an error is the resolution step refusing.
func TestDocumentOperations_UnknownProjectIsDenied(t *testing.T) {
	e, _ := startDocumentPermissionServer(t, true)

	for _, projectID := range []string{"not-a-valid-id", project.NewID().String()} {
		t.Run(projectID, func(t *testing.T) {
			res := e.POST("/api/graphql").
				WithHeader("Origin", "https://example.com").
				WithHeader("authorization", "Bearer test").
				WithHeader("Content-Type", "application/json").
				WithJSON(map[string]any{
					"query":     `query($projectId: ID!) { projectHistory(projectId: $projectId) { version } }`,
					"variables": map[string]any{"projectId": projectID},
				}).
				Expect().
				Status(http.StatusOK).
				JSON().Object()

			// Same reasoning as above, via the negative: permissions are granted here,
			// so the request must be stopped by project resolution BEFORE the client is
			// reached. "server returned" in the message would mean it went through.
			res.ContainsKey("errors")
			msg := res.Value("errors").Array().Value(0).Object().Value("message").String()
			msg.NotContains("server returned")
		})
	}
}
