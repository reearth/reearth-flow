package e2e

import (
	"encoding/json"
	"fmt"
	"net/http"
	"testing"
	"time"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/stretchr/testify/assert"
)

func TestProjectWorkflows(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	projectId := testCreateProject(t, e)

	// Test update project
	testUpdateProject(t, e, projectId)

	// Test delete project
	testDeleteProject(t, e, projectId)
}

func testCreateProject(t *testing.T, e *httpexpect.Expect) string {
	query := `mutation($input: CreateProjectInput!) {
		createProject(input: $input) {
			project {
				id
				name
				description
				isArchived
				isBasicAuthActive
				basicAuthUsername
				basicAuthPassword
				version
				createdAt
				updatedAt
				workspaceId
			}
		}
	}`

	variables := fmt.Sprintf(`{
		"input": {
			"workspaceId": "%s",
			"name": "Test Project",
			"description": "Test project description",
			"archived": false
		}
	}`, wId1.String())

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
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CreateProject struct {
				Project struct {
					ID          string `json:"id"`
					Name        string `json:"name"`
					Description string `json:"description"`
					IsArchived  bool   `json:"isArchived"`
					WorkspaceID string `json:"workspaceId"`
					Version     int    `json:"version"`
				} `json:"project"`
			} `json:"createProject"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	project := result.Data.CreateProject.Project
	assert.NotEmpty(t, project.ID)
	assert.Equal(t, "Test Project", project.Name)
	assert.Equal(t, "Test project description", project.Description)
	assert.False(t, project.IsArchived)
	assert.Equal(t, wId1.String(), project.WorkspaceID)
	assert.Equal(t, 0, project.Version)

	return project.ID
}

func testUpdateProject(t *testing.T, e *httpexpect.Expect, projectId string) {
	query := `mutation($input: UpdateProjectInput!) {
		updateProject(input: $input) {
			project {
				id
				name
				description
				isArchived
				isBasicAuthActive
				basicAuthUsername
				basicAuthPassword
			}
		}
	}`

	variables := fmt.Sprintf(`{
		"input": {
			"projectId": "%s",
			"name": "Updated Project",
			"description": "Updated description",
			"archived": true,
			"isBasicAuthActive": true,
			"basicAuthUsername": "testuser",
			"basicAuthPassword": "testpass"
		}
	}`, projectId)

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
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			UpdateProject struct {
				Project struct {
					ID                string `json:"id"`
					Name              string `json:"name"`
					Description       string `json:"description"`
					IsArchived        bool   `json:"isArchived"`
					IsBasicAuthActive bool   `json:"isBasicAuthActive"`
					BasicAuthUsername string `json:"basicAuthUsername"`
					BasicAuthPassword string `json:"basicAuthPassword"`
				} `json:"project"`
			} `json:"updateProject"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	project := result.Data.UpdateProject.Project
	assert.Equal(t, projectId, project.ID)
	assert.Equal(t, "Updated Project", project.Name)
	assert.Equal(t, "Updated description", project.Description)
	assert.True(t, project.IsArchived)
	assert.True(t, project.IsBasicAuthActive)
	assert.Equal(t, "testuser", project.BasicAuthUsername)
	assert.Equal(t, "testpass", project.BasicAuthPassword)
}

func testDeleteProject(t *testing.T, e *httpexpect.Expect, projectId string) {
	query := `mutation($input: DeleteProjectInput!) {
		deleteProject(input: $input) {
			projectId
		}
	}`

	variables := fmt.Sprintf(`{
		"input": {
			"projectId": "%s"
		}
	}`, projectId)

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
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			DeleteProject struct {
				ProjectID string `json:"projectId"`
			} `json:"deleteProject"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	assert.Equal(t, projectId, result.Data.DeleteProject.ProjectID)
}

func TestListProjects(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	// Create test projects
	projectIDs := make([]string, 3)
	for i := 0; i < 3; i++ {
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
			WithHeader("authorization", "Bearer test").
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

	// Test listing projects with pagination
	query := fmt.Sprintf(`{
		projects(
			workspaceId: "%s"
			pagination: {
				page: 1
				pageSize: 2
				orderBy: "name"
				orderDir: ASC
			}
		) {
			nodes {
				id
				name
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
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			ProjectsPage struct {
				Nodes []struct {
					ID          string `json:"id"`
					Name        string `json:"name"`
					Description string `json:"description"`
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

	// Verify the response
	projects := result.Data.ProjectsPage
	assert.NotNil(t, projects.Nodes)
	assert.Len(t, projects.Nodes, 2)
	for _, node := range projects.Nodes {
		assert.NotEmpty(t, node.ID)
		assert.NotEmpty(t, node.Name)
		assert.NotEmpty(t, node.Description)
	}
	assert.Equal(t, 3, projects.PageInfo.TotalCount)
	assert.Equal(t, 2, projects.PageInfo.TotalPages)
	assert.Equal(t, 1, projects.PageInfo.CurrentPage)
}
