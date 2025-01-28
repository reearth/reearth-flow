package e2e

import (
	"encoding/json"
	"fmt"
	"net/http"
	"testing"

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
	}, true, baseSeederUser)

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
	}, true, baseSeederUser)

	query := fmt.Sprintf(`{
		projects(
			workspaceId: "%s"
			includeArchived: true
			pagination: {
				first: 10
			}
		) {
			edges {
				node {
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
				cursor
			}
			pageInfo {
				hasNextPage
				hasPreviousPage
				startCursor
				endCursor
				totalCount
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
			Projects struct {
				Edges []struct {
					Node struct {
						ID                string `json:"id"`
						Name              string `json:"name"`
						Description       string `json:"description"`
						IsArchived        bool   `json:"isArchived"`
						IsBasicAuthActive bool   `json:"isBasicAuthActive"`
						BasicAuthUsername string `json:"basicAuthUsername"`
						BasicAuthPassword string `json:"basicAuthPassword"`
						Version           int    `json:"version"`
						WorkspaceID       string `json:"workspaceId"`
					} `json:"node"`
					Cursor string `json:"cursor"`
				} `json:"edges"`
				PageInfo struct {
					HasNextPage     bool   `json:"hasNextPage"`
					HasPreviousPage bool   `json:"hasPreviousPage"`
					StartCursor     string `json:"startCursor"`
					EndCursor       string `json:"endCursor"`
					TotalCount      int    `json:"totalCount"`
				} `json:"pageInfo"`
			} `json:"projects"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	// Verify the response
	projects := result.Data.Projects
	assert.NotNil(t, projects.Edges)
	for _, edge := range projects.Edges {
		assert.NotEmpty(t, edge.Node.ID)
		assert.NotEmpty(t, edge.Node.Name)
		assert.Equal(t, wId1.String(), edge.Node.WorkspaceID)
		assert.NotEmpty(t, edge.Cursor)
	}
	assert.NotNil(t, projects.PageInfo)
}
