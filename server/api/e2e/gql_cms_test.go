package e2e

import (
	"context"
	"encoding/json"
	"net"
	"net/http"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/cms"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountusecase/accountgateway"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/mailer"
	"github.com/samber/lo"
	"github.com/spf13/afero"
	"github.com/stretchr/testify/assert"
)

func StartGQLServerWithCMSGateway(t *testing.T, cfg *config.Config, repos *repo.Container, accountrepos *accountrepo.Container, cmsGateway gateway.CMS, allowPermission bool) *httpexpect.Expect {
	t.Helper()

	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}

	ctx := context.Background()

	l, err := net.Listen("tcp", ":0")
	if err != nil {
		t.Fatalf("server failed to listen: %v", err)
	}

	mockPermissionChecker := gateway.NewMockPermissionChecker()
	mockPermissionChecker.Allow = allowPermission

	cfg.SkipPermissionCheck = true
	srv := app.NewServer(ctx, &app.ServerConfig{
		Config:       cfg,
		Repos:        repos,
		AccountRepos: accountrepos,
		Gateways: &gateway.Container{
			File: lo.Must(fs.NewFile(afero.NewMemMapFs(), "https://example.com", "https://example2.com")),
			CMS:  cmsGateway,
		},
		AccountGateways: &accountgateway.Container{
			Mailer: mailer.New(ctx, &mailer.Config{}),
		},
		Debug:             true,
		PermissionChecker: mockPermissionChecker,
	})

	ch := make(chan error)
	go func() {
		if err := srv.Serve(l); err != http.ErrServerClosed {
			ch <- err
		}
		close(ch)
	}()
	t.Cleanup(func() {
		if err := srv.Shutdown(context.Background()); err != nil {
			t.Fatalf("server shutdown: %v", err)
		}

		if err := <-ch; err != nil {
			t.Fatalf("server serve: %v", err)
		}
	})
	return httpexpect.Default(t, "http://"+l.Addr().String())
}

func TestCMSWorkflows(t *testing.T) {
	mockCMS, err := NewMockCMSServer()
	if err != nil {
		t.Fatalf("failed to start mock CMS server: %v", err)
	}
	defer mockCMS.Close()

	cmsGateway, err := cms.NewGRPCClient(mockCMS.GetServiceURL(), "", "")
	if err != nil {
		t.Fatalf("failed to create CMS gateway: %v", err)
	}

	repos := initRepos(t, false, baseSeederUser)
	accountRepos := repos.AccountRepos()

	e := StartGQLServerWithCMSGateway(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, repos, accountRepos, cmsGateway, true)

	testGetCMSProject(t, e)

	testListCMSProjects(t, e)

	projectID := testCreateCMSProject(t, e)

	testUpdateCMSProject(t, e, projectID)

	testCheckCMSAlias(t, e)

	testDeleteCMSProject(t, e, projectID)

	testListCMSModels(t, e)

	testListCMSItems(t, e)

	testGetCMSModelExportURL(t, e)
}

func testGetCMSProject(t *testing.T, e *httpexpect.Expect) {
	query := `query($projectId: ID!) {
		cmsProject(projectIdOrAlias: $projectId) {
			id
			name
			alias
			description
			workspaceId
			visibility
			createdAt
			updatedAt
		}
	}`

	variables := map[string]interface{}{
		"projectId": "project-123",
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CMSProject struct {
				ID          string `json:"id"`
				Name        string `json:"name"`
				Alias       string `json:"alias"`
				Description string `json:"description"`
				WorkspaceID string `json:"workspaceId"`
				Visibility  string `json:"visibility"`
			} `json:"cmsProject"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	project := result.Data.CMSProject
	assert.Equal(t, "project-123", project.ID)
	assert.Equal(t, "Test Project 1", project.Name)
	assert.Equal(t, "test-project-1", project.Alias)
	assert.Equal(t, "Test project description", project.Description)
	assert.Equal(t, "workspace-123", project.WorkspaceID)
	assert.Equal(t, "PUBLIC", project.Visibility)
}

func testListCMSProjects(t *testing.T, e *httpexpect.Expect) {
	query := `query($workspaceId: ID!, $publicOnly: Boolean) {
		cmsProjects(workspaceId: $workspaceId, publicOnly: $publicOnly) {
			id
			name
			alias
			workspaceId
			visibility
		}
	}`

	variables := map[string]interface{}{
		"workspaceId": "workspace-123",
		"publicOnly":  false,
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CMSProjects []struct {
				ID          string `json:"id"`
				Name        string `json:"name"`
				Alias       string `json:"alias"`
				WorkspaceID string `json:"workspaceId"`
				Visibility  string `json:"visibility"`
			} `json:"cmsProjects"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	projects := result.Data.CMSProjects
	assert.Len(t, projects, 1)
	assert.Equal(t, "project-123", projects[0].ID)
	assert.Equal(t, "Test Project 1", projects[0].Name)
}

func testCreateCMSProject(t *testing.T, e *httpexpect.Expect) string {
	mutation := `mutation($input: CreateCMSProjectInput!) {
		createCMSProject(input: $input) {
			project {
				id
				name
				alias
				description
				workspaceId
				visibility
			}
		}
	}`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"workspaceId": wId1.String(),
			"name":        "New CMS Project",
			"alias":       "new-cms-project",
			"description": "A new CMS project for testing",
			"visibility":  "PUBLIC",
		},
	}

	request := GraphQLRequest{
		Query:     mutation,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CreateCMSProject struct {
				Project struct {
					ID          string `json:"id"`
					Name        string `json:"name"`
					Alias       string `json:"alias"`
					Description string `json:"description"`
					WorkspaceID string `json:"workspaceId"`
					Visibility  string `json:"visibility"`
				} `json:"project"`
			} `json:"createCMSProject"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	project := result.Data.CreateCMSProject.Project
	assert.NotEmpty(t, project.ID)
	assert.Equal(t, "New CMS Project", project.Name)
	assert.Equal(t, "new-cms-project", project.Alias)
	assert.Equal(t, "A new CMS project for testing", project.Description)
	assert.Equal(t, wId1.String(), project.WorkspaceID)
	assert.Equal(t, "PUBLIC", project.Visibility)

	return project.ID
}

func testUpdateCMSProject(t *testing.T, e *httpexpect.Expect, projectID string) {
	mutation := `mutation($input: UpdateCMSProjectInput!) {
		updateCMSProject(input: $input) {
			project {
				id
				name
				alias
				description
				visibility
			}
		}
	}`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"projectId":   projectID,
			"name":        "Updated CMS Project",
			"alias":       "updated-cms-project",
			"description": "Updated description",
			"visibility":  "PRIVATE",
		},
	}

	request := GraphQLRequest{
		Query:     mutation,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			UpdateCMSProject struct {
				Project struct {
					ID          string `json:"id"`
					Name        string `json:"name"`
					Alias       string `json:"alias"`
					Description string `json:"description"`
					Visibility  string `json:"visibility"`
				} `json:"project"`
			} `json:"updateCMSProject"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	project := result.Data.UpdateCMSProject.Project
	assert.Equal(t, projectID, project.ID)
	assert.Equal(t, "Updated CMS Project", project.Name)
	assert.Equal(t, "updated-cms-project", project.Alias)
	assert.Equal(t, "Updated description", project.Description)
	assert.Equal(t, "PRIVATE", project.Visibility)
}

func testCheckCMSAlias(t *testing.T, e *httpexpect.Expect) {
	mutation := `mutation($input: CheckCMSAliasAvailabilityInput!) {
		checkCMSAliasAvailability(input: $input) {
			available
		}
	}`

	// Test with available alias
	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"alias": "available-alias",
		},
	}

	request := GraphQLRequest{
		Query:     mutation,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CheckCMSAliasAvailability struct {
				Available bool `json:"available"`
			} `json:"checkCMSAliasAvailability"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	assert.True(t, result.Data.CheckCMSAliasAvailability.Available)
}

func testDeleteCMSProject(t *testing.T, e *httpexpect.Expect, projectID string) {
	mutation := `mutation($input: DeleteCMSProjectInput!) {
		deleteCMSProject(input: $input) {
			projectId
		}
	}`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"projectId": projectID,
		},
	}

	request := GraphQLRequest{
		Query:     mutation,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			DeleteCMSProject struct {
				ProjectID string `json:"projectId"`
			} `json:"deleteCMSProject"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	assert.Equal(t, projectID, result.Data.DeleteCMSProject.ProjectID)
}

func testListCMSModels(t *testing.T, e *httpexpect.Expect) {
	query := `query($projectId: ID!) {
		cmsModels(projectId: $projectId) {
			id
			projectId
			name
			key
		}
	}`

	variables := map[string]interface{}{
		"projectId": "project-123",
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CMSModels []struct {
				ID        string `json:"id"`
				ProjectID string `json:"projectId"`
				Name      string `json:"name"`
				Key       string `json:"key"`
			} `json:"cmsModels"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	models := result.Data.CMSModels
	assert.Len(t, models, 1)
	assert.Equal(t, "model-123", models[0].ID)
	assert.Equal(t, "project-123", models[0].ProjectID)
	assert.Equal(t, "Test Model", models[0].Name)
	assert.Equal(t, "test_model", models[0].Key)
}

func testListCMSItems(t *testing.T, e *httpexpect.Expect) {
	query := `query($projectId: ID!, $modelId: ID!) {
		cmsItems(projectId: $projectId, modelId: $modelId) {
			items {
				id
			}
			totalCount
		}
	}`

	variables := map[string]interface{}{
		"projectId": "project-123",
		"modelId":   "model-123",
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CMSItems struct {
				Items []struct {
					ID string `json:"id"`
				} `json:"items"`
				TotalCount int `json:"totalCount"`
			} `json:"cmsItems"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	assert.Len(t, result.Data.CMSItems.Items, 1)
	assert.Equal(t, 1, result.Data.CMSItems.TotalCount)
	assert.Equal(t, "item-123", result.Data.CMSItems.Items[0].ID)
}

func testGetCMSModelExportURL(t *testing.T, e *httpexpect.Expect) {
	query := `query($projectId: ID!, $modelId: ID!) {
		cmsModelExportUrl(projectId: $projectId, modelId: $modelId)
	}`

	variables := map[string]interface{}{
		"projectId": "project-123",
		"modelId":   "model-123",
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CMSModelExportURL string `json:"cmsModelExportUrl"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	expectedURL := "https://mock-cms.example.com/export/project-123/model-123.geojson"
	assert.Equal(t, expectedURL, result.Data.CMSModelExportURL)
}

func TestCMSErrorHandling(t *testing.T) {
	mockCMS, err := NewMockCMSServer()
	if err != nil {
		t.Fatalf("failed to start mock CMS server: %v", err)
	}
	defer mockCMS.Close()

	cmsGateway, err := cms.NewGRPCClient(mockCMS.GetServiceURL(), "", "")
	if err != nil {
		t.Fatalf("failed to create CMS gateway: %v", err)
	}

	repos := initRepos(t, false, baseSeederUser)
	accountRepos := repos.AccountRepos()

	e := StartGQLServerWithCMSGateway(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, repos, accountRepos, cmsGateway, true)

	testGetNonExistentCMSProject(t, e)

	testCreateCMSProjectWithDuplicateAlias(t, e)
}

func testGetNonExistentCMSProject(t *testing.T, e *httpexpect.Expect) {
	query := `query($projectId: ID!) {
		cmsProject(projectIdOrAlias: $projectId) {
			id
			name
		}
	}`

	variables := map[string]interface{}{
		"projectId": "non-existent-project",
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Errors []struct {
			Message string `json:"message"`
		} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)
	assert.NotEmpty(t, result.Errors)
}

func testCreateCMSProjectWithDuplicateAlias(t *testing.T, e *httpexpect.Expect) {
	mutation := `mutation($input: CreateCMSProjectInput!) {
		createCMSProject(input: $input) {
			project {
				id
				name
			}
		}
	}`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"workspaceId": wId1.String(),
			"name":        "Duplicate Project",
			"alias":       "test-project-1",
			"visibility":  "PUBLIC",
		},
	}

	request := GraphQLRequest{
		Query:     mutation,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Errors []struct {
			Message string `json:"message"`
		} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)
	assert.NotEmpty(t, result.Errors)
}
