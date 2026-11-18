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

// These tests boot the real server to pin that Container.Websocket is the
// authorizing interactor rather than the raw client, which unit tests cannot see.

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

// assertDenied requires the specific authorization error, because a raw-client
// wiring fails with a 404 from the absent websocket server and would pass a
// generic error check.
func assertDenied(t *testing.T, e *httpexpect.Expect, name, query string, vars map[string]any) {
	t.Helper()
	res := e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithJSON(map[string]any{"query": query, "variables": vars}).
		Expect().Status(http.StatusOK).JSON().Object()

	res.ContainsKey("errors")
	msg := res.Value("errors").Array().Value(0).Object().Value("message").String()
	msg.Contains("operation denied")
	msg.NotContains("server returned")
}

func TestDocumentOperations_DeniedWithoutPermission(t *testing.T) {
	e, projectRepo := startDocumentPermissionServer(t, false)

	// Two projects in DIFFERENT workspaces: copyProject addresses both, and one
	// project for each end would let a destination-only check look correct.
	prj := project.New().NewID().Workspace(project.NewWorkspaceID()).MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), prj))
	other := project.New().NewID().Workspace(project.NewWorkspaceID()).MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), other))
	p, o := prj.ID().String(), other.ID().String()

	assertDenied(t, e, "latestProjectSnapshot",
		`query($projectId: ID!) { latestProjectSnapshot(projectId: $projectId) { version } }`,
		map[string]any{"projectId": p})
	assertDenied(t, e, "projectHistory",
		`query($projectId: ID!) { projectHistory(projectId: $projectId) { version } }`,
		map[string]any{"projectId": p})
	assertDenied(t, e, "projectSnapshot",
		`query($projectId: ID!, $version: Int!) { projectSnapshot(projectId: $projectId, version: $version) { version } }`,
		map[string]any{"projectId": p, "version": 1})
	assertDenied(t, e, "projectSnapshot(snapshotNumber:)",
		`query($projectId: ID!, $snapshotNumber: Int!) { projectSnapshot(projectId: $projectId, snapshotNumber: $snapshotNumber) { snapshotNumber updates } }`,
		map[string]any{"projectId": p, "snapshotNumber": 1})
	assertDenied(t, e, "rollbackProject",
		`mutation($projectId: ID!, $version: Int!) { rollbackProject(projectId: $projectId, version: $version) { version } }`,
		map[string]any{"projectId": p, "version": 1})
	assertDenied(t, e, "saveSnapshot",
		`mutation($projectId: ID!) { saveSnapshot(projectId: $projectId) }`,
		map[string]any{"projectId": p})
	assertDenied(t, e, "previewSnapshot",
		`mutation($projectId: ID!, $version: Int!) { previewSnapshot(projectId: $projectId, version: $version) { version } }`,
		map[string]any{"projectId": p, "version": 1})
	// Bytes is a number array (Uint8Array), not base64: a string fails scalar
	// coercion before the resolver runs, so the case would assert nothing.
	assertDenied(t, e, "importProject",
		`mutation($projectId: ID!, $data: Bytes!) { importProject(projectId: $projectId, data: $data) }`,
		map[string]any{"projectId": p, "data": []int{1, 2, 3}})
	assertDenied(t, e, "copyProject",
		`mutation($projectId: ID!, $source: ID!) { copyProject(projectId: $projectId, source: $source) }`,
		map[string]any{"projectId": p, "source": o})
}

// Permissions are granted here, so an error can only come from project
// resolution failing closed.
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

			// "server returned" would mean the request reached the websocket client.
			res.ContainsKey("errors")
			msg := res.Value("errors").Array().Value(0).Object().Value("message").String()
			msg.NotContains("server returned")
		})
	}
}
