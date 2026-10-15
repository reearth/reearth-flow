package e2e

import (
	"net/http"
	"testing"

	"github.com/gavv/httpexpect/v2"
	usermockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user/mockrepo"
	workspacemockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace/mockrepo"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"go.uber.org/mock/gomock"
)

const projectMetadataFields = `
	id
	alias
	visibility
	readme
	license
	topics
	starCount
	starred
`

func TestProjectMetadata(t *testing.T) {
	ctrl := gomock.NewController(t)
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(accountsuser.NewID())
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wid := accountsworkspace.NewID()
	w := factory.NewWorkspace(func(b *accountsworkspace.Builder) { b.ID(wid) })

	userRepo := usermockrepo.NewMockRepo(ctrl)
	userRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	workspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	workspaceRepo.EXPECT().FindByID(gomock.Any(), gomock.Any()).Return(w, nil).AnyTimes()

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, &TestMocks{UserRepo: userRepo, WorkspaceRepo: workspaceRepo})
	do := func(query string, vars map[string]any) *httpexpect.Object {
		return e.POST("/api/graphql").
			WithHeader("authorization", "Bearer test").
			WithHeader("X-Reearth-Debug-User", operator.ID().String()).
			WithJSON(GraphQLRequest{Query: query, Variables: vars}).
			Expect().Status(http.StatusOK).JSON().Object()
	}
	createProject := func(input map[string]any) *httpexpect.Object {
		input["workspaceId"] = wid.String()
		return do(`mutation($input: CreateProjectInput!) { createProject(input: $input) { project {`+projectMetadataFields+`} } }`,
			map[string]any{"input": input})
	}
	updateProject := func(input map[string]any) *httpexpect.Object {
		return do(`mutation($input: UpdateProjectInput!) { updateProject(input: $input) { project {`+projectMetadataFields+`} } }`,
			map[string]any{"input": input})
	}
	checkAlias := func(alias string, projectID any) *httpexpect.Object {
		return do(`query($w: ID!, $a: String!, $p: ID) { checkProjectAlias(workspaceId: $w, alias: $a, projectId: $p) { alias available } }`,
			map[string]any{"w": wid.String(), "a": alias, "p": projectID})
	}
	setStar := func(projectID string, starred bool) *httpexpect.Object {
		return do(`mutation($input: SetProjectStarInput!) { setProjectStar(input: $input) { project {`+projectMetadataFields+`} } }`,
			map[string]any{"input": map[string]any{"projectId": projectID, "starred": starred}})
	}

	t.Run("defaults", func(t *testing.T) {
		p := createProject(map[string]any{"name": "defaults"}).Path("$.data.createProject.project").Object()
		id := p.Value("id").String().Raw()
		p.Value("alias").IsEqual("p-" + id)
		p.Value("visibility").IsEqual("PUBLIC")
		p.Value("readme").IsEqual("")
		p.Value("license").IsEqual("")
		p.Value("topics").Array().IsEmpty()
		p.Value("starCount").IsEqual(0)
		p.Value("starred").IsEqual(false)
	})

	var projectID string
	t.Run("create with metadata", func(t *testing.T) {
		p := createProject(map[string]any{
			"name":       "with metadata",
			"alias":      "e2e-project",
			"visibility": "PRIVATE",
			"readme":     "# Hello",
			"license":    "MIT",
			"topics":     []string{"gis", "plateau", "gis"},
		}).Path("$.data.createProject.project").Object()
		projectID = p.Value("id").String().Raw()
		p.Value("alias").IsEqual("e2e-project")
		p.Value("visibility").IsEqual("PRIVATE")
		p.Value("readme").IsEqual("# Hello")
		p.Value("license").IsEqual("MIT")
		p.Value("topics").IsEqual([]string{"gis", "plateau"})
	})

	t.Run("taken alias", func(t *testing.T) {
		res := createProject(map[string]any{"name": "dup", "alias": "e2e-project"})
		res.Path("$.errors[0].message").String().Contains("project alias is already used")
	})

	t.Run("invalid alias", func(t *testing.T) {
		res := createProject(map[string]any{"name": "bad", "alias": "no"})
		res.Path("$.errors[0].message").String().Contains("invalid alias")
	})

	t.Run("checkProjectAlias", func(t *testing.T) {
		checkAlias("e2e-project", nil).Path("$.data.checkProjectAlias.available").IsEqual(false)
		checkAlias("e2e-project", projectID).Path("$.data.checkProjectAlias.available").IsEqual(true)
		checkAlias("free-alias", nil).Path("$.data.checkProjectAlias").IsEqual(map[string]any{"alias": "free-alias", "available": true})
		checkAlias("p-reserved", nil).Path("$.errors[0].message").String().Contains("reserved")
	})

	t.Run("update", func(t *testing.T) {
		p := updateProject(map[string]any{
			"projectId":  projectID,
			"alias":      "e2e-renamed",
			"visibility": "PUBLIC",
			"topics":     []string{},
		}).Path("$.data.updateProject.project").Object()
		p.Value("alias").IsEqual("e2e-renamed")
		p.Value("visibility").IsEqual("PUBLIC")
		p.Value("topics").Array().IsEmpty()
		p.Value("readme").IsEqual("# Hello")
		checkAlias("e2e-project", nil).Path("$.data.checkProjectAlias.available").IsEqual(true)
	})

	t.Run("stars", func(t *testing.T) {
		p := setStar(projectID, true).Path("$.data.setProjectStar.project").Object()
		p.Value("starCount").IsEqual(1)
		p.Value("starred").IsEqual(true)

		setStar(projectID, true).Path("$.data.setProjectStar.project.starCount").IsEqual(1)

		listed := do(`query($w: ID!) { projects(workspaceId: $w, pagination: {page: 1, pageSize: 10}) { nodes { id starred starCount } } }`,
			map[string]any{"w": wid.String()}).Path("$.data.projects.nodes").Array()
		found := false
		for _, n := range listed.Iter() {
			o := n.Object()
			if o.Value("id").String().Raw() == projectID {
				o.Value("starred").IsEqual(true)
				o.Value("starCount").IsEqual(1)
				found = true
			}
		}
		if !found {
			t.Fatalf("project %s missing from the list", projectID)
		}

		p = setStar(projectID, false).Path("$.data.setProjectStar.project").Object()
		p.Value("starCount").IsEqual(0)
		p.Value("starred").IsEqual(false)
	})
}
