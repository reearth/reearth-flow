package e2e

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gavv/httpexpect/v2"
	usermockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user/mockrepo"
	workspacemockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace/mockrepo"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/policy"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"go.uber.org/mock/gomock"
)

// TestProjectVisibilityPolicy drives the real HTTP policy checker against a
// stand-in for the policy-check endpoint: the free workspace may not have
// private projects, the paid one may.
func TestProjectVisibilityPolicy(t *testing.T) {
	ctrl := gomock.NewController(t)
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(accountsuser.NewID())
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	freeWS := factory.NewWorkspace(func(b *accountsworkspace.Builder) { b.ID(accountsworkspace.NewID()) })
	paidWS := factory.NewWorkspace(func(b *accountsworkspace.Builder) { b.ID(accountsworkspace.NewID()) })

	policyService := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			WorkspaceID string `json:"workspace_id"`
			CheckType   string `json:"check_type"`
		}
		_ = json.NewDecoder(r.Body).Decode(&req)
		allowed := req.CheckType == "general_private_project_creation" && req.WorkspaceID == paidWS.ID().String()
		_ = json.NewEncoder(w).Encode(map[string]any{"allowed": allowed, "check_type": req.CheckType, "value": 1})
	}))
	defer policyService.Close()

	userRepo := usermockrepo.NewMockRepo(ctrl)
	userRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	workspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	workspaceRepo.EXPECT().FindByID(gomock.Any(), gomock.Any()).DoAndReturn(func(_ context.Context, id string) (*accountsworkspace.Workspace, error) {
		if id == paidWS.ID().String() {
			return paidWS, nil
		}
		return freeWS, nil
	}).AnyTimes()

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, &TestMocks{
		UserRepo:      userRepo,
		WorkspaceRepo: workspaceRepo,
		PolicyChecker: policy.NewHTTP(policyService.URL, "token", 5*time.Second),
	})
	do := func(query string, vars map[string]any) *httpexpect.Object {
		return e.POST("/api/graphql").
			WithHeader("authorization", "Bearer test").
			WithHeader("X-Reearth-Debug-User", operator.ID().String()).
			WithJSON(GraphQLRequest{Query: query, Variables: vars}).
			Expect().Status(http.StatusOK).JSON().Object()
	}
	create := func(ws *accountsworkspace.Workspace, visibility any) *httpexpect.Object {
		return do(`mutation($input: CreateProjectInput!) { createProject(input: $input) { project { id visibility } } }`,
			map[string]any{"input": map[string]any{"workspaceId": ws.ID().String(), "name": "p", "visibility": visibility}})
	}
	policyOf := func(ws *accountsworkspace.Workspace) *httpexpect.Value {
		return do(`query($w: ID!) { workspacePolicy(workspaceId: $w) { workspaceId canCreatePrivateProject } }`,
			map[string]any{"w": ws.ID().String()}).Path("$.data.workspacePolicy.canCreatePrivateProject")
	}

	t.Run("free workspace", func(t *testing.T) {
		policyOf(freeWS).IsEqual(false)

		create(freeWS, "PRIVATE").Path("$.errors[0].message").String().Contains("does not allow private projects")

		p := create(freeWS, nil).Path("$.data.createProject.project").Object()
		p.Value("visibility").IsEqual("PUBLIC")

		do(`mutation($input: UpdateProjectInput!) { updateProject(input: $input) { project { visibility } } }`,
			map[string]any{"input": map[string]any{"projectId": p.Value("id").String().Raw(), "visibility": "PRIVATE"}}).
			Path("$.errors[0].message").String().Contains("does not allow private projects")
	})

	t.Run("paid workspace", func(t *testing.T) {
		policyOf(paidWS).IsEqual(true)
		create(paidWS, "PRIVATE").Path("$.data.createProject.project.visibility").IsEqual("PRIVATE")
	})
}
