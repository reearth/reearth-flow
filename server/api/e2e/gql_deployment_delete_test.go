package e2e

import (
	"bytes"
	"encoding/json"
	"mime/multipart"
	"net/http"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
)

func TestDeleteDeploymentWithTriggers(t *testing.T) {
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

	// Create a deployment
	deploymentID := createDeploymentForTest(t, e)
	assert.NotEmpty(t, deploymentID)
	t.Logf("Created deployment: %s", deploymentID)

	// Create a trigger for the deployment
	triggerID := createTriggerForDeployment(t, e, deploymentID)
	assert.NotEmpty(t, triggerID)
	t.Logf("Created trigger: %s", triggerID)

	// Test: Attempt to delete deployment with active trigger (should fail)
	t.Run("should_fail_to_delete_deployment_with_trigger", func(t *testing.T) {
		mutation := `mutation($input: DeleteDeploymentInput!) {
			deleteDeployment(input: $input) {
				deploymentId
			}
		}`

		request := GraphQLRequest{
			Query: mutation,
			Variables: map[string]interface{}{
				"input": map[string]interface{}{
					"deploymentId": deploymentID,
				},
			},
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
				DeleteDeployment *struct {
					DeploymentID string `json:"deploymentId"`
				} `json:"deleteDeployment"`
			} `json:"data"`
			Errors []struct {
				Message string `json:"message"`
			} `json:"errors"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Should have an error
		if assert.NotEmpty(t, result.Errors, "Expected error when deleting deployment with triggers") {
			assert.Contains(t, result.Errors[0].Message, "has active triggers", "Error should mention active triggers")
			t.Logf("Expected error received: %s", result.Errors[0].Message)
		}
	})

	// Delete the trigger
	t.Run("delete_trigger", func(t *testing.T) {
		mutation := `mutation($triggerId: ID!) {
			deleteTrigger(triggerId: $triggerId)
		}`

		request := GraphQLRequest{
			Query: mutation,
			Variables: map[string]interface{}{
				"triggerId": triggerID,
			},
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
				DeleteTrigger bool `json:"deleteTrigger"`
			} `json:"data"`
			Errors []struct {
				Message string `json:"message"`
			} `json:"errors"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Should succeed
		assert.Empty(t, result.Errors)
		assert.True(t, result.Data.DeleteTrigger)
		t.Log("Trigger deleted successfully")
	})

	// Test: Delete deployment without triggers (should succeed)
	t.Run("should_succeed_to_delete_deployment_without_triggers", func(t *testing.T) {
		mutation := `mutation($input: DeleteDeploymentInput!) {
			deleteDeployment(input: $input) {
				deploymentId
			}
		}`

		request := GraphQLRequest{
			Query: mutation,
			Variables: map[string]interface{}{
				"input": map[string]interface{}{
					"deploymentId": deploymentID,
				},
			},
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
				DeleteDeployment struct {
					DeploymentID string `json:"deploymentId"`
				} `json:"deleteDeployment"`
			} `json:"data"`
			Errors []struct {
				Message string `json:"message"`
			} `json:"errors"`
		}

		err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
		assert.NoError(t, err)

		// Should succeed
		assert.Empty(t, result.Errors, "Should not have errors when deleting deployment without triggers")
		assert.Equal(t, deploymentID, result.Data.DeleteDeployment.DeploymentID)
		t.Log("Deployment deleted successfully")
	})
}

func createDeploymentForTest(t *testing.T, e *httpexpect.Expect) string {
	t.Helper()

	var b bytes.Buffer
	w := multipart.NewWriter(&b)

	operations := map[string]interface{}{
		"query": `mutation($input: CreateDeploymentInput!) {
			createDeployment(input: $input) {
				deployment {
					id
					workspaceId
					description
				}
			}
		}`,
		"variables": map[string]interface{}{
			"input": map[string]interface{}{
				"workspaceId": wId1.String(),
				"description": "Test deployment for deletion",
				"file":        nil,
			},
		},
	}

	operationsJSON, err := json.Marshal(operations)
	assert.NoError(t, err)

	err = w.WriteField("operations", string(operationsJSON))
	assert.NoError(t, err)

	err = w.WriteField("map", `{"0": ["variables.input.file"]}`)
	assert.NoError(t, err)

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

	var result struct {
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

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Fatalf("Failed to create deployment: %v", result.Errors[0].Message)
	}

	return result.Data.CreateDeployment.Deployment.ID
}

func createTriggerForDeployment(t *testing.T, e *httpexpect.Expect, deploymentID string) string {
	t.Helper()

	mutation := `mutation($input: CreateTriggerInput!) {
		createTrigger(input: $input) {
			id
			description
			deploymentId
		}
	}`

	request := GraphQLRequest{
		Query: mutation,
		Variables: map[string]interface{}{
			"input": map[string]interface{}{
				"workspaceId":  wId1.String(),
				"deploymentId": deploymentID,
				"description":  "Test trigger",
				"timeDriverInput": map[string]interface{}{
					"interval": "EVERY_DAY",
				},
				"variables": map[string]interface{}{
					"TRIGGER_VAR_1": "trigger_value_1",
					"TRIGGER_VAR_2": "trigger_value_2",
				},
			},
		},
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
				ID           string `json:"id"`
				Description  string `json:"description"`
				DeploymentID string `json:"deploymentId"`
			} `json:"createTrigger"`
		} `json:"data"`
		Errors []struct {
			Message string `json:"message"`
		} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Fatalf("Failed to create trigger: %v", result.Errors[0].Message)
	}

	assert.Equal(t, deploymentID, result.Data.CreateTrigger.DeploymentID, "Trigger should reference the deployment")

	return result.Data.CreateTrigger.ID
}
