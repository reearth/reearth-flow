package e2e

import (
	"encoding/json"
	"net/http"
	"testing"

	usermockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user/mockrepo"
	workspacemockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace/mockrepo"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.uber.org/mock/gomock"
)

func TestProjectAuthorship(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wid := accountsworkspace.NewID()
	w := factory.NewWorkspace(func(b *accountsworkspace.Builder) {
		b.ID(wid)
	})

	mockUserRepo := usermockrepo.NewMockRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), gomock.Any()).Return(w, nil).AnyTimes()

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	})

	post := func(query string, variables map[string]any) string {
		jsonData, err := json.Marshal(GraphQLRequest{Query: query, Variables: variables})
		require.NoError(t, err)
		return e.POST("/api/graphql").
			WithHeader("authorization", "Bearer test").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", operatorID.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK).Body().Raw()
	}

	var created struct {
		Data struct {
			CreateProject struct {
				Project struct {
					CreatedByID *string `json:"createdById"`
					UpdatedByID *string `json:"updatedById"`
					ID          string  `json:"id"`
				} `json:"project"`
			} `json:"createProject"`
		} `json:"data"`
	}
	require.NoError(t, json.Unmarshal([]byte(post(`mutation($input: CreateProjectInput!) {
		createProject(input: $input) { project { id createdById updatedById } }
	}`, map[string]any{"input": map[string]any{"workspaceId": wid.String(), "name": "mine"}})), &created))
	p := created.Data.CreateProject.Project
	require.NotNil(t, p.CreatedByID)
	assert.Equal(t, operatorID.String(), *p.CreatedByID)
	require.NotNil(t, p.UpdatedByID)
	assert.Equal(t, operatorID.String(), *p.UpdatedByID)

	listQuery := `query($workspaceId: ID!, $createdBy: [ID!]) {
		projects(workspaceId: $workspaceId, createdBy: $createdBy, pagination: { page: 1, pageSize: 10 }) {
			nodes { id createdById }
			totalCount
		}
	}`
	type listResult struct {
		Data struct {
			Projects *struct {
				Nodes []struct {
					ID string `json:"id"`
				} `json:"nodes"`
				TotalCount int `json:"totalCount"`
			} `json:"projects"`
		} `json:"data"`
		Errors []any `json:"errors"`
	}

	var mine listResult
	require.NoError(t, json.Unmarshal([]byte(post(listQuery, map[string]any{
		"workspaceId": wid.String(), "createdBy": []string{operatorID.String()},
	})), &mine))
	require.Empty(t, mine.Errors)
	require.NotNil(t, mine.Data.Projects)
	assert.Equal(t, 1, mine.Data.Projects.TotalCount)
	require.Len(t, mine.Data.Projects.Nodes, 1)
	assert.Equal(t, p.ID, mine.Data.Projects.Nodes[0].ID)

	var others listResult
	require.NoError(t, json.Unmarshal([]byte(post(listQuery, map[string]any{
		"workspaceId": wid.String(), "createdBy": []string{accountsuser.NewID().String()},
	})), &others))
	require.Empty(t, others.Errors)
	assert.Equal(t, 0, others.Data.Projects.TotalCount)

	var malformed listResult
	require.NoError(t, json.Unmarshal([]byte(post(listQuery, map[string]any{
		"workspaceId": wid.String(), "createdBy": []string{operatorID.String(), "not-an-id"},
	})), &malformed))
	assert.NotEmpty(t, malformed.Errors, "a malformed creator must not list every project")
}
