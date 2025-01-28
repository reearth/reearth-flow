package e2e

import (
	"encoding/json"
	"fmt"
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
		projectIDs[i] = result.Data.CreateProject.Project.ID
	}

	// Test pagination
	t.Run("test_pagination", func(t *testing.T) {
		query := fmt.Sprintf(`{
			projects(
				workspaceId: "%s"
				pagination: {
					first: 2
				}
			) {
				edges {
					node {
						id
						name
					}
				}
				pageInfo {
					hasNextPage
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
							ID   string `json:"id"`
							Name string `json:"name"`
						} `json:"node"`
					} `json:"edges"`
					PageInfo struct {
						HasNextPage bool   `json:"hasNextPage"`
						EndCursor   string `json:"endCursor"`
						TotalCount  int    `json:"totalCount"`
					} `json:"pageInfo"`
				} `json:"projects"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		assert.Len(t, result.Data.Projects.Edges, 2)
		assert.True(t, result.Data.Projects.PageInfo.HasNextPage)
		assert.Equal(t, 5, result.Data.Projects.PageInfo.TotalCount)
	})

	// Test sorting
	t.Run("test_sorting", func(t *testing.T) {
		query := fmt.Sprintf(`{
			projects(
				workspaceId: "%s"
				pagination: {
					first: 5
					orderBy: "name"
					orderDir: ASC
				}
			) {
				edges {
					node {
						id
						name
					}
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
							ID   string `json:"id"`
							Name string `json:"name"`
						} `json:"node"`
					} `json:"edges"`
				} `json:"projects"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify sorting
		for i := 1; i < len(result.Data.Projects.Edges); i++ {
			prev := result.Data.Projects.Edges[i-1].Node.Name
			curr := result.Data.Projects.Edges[i].Node.Name
			assert.True(t, prev <= curr, "Projects should be sorted by name in ascending order")
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

	// Test pagination
	t.Run("test_pagination", func(t *testing.T) {
		query := fmt.Sprintf(`{
			jobs(
				workspaceId: "%s"
				pagination: {
					first: 2
				}
			) {
				edges {
					node {
						id
						status
					}
				}
				pageInfo {
					hasNextPage
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
				Jobs struct {
					Edges []struct {
						Node struct {
							ID     string `json:"id"`
							Status string `json:"status"`
						} `json:"node"`
					} `json:"edges"`
					PageInfo struct {
						HasNextPage bool   `json:"hasNextPage"`
						EndCursor   string `json:"endCursor"`
						TotalCount  int    `json:"totalCount"`
					} `json:"pageInfo"`
				} `json:"jobs"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify pagination results
		assert.Len(t, result.Data.Jobs.Edges, 2, "Should return exactly 2 jobs")
		assert.NotZero(t, result.Data.Jobs.PageInfo.TotalCount, "Total count should be greater than zero")
		if result.Data.Jobs.PageInfo.TotalCount > 2 {
			assert.True(t, result.Data.Jobs.PageInfo.HasNextPage, "Should have next page")
			assert.NotEmpty(t, result.Data.Jobs.PageInfo.EndCursor, "End cursor should not be empty")
		}
	})

	// Test sorting
	t.Run("test_sorting", func(t *testing.T) {
		query := fmt.Sprintf(`{
			jobs(
				workspaceId: "%s"
				pagination: {
					first: 5
					orderBy: "startedAt"
					orderDir: DESC
				}
			) {
				edges {
					node {
						id
						startedAt
					}
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
				Jobs struct {
					Edges []struct {
						Node struct {
							ID        string    `json:"id"`
							StartedAt time.Time `json:"startedAt"`
						} `json:"node"`
					} `json:"edges"`
				} `json:"jobs"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)
		// Verify pagination results
		assert.Len(t, result.Data.Jobs.Edges, 2, "Should return exactly 2 jobs")
		// Verify sorting
		for i := 1; i < len(result.Data.Jobs.Edges); i++ {
			prev := result.Data.Jobs.Edges[i-1].Node.StartedAt
			curr := result.Data.Jobs.Edges[i].Node.StartedAt
			assert.True(t, prev.After(curr), "Jobs should be sorted by startedAt in descending order")
		}
	})

	// Test sorting in ascending order
	t.Run("test_sorting_ascending", func(t *testing.T) {
		query := fmt.Sprintf(`{
			jobs(
				workspaceId: "%s"
				pagination: {
					first: 5
					orderBy: "startedAt"
					orderDir: ASC
				}
			) {
				edges {
					node {
						id
						startedAt
					}
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
				Jobs struct {
					Edges []struct {
						Node struct {
							ID        string    `json:"id"`
							StartedAt time.Time `json:"startedAt"`
						} `json:"node"`
					} `json:"edges"`
				} `json:"jobs"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)
		// Verify sorting
		for i := 1; i < len(result.Data.Jobs.Edges); i++ {
			prev := result.Data.Jobs.Edges[i-1].Node.StartedAt
			curr := result.Data.Jobs.Edges[i].Node.StartedAt
			assert.True(t, prev.Before(curr), "Jobs should be sorted by startedAt in ascending order")
		}
	})
}

func TestTriggersPagination(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser)

	// Test pagination
	t.Run("test_pagination", func(t *testing.T) {
		query := fmt.Sprintf(`{
			triggers(
				workspaceId: "%s"
				pagination: {
					first: 2
				}
			) {
				edges {
					node {
						id
						description
					}
				}
				pageInfo {
					hasNextPage
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
				Triggers struct {
					Edges []struct {
						Node struct {
							ID          string `json:"id"`
							Description string `json:"description"`
						} `json:"node"`
					} `json:"edges"`
					PageInfo struct {
						HasNextPage bool   `json:"hasNextPage"`
						EndCursor   string `json:"endCursor"`
						TotalCount  int    `json:"totalCount"`
					} `json:"pageInfo"`
				} `json:"triggers"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)
	})

	// Test sorting
	t.Run("test_sorting", func(t *testing.T) {
		query := fmt.Sprintf(`{
			triggers(
				workspaceId: "%s"
				pagination: {
					first: 5
					orderBy: "createdAt"
					orderDir: DESC
				}
			) {
				edges {
					node {
						id
						createdAt
					}
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
				Triggers struct {
					Edges []struct {
						Node struct {
							ID        string    `json:"id"`
							CreatedAt time.Time `json:"createdAt"`
						} `json:"node"`
					} `json:"edges"`
				} `json:"triggers"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify sorting
		for i := 1; i < len(result.Data.Triggers.Edges); i++ {
			prev := result.Data.Triggers.Edges[i-1].Node.CreatedAt
			curr := result.Data.Triggers.Edges[i].Node.CreatedAt
			assert.True(t, prev.After(curr), "Triggers should be sorted by createdAt in descending order")
		}
	})
}
