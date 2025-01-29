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
			projectsPage(
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
				} `json:"projectsPage"`
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
		query := fmt.Sprintf(`{
			projectsPage(
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

		var result struct {
			Data struct {
				ProjectsPage struct {
					Nodes []struct {
						ID   string `json:"id"`
						Name string `json:"name"`
					} `json:"nodes"`
				} `json:"projectsPage"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify sorting
		for i := 1; i < len(result.Data.ProjectsPage.Nodes); i++ {
			prev := result.Data.ProjectsPage.Nodes[i-1].Name
			curr := result.Data.ProjectsPage.Nodes[i].Name
			assert.True(t, prev <= curr, "Projects should be sorted by name in ascending order")
		}
	})

	// Test last page
	t.Run("test_last_page", func(t *testing.T) {
		query := fmt.Sprintf(`{
			projectsPage(
				workspaceId: "%s"
				pagination: {
					page: 3
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
				} `json:"projectsPage"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		assert.Len(t, result.Data.ProjectsPage.Nodes, 1) // Last page should have 1 item
		assert.Equal(t, 5, result.Data.ProjectsPage.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.ProjectsPage.PageInfo.TotalPages)
		assert.Equal(t, 3, result.Data.ProjectsPage.PageInfo.CurrentPage)
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
	}

	// Test pagination
	t.Run("test_pagination", func(t *testing.T) {
		query := fmt.Sprintf(`{
			jobsPage(
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
				JobsPage struct {
					Nodes []struct {
						ID     string `json:"id"`
						Status string `json:"status"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						CurrentPage int `json:"currentPage"`
						TotalPages  int `json:"totalPages"`
					} `json:"pageInfo"`
				} `json:"jobsPage"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify first page results
		assert.Len(t, result.Data.JobsPage.Nodes, 2, "Should return exactly 2 jobs")
		assert.Greater(t, result.Data.JobsPage.PageInfo.TotalCount, 0, "Total count should be greater than zero")
	})

	// Test sorting
	t.Run("test_sorting", func(t *testing.T) {
		query := fmt.Sprintf(`{
			jobsPage(
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

		var result struct {
			Data struct {
				JobsPage struct {
					Nodes []struct {
						ID        string    `json:"id"`
						StartedAt time.Time `json:"startedAt"`
					} `json:"nodes"`
				} `json:"jobsPage"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)
		// Verify pagination results
		assert.Len(t, result.Data.JobsPage.Nodes, 5, "Should return exactly 5 jobs")
		// Verify sorting
		for i := 1; i < len(result.Data.JobsPage.Nodes); i++ {
			prev := result.Data.JobsPage.Nodes[i-1].StartedAt
			curr := result.Data.JobsPage.Nodes[i].StartedAt
			assert.True(t, prev.After(curr), "Jobs should be sorted by startedAt in descending order")
		}
	})

	// Test sorting in ascending order
	t.Run("test_sorting_ascending", func(t *testing.T) {
		query := fmt.Sprintf(`{
			jobsPage(
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
				JobsPage struct {
					Nodes []struct {
						ID        string    `json:"id"`
						StartedAt time.Time `json:"startedAt"`
					} `json:"nodes"`
				} `json:"jobsPage"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)
		// Verify sorting
		for i := 1; i < len(result.Data.JobsPage.Nodes); i++ {
			prev := result.Data.JobsPage.Nodes[i-1].StartedAt
			curr := result.Data.JobsPage.Nodes[i].StartedAt
			assert.True(t, prev.Before(curr), "Jobs should be sorted by startedAt in ascending order")
		}
	})

	// Test page-based pagination
	t.Run("test_page_pagination", func(t *testing.T) {
		// Test first page
		query := fmt.Sprintf(`{
			jobsPage(
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
				JobsPage struct {
					Nodes []struct {
						ID     string `json:"id"`
						Status string `json:"status"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						CurrentPage int `json:"currentPage"`
						TotalPages  int `json:"totalPages"`
					} `json:"pageInfo"`
				} `json:"jobsPage"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify first page results
		assert.Len(t, result.Data.JobsPage.Nodes, 2)
		assert.Equal(t, 1, result.Data.JobsPage.PageInfo.CurrentPage)
		assert.Equal(t, 5, result.Data.JobsPage.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.JobsPage.PageInfo.TotalPages)

		// Test second page
		query = fmt.Sprintf(`{
			jobsPage(
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
		assert.Equal(t, 2, result.Data.JobsPage.PageInfo.CurrentPage)
		assert.Equal(t, 5, result.Data.JobsPage.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.JobsPage.PageInfo.TotalPages)
		assert.Len(t, result.Data.JobsPage.Nodes, 2)
	})
}
