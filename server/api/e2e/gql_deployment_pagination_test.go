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
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
)

func TestDeploymentsPagination(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operator := factory.NewUser(func(b *pkguser.Builder) {})
	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	// Log workspace and user info
	t.Log("Workspace ID:", wId1.String())
	t.Log("User ID:", uId1.String())

	// Create multiple deployments for testing
	deploymentIDs := make([]string, 5)
	for i := 0; i < 5; i++ {
		// Create multipart form data
		var b bytes.Buffer
		w := multipart.NewWriter(&b)

		// Add operations field
		operations := map[string]any{
			"query": `mutation($input: CreateDeploymentInput!) {
				createDeployment(input: $input) {
					deployment {
						id
					}
				}
			}`,
			"variables": map[string]any{
				"input": map[string]any{
					"workspaceId": wId1.String(),
					"description": fmt.Sprintf("Test Deployment %d", i),
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

		// Add workflow file
		workflowContent := fmt.Sprintf(`{
			"name": "Test Workflow %d",
			"version": "1.0",
			"steps": []
		}`, i)
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

		var createResult struct {
			Data struct {
				CreateDeployment struct {
					Deployment struct {
						ID string `json:"id"`
					} `json:"deployment"`
				} `json:"createDeployment"`
			} `json:"data"`
			Errors []struct {
				Message string `json:"message"`
			} `json:"errors"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &createResult)
		assert.NoError(t, err)

		// Check for GraphQL errors
		if len(createResult.Errors) > 0 {
			t.Logf("GraphQL Error: %v", createResult.Errors[0].Message)
			t.FailNow()
		}

		assert.NotEmpty(t, createResult.Data.CreateDeployment.Deployment.ID)
		deploymentIDs[i] = createResult.Data.CreateDeployment.Deployment.ID

		// Add debug logging
		t.Logf("Created deployment with ID: %s", deploymentIDs[i])

		// Add a small delay between deployments
		time.Sleep(100 * time.Millisecond)
	}

	// Verify all deployments were created
	assert.Len(t, deploymentIDs, 5, "Expected 5 deployments to be created")
	for i, id := range deploymentIDs {
		assert.NotEmpty(t, id, fmt.Sprintf("Deployment %d was not created successfully", i))
	}

	// Test basic pagination
	t.Run("test_basic_pagination", func(t *testing.T) {
		query := fmt.Sprintf(`{
			deployments(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 2
				}
			) {
				nodes {
					id
					description
					workflowUrl
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

		// Log raw response for debugging
		t.Logf("Raw response: %s", resp.Body().Raw())

		var result struct {
			Data struct {
				Deployments struct {
					Nodes []struct {
						ID          string `json:"id"`
						Description string `json:"description"`
						WorkflowURL string `json:"workflowUrl"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						TotalPages  int `json:"totalPages"`
						CurrentPage int `json:"currentPage"`
					} `json:"pageInfo"`
				} `json:"deployments"`
			} `json:"data"`
			Errors []struct {
				Message string   `json:"message"`
				Path    []string `json:"path"`
			} `json:"errors"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Log errors if any
		if len(result.Errors) > 0 {
			t.Logf("GraphQL Errors: %+v", result.Errors)
		}

		// Log deployment IDs
		t.Log("Created deployment IDs:", deploymentIDs)

		// Log page info
		t.Logf("Page Info: totalCount=%d, totalPages=%d, currentPage=%d",
			result.Data.Deployments.PageInfo.TotalCount,
			result.Data.Deployments.PageInfo.TotalPages,
			result.Data.Deployments.PageInfo.CurrentPage)

		// Log nodes
		t.Logf("Returned nodes: %+v", result.Data.Deployments.Nodes)

		// Check for GraphQL errors
		if len(result.Errors) > 0 {
			t.Logf("GraphQL Error: %v", result.Errors[0].Message)
			t.FailNow()
		}

		assert.Len(t, result.Data.Deployments.Nodes, 2)
		assert.Equal(t, 5, result.Data.Deployments.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.Deployments.PageInfo.TotalPages)
		assert.Equal(t, 1, result.Data.Deployments.PageInfo.CurrentPage)
	})

	// Test sorting by description
	t.Run("test_sorting_by_description", func(t *testing.T) {
		// First test ASC order
		query := fmt.Sprintf(`{
			deployments(
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
				Deployments struct {
					Nodes []struct {
						ID          string `json:"id"`
						Description string `json:"description"`
					} `json:"nodes"`
				} `json:"deployments"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &resultAsc)
		assert.NoError(t, err)

		// Verify ASC order
		for i := 1; i < len(resultAsc.Data.Deployments.Nodes); i++ {
			prev := resultAsc.Data.Deployments.Nodes[i-1].Description
			curr := resultAsc.Data.Deployments.Nodes[i].Description
			assert.True(t, prev <= curr, "Deployments should be sorted by description in ascending order")
		}

		// Now test DESC order
		query = fmt.Sprintf(`{
			deployments(
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
				Deployments struct {
					Nodes []struct {
						ID          string `json:"id"`
						Description string `json:"description"`
					} `json:"nodes"`
				} `json:"deployments"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &resultDesc)
		assert.NoError(t, err)

		// Verify DESC order
		for i := 1; i < len(resultDesc.Data.Deployments.Nodes); i++ {
			prev := resultDesc.Data.Deployments.Nodes[i-1].Description
			curr := resultDesc.Data.Deployments.Nodes[i].Description
			assert.True(t, prev >= curr, "Deployments should be sorted by description in descending order")
		}

		// Verify that ASC and DESC orders are opposite of each other
		if len(resultAsc.Data.Deployments.Nodes) > 0 && len(resultDesc.Data.Deployments.Nodes) > 0 {
			assert.Equal(t,
				resultAsc.Data.Deployments.Nodes[0].Description,
				resultDesc.Data.Deployments.Nodes[len(resultDesc.Data.Deployments.Nodes)-1].Description,
				"First element in ASC should equal last element in DESC")
		}
	})

	// Test last page
	t.Run("test_last_page", func(t *testing.T) {
		query := fmt.Sprintf(`{
			deployments(
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
				Deployments struct {
					Nodes []struct {
						ID          string `json:"id"`
						Description string `json:"description"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						TotalPages  int `json:"totalPages"`
						CurrentPage int `json:"currentPage"`
					} `json:"pageInfo"`
				} `json:"deployments"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		assert.Len(t, result.Data.Deployments.Nodes, 1) // Last page should have 1 item
		assert.Equal(t, 5, result.Data.Deployments.PageInfo.TotalCount)
		assert.Equal(t, 3, result.Data.Deployments.PageInfo.TotalPages)
		assert.Equal(t, 3, result.Data.Deployments.PageInfo.CurrentPage)
	})

	// Test sorting by creation time
	t.Run("test_sorting_by_creation_time", func(t *testing.T) {
		query := fmt.Sprintf(`{
			deployments(
				workspaceId: "%s"
				pagination: {
					page: 1
					pageSize: 5
					orderBy: "updatedAt"
					orderDir: DESC
				}
			) {
				nodes {
					id
					updatedAt
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
				Deployments struct {
					Nodes []struct {
						ID        string    `json:"id"`
						UpdatedAt time.Time `json:"updatedAt"`
					} `json:"nodes"`
				} `json:"deployments"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Verify sorting
		for i := 1; i < len(result.Data.Deployments.Nodes); i++ {
			prev := result.Data.Deployments.Nodes[i-1].UpdatedAt
			curr := result.Data.Deployments.Nodes[i].UpdatedAt
			assert.True(t, prev.After(curr) || prev.Equal(curr), "Deployments should be sorted by updatedAt in descending order")
		}
	})

	// Test invalid page number
	t.Run("test_invalid_page", func(t *testing.T) {
		query := fmt.Sprintf(`{
			deployments(
				workspaceId: "%s"
				pagination: {
					page: 999
					pageSize: 2
				}
			) {
				nodes {
					id
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
				Deployments struct {
					Nodes []struct {
						ID string `json:"id"`
					} `json:"nodes"`
					PageInfo struct {
						TotalCount  int `json:"totalCount"`
						TotalPages  int `json:"totalPages"`
						CurrentPage int `json:"currentPage"`
					} `json:"pageInfo"`
				} `json:"deployments"`
			} `json:"data"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Should return empty nodes for invalid page
		assert.Len(t, result.Data.Deployments.Nodes, 0)
	})
}
