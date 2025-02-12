package e2e

import (
	"bytes"
	"encoding/json"
	"fmt"
	"mime/multipart"
	"net/http"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/stretchr/testify/assert"
)

func TestProjectsPagination(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser)

	// Create multiple projects for testing
	projectIDs := make([]string, 5)
	for i := 0; i < 5; i++ {
		query := `mutation($input: CreateProjectInput!) {
			createProject(input: $input) {
				project {
					id
				}
			}
		}`

		variables := fmt.Sprintf(`{
			"input": {
				"workspaceId": "%s",
				"name": "Test Project %d",
				"description": "Test project description %d"
			}
		}`, wId1.String(), i, i)

		var variablesMap map[string]any
		err := json.Unmarshal([]byte(variables), &variablesMap)
		assert.NoError(t, err)

		request := GraphQLRequest{
			Query:     query,
			Variables: variablesMap,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var result struct {
			Data struct {
				CreateProject struct {
					Project struct {
						ID string `json:"id"`
					} `json:"project"`
				} `json:"createProject"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)
		assert.NotEmpty(t, result.Data.CreateProject.Project.ID, "Project creation failed")
		projectIDs[i] = result.Data.CreateProject.Project.ID

		// Add a small delay between project creations
		time.Sleep(100 * time.Millisecond)
	}

	// Verify all projects were created
	assert.Len(t, projectIDs, 5, "Expected 5 projects to be created")
	for i, id := range projectIDs {
		assert.NotEmpty(t, id, fmt.Sprintf("Project %d was not created successfully", i))
	}

	// Test page-based pagination
	t.Run("test_page_based_pagination", func(t *testing.T) {
		query := fmt.Sprintf(`{
			projects(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 2
				}
			) {
				nodes {
					id
					name
				}
				pageInfo {
					totalCount
					totalPages
					currentPage
				}
			}
		}`, wId1.String())

		request := GraphQLRequest{
			Query: query,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var result struct {
			Data struct {
				ProjectsPage struct {
					Nodes []struct {
						ID   string `json:"id"`
						Name string `json:"name"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						TotalPages  int `json:"totalPages"`
						CurrentPage int `json:"currentPage"`
					} `json:"pageInfo"`
				} `json:"projects"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		assert.Len(t, result.Data.ProjectsPage.Nodes, 2)
		assert.Equal(t, 5, result.Data.ProjectsPage.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.ProjectsPage.PageInfo.TotalPages)
		assert.Equal(t, 1, result.Data.ProjectsPage.PageInfo.CurrentPage)
	})

	// Test sorting
	t.Run("test_sorting", func(t *testing.T) {
		// First test ASC order
		query := fmt.Sprintf(`{
			projects(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 5
					orderBy: "name"
					orderDir: ASC
				}
			) {
				nodes {
					id
					name
				}
			}
		}`, wId1.String())

		request := GraphQLRequest{
			Query: query,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var resultAsc struct {
			Data struct {
				ProjectsPage struct {
					Nodes []struct {
						ID   string `json:"id"`
						Name string `json:"name"`
					} `json:"nodes"`
				} `json:"projects"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &resultAsc)
		assert.NoError(t, err)

		// Verify ASC order
		for i := 1; i < len(resultAsc.Data.ProjectsPage.Nodes); i++ {
			prev := resultAsc.Data.ProjectsPage.Nodes[i-1].Name
			curr := resultAsc.Data.ProjectsPage.Nodes[i].Name
			assert.True(t, prev <= curr, "Projects should be sorted by name in ascending order")
		}

		// Now test DESC order
		query = fmt.Sprintf(`{
			projects(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 5
					orderBy: "name"
					orderDir: DESC
				}
			) {
				nodes {
					id
					name
				}
			}
		}`, wId1.String())

		request = GraphQLRequest{
			Query: query,
		}
		jsonData, err = json.Marshal(request)
		assert.NoError(t, err)

		resp = e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var resultDesc struct {
			Data struct {
				ProjectsPage struct {
					Nodes []struct {
						ID   string `json:"id"`
						Name string `json:"name"`
					} `json:"nodes"`
				} `json:"projects"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &resultDesc)
		assert.NoError(t, err)

		// Verify DESC order
		for i := 1; i < len(resultDesc.Data.ProjectsPage.Nodes); i++ {
			prev := resultDesc.Data.ProjectsPage.Nodes[i-1].Name
			curr := resultDesc.Data.ProjectsPage.Nodes[i].Name
			assert.True(t, prev >= curr, "Projects should be sorted by name in descending order")
		}

		// Verify that ASC and DESC orders are opposite of each other
		if len(resultAsc.Data.ProjectsPage.Nodes) > 0 && len(resultDesc.Data.ProjectsPage.Nodes) > 0 {
			assert.Equal(t,
				resultAsc.Data.ProjectsPage.Nodes[0].Name,
				resultDesc.Data.ProjectsPage.Nodes[len(resultDesc.Data.ProjectsPage.Nodes)-1].Name,
				"First element in ASC should equal last element in DESC",
			)
		}
	})
}

func TestJobsPagination(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser)

	deploymentQuery := `mutation($input: CreateDeploymentInput!) {
		createDeployment(input: $input) {
			deployment {
				id
			}
		}
	}`

	// Create multipart form data
	var b bytes.Buffer
	w := multipart.NewWriter(&b)

	// Add operations field
	operations := map[string]any{
		"query": deploymentQuery,
		"variables": map[string]any{
			"input": map[string]any{
				"workspaceId": wId1.String(),
				"description": "Test deployment description",
				"file":        nil,
			},
		},
	}
	operationsJSON, err := json.Marshal(operations)
	assert.NoError(t, err)

	err = w.WriteField("operations", string(operationsJSON))
	assert.NoError(t, err)

	// Add map field
	err = w.WriteField("map", `{"0": ["variables.input.file"]}`)
	assert.NoError(t, err)

	// Add file
	workflowContent := `{
		"name": "Test Workflow",
		"version": "1.0",
		"steps": []
	}`
	part, err := w.CreateFormFile("0", "workflow.json")
	assert.NoError(t, err)
	_, err = part.Write([]byte(workflowContent))
	assert.NoError(t, err)

	err = w.Close()
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("Content-Type", w.FormDataContentType()).
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(b.Bytes()).
		Expect().Status(http.StatusOK)

	var deploymentResult struct {
		Data struct {
			CreateDeployment struct {
				Deployment struct {
					ID string `json:"id"`
				} `json:"deployment"`
			} `json:"createDeployment"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &deploymentResult)
	assert.NoError(t, err)
	deploymentId := deploymentResult.Data.CreateDeployment.Deployment.ID

	// Create multiple jobs
	for i := 0; i < 5; i++ {
		jobQuery := `mutation($input: ExecuteDeploymentInput!) {
			executeDeployment(input: $input) {
				job {
					id
				}
			}
		}`

		jobVariables := fmt.Sprintf(`{
			"input": {
				"deploymentId": "%s"
			}
		}`, deploymentId)

		var jobVariablesMap map[string]any
		err := json.Unmarshal([]byte(jobVariables), &jobVariablesMap)
		assert.NoError(t, err)

		request := GraphQLRequest{
			Query:     jobQuery,
			Variables: jobVariablesMap,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var result struct {
			Data struct {
				ExecuteDeployment struct {
					Job struct {
						ID string `json:"id"`
					} `json:"job"`
				} `json:"executeDeployment"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Add a small delay to ensure the job is saved
		time.Sleep(100 * time.Millisecond)
	}

	// Test pagination
	t.Run("test_pagination", func(t *testing.T) {
		query := fmt.Sprintf(`{
			jobs(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 2
				}
			) {
				nodes {
					id
					status
				}
				pageInfo {
					totalCount
					currentPage
					totalPages
				}
			}
		}`, wId1.String())

		request := GraphQLRequest{
			Query: query,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var result struct {
			Data struct {
				Jobs struct {
					Nodes []struct {
						ID     string `json:"id"`
						Status string `json:"status"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						CurrentPage int `json:"currentPage"`
						TotalPages  int `json:"totalPages"`
					} `json:"pageInfo"`
				} `json:"jobs"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify first page results
		assert.Len(t, result.Data.Jobs.Nodes, 2, "Should return exactly 2 jobs")
		assert.Greater(t, result.Data.Jobs.PageInfo.TotalCount, 0, "Total count should be greater than zero")
	})

	// Test sorting
	t.Run("test_sorting", func(t *testing.T) {
		// First test DESC order
		query := fmt.Sprintf(`{
			jobs(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 5
					orderBy: "startedAt"
					orderDir: DESC
				}
			) {
				nodes {
					id
					startedAt
				}
			}
		}`, wId1.String())

		request := GraphQLRequest{
			Query: query,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var resultDesc struct {
			Data struct {
				Jobs struct {
					Nodes []struct {
						ID        string    `json:"id"`
						StartedAt time.Time `json:"startedAt"`
					} `json:"nodes"`
				} `json:"jobs"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &resultDesc)
		assert.NoError(t, err)

		// Verify DESC order
		for i := 1; i < len(resultDesc.Data.Jobs.Nodes); i++ {
			prev := resultDesc.Data.Jobs.Nodes[i-1].StartedAt
			curr := resultDesc.Data.Jobs.Nodes[i].StartedAt
			assert.True(t, prev.After(curr) || prev.Equal(curr), "Jobs should be sorted by startedAt in descending order")
		}

		// Now test ASC order
		query = fmt.Sprintf(`{
			jobs(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 5
					orderBy: "startedAt"
					orderDir: ASC
				}
			) {
				nodes {
					id
					startedAt
				}
			}
		}`, wId1.String())

		request = GraphQLRequest{
			Query: query,
		}
		jsonData, err = json.Marshal(request)
		assert.NoError(t, err)

		resp = e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var resultAsc struct {
			Data struct {
				Jobs struct {
					Nodes []struct {
						ID        string    `json:"id"`
						StartedAt time.Time `json:"startedAt"`
					} `json:"nodes"`
				} `json:"jobs"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &resultAsc)
		assert.NoError(t, err)

		// Verify ASC order
		for i := 1; i < len(resultAsc.Data.Jobs.Nodes); i++ {
			prev := resultAsc.Data.Jobs.Nodes[i-1].StartedAt
			curr := resultAsc.Data.Jobs.Nodes[i].StartedAt
			assert.True(t, prev.Before(curr) || prev.Equal(curr), "Jobs should be sorted by startedAt in ascending order")
		}

		// Verify that ASC and DESC orders are opposite of each other
		if len(resultAsc.Data.Jobs.Nodes) > 0 && len(resultDesc.Data.Jobs.Nodes) > 0 {
			assert.Equal(t,
				resultAsc.Data.Jobs.Nodes[0].StartedAt,
				resultDesc.Data.Jobs.Nodes[len(resultDesc.Data.Jobs.Nodes)-1].StartedAt,
				"First element in ASC should equal last element in DESC",
			)
		}
	})

	// Test page-based pagination
	t.Run("test_page_pagination", func(t *testing.T) {
		// Test first page
		query := fmt.Sprintf(`{
			jobs(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 2
				}
			) {
				nodes {
					id
					status
				}
				pageInfo {
					totalCount
					currentPage
					totalPages
				}
			}
		}`, wId1.String())

		request := GraphQLRequest{
			Query: query,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var result struct {
			Data struct {
				Jobs struct {
					Nodes []struct {
						ID     string `json:"id"`
						Status string `json:"status"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						CurrentPage int `json:"currentPage"`
						TotalPages  int `json:"totalPages"`
					} `json:"pageInfo"`
				} `json:"jobs"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify first page results
		assert.Len(t, result.Data.Jobs.Nodes, 2)
		assert.Equal(t, 1, result.Data.Jobs.PageInfo.CurrentPage)
		assert.Equal(t, 5, result.Data.Jobs.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.Jobs.PageInfo.TotalPages)

		// Test second page
		query = fmt.Sprintf(`{
			jobs(
				workspaceId: "%s"
				pagination: {
					page: 2
					pageSize: 2
				}
			) {
				nodes {
					id
					status
				}
				pageInfo {
					totalCount
					currentPage
					totalPages
				}
			}
		}`, wId1.String())

		request = GraphQLRequest{
			Query: query,
		}
		jsonData, err = json.Marshal(request)
		assert.NoError(t, err)

		resp = e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify second page results
		assert.Equal(t, 2, result.Data.Jobs.PageInfo.CurrentPage)
		assert.Equal(t, 5, result.Data.Jobs.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.Jobs.PageInfo.TotalPages)
		assert.Len(t, result.Data.Jobs.Nodes, 2)
	})
}

func TestTriggersPagination(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser)

	// Create a test deployment first
	deploymentId := createTestDeployment(t, e)
	assert.NotEmpty(t, deploymentId)

	// Create multiple triggers for testing
	triggerIDs := make([]string, 5)
	for i := 0; i < 5; i++ {
		query := `mutation($input: CreateTriggerInput!) {
			createTrigger(input: $input) {
				id
			}
		}`

		variables := map[string]interface{}{
			"input": map[string]interface{}{
				"workspaceId":  wId1.String(),
				"deploymentId": deploymentId,
				"description":  fmt.Sprintf("Test Trigger %d", i),
				"timeDriverInput": map[string]interface{}{
					"interval": "EVERY_DAY",
				},
			},
		}

		request := GraphQLRequest{
			Query:     query,
			Variables: variables,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var result struct {
			Data struct {
				CreateTrigger struct {
					ID string `json:"id"`
				} `json:"createTrigger"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)
		assert.NotEmpty(t, result.Data.CreateTrigger.ID, "Trigger creation failed")
		triggerIDs[i] = result.Data.CreateTrigger.ID

		// Add a small delay between trigger creations
		time.Sleep(100 * time.Millisecond)
	}

	// Verify all triggers were created
	assert.Len(t, triggerIDs, 5, "Expected 5 triggers to be created")
	for i, id := range triggerIDs {
		assert.NotEmpty(t, id, fmt.Sprintf("Trigger %d was not created successfully", i))
	}

	// Test page-based pagination
	t.Run("test_page_based_pagination", func(t *testing.T) {
		query := fmt.Sprintf(`{
			triggers(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 2
				}
			) {
				nodes {
					id
					description
				}
				pageInfo {
					totalCount
					totalPages
					currentPage
				}
			}
		}`, wId1.String())

		request := GraphQLRequest{
			Query: query,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var result struct {
			Data struct {
				Triggers struct {
					Nodes []struct {
						ID          string `json:"id"`
						Description string `json:"description"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						TotalPages  int `json:"totalPages"`
						CurrentPage int `json:"currentPage"`
					} `json:"pageInfo"`
				} `json:"triggers"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		assert.Len(t, result.Data.Triggers.Nodes, 2)
		assert.Equal(t, 5, result.Data.Triggers.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.Triggers.PageInfo.TotalPages)
		assert.Equal(t, 1, result.Data.Triggers.PageInfo.CurrentPage)
	})

	// Test sorting
	t.Run("test_sorting", func(t *testing.T) {
		// First test ASC order
		query := fmt.Sprintf(`{
			triggers(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 5
					orderBy: "description"
					orderDir: ASC
				}
			) {
				nodes {
					id
					description
				}
			}
		}`, wId1.String())

		request := GraphQLRequest{
			Query: query,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var resultAsc struct {
			Data struct {
				Triggers struct {
					Nodes []struct {
						ID          string `json:"id"`
						Description string `json:"description"`
					} `json:"nodes"`
				} `json:"triggers"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &resultAsc)
		assert.NoError(t, err)

		// Verify ASC order
		for i := 1; i < len(resultAsc.Data.Triggers.Nodes); i++ {
			prev := resultAsc.Data.Triggers.Nodes[i-1].Description
			curr := resultAsc.Data.Triggers.Nodes[i].Description
			assert.True(t, prev <= curr, "Triggers should be sorted by description in ascending order")
		}

		// Now test DESC order
		query = fmt.Sprintf(`{
			triggers(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 5
					orderBy: "description"
					orderDir: DESC
				}
			) {
				nodes {
					id
					description
				}
			}
		}`, wId1.String())

		request = GraphQLRequest{
			Query: query,
		}
		jsonData, err = json.Marshal(request)
		assert.NoError(t, err)

		resp = e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var resultDesc struct {
			Data struct {
				Triggers struct {
					Nodes []struct {
						ID          string `json:"id"`
						Description string `json:"description"`
					} `json:"nodes"`
				} `json:"triggers"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &resultDesc)
		assert.NoError(t, err)

		// Verify DESC order
		for i := 1; i < len(resultDesc.Data.Triggers.Nodes); i++ {
			prev := resultDesc.Data.Triggers.Nodes[i-1].Description
			curr := resultDesc.Data.Triggers.Nodes[i].Description
			assert.True(t, prev >= curr, "Triggers should be sorted by description in descending order")
		}

		// Verify that ASC and DESC orders are opposite of each other
		if len(resultAsc.Data.Triggers.Nodes) > 0 && len(resultDesc.Data.Triggers.Nodes) > 0 {
			assert.Equal(t,
				resultAsc.Data.Triggers.Nodes[0].Description,
				resultDesc.Data.Triggers.Nodes[len(resultDesc.Data.Triggers.Nodes)-1].Description,
				"First element in ASC should equal last element in DESC",
			)
		}
	})

	// Test last page
	t.Run("test_last_page", func(t *testing.T) {
		query := fmt.Sprintf(`{
			triggers(
				workspaceId: "%s"
				pagination: {
					page: 3
					pageSize: 2
				}
			) {
				nodes {
					id
					description
				}
				pageInfo {
					totalCount
					totalPages
					currentPage
				}
			}
		}`, wId1.String())

		request := GraphQLRequest{
			Query: query,
		}
		jsonData, err := json.Marshal(request)
		assert.NoError(t, err)

		resp := e.POST("/api/graphql").
			WithHeader("Content-Type", "application/json").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithBytes(jsonData).
			Expect().Status(http.StatusOK)

		var result struct {
			Data struct {
				Triggers struct {
					Nodes []struct {
						ID          string `json:"id"`
						Description string `json:"description"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						TotalPages  int `json:"totalPages"`
						CurrentPage int `json:"currentPage"`
					} `json:"pageInfo"`
				} `json:"triggers"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		assert.Len(t, result.Data.Triggers.Nodes, 1) // Last page should have 1 item
		assert.Equal(t, 5, result.Data.Triggers.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.Triggers.PageInfo.TotalPages)
		assert.Equal(t, 3, result.Data.Triggers.PageInfo.CurrentPage)
	})
}
