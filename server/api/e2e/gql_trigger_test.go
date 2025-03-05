package e2e

import (
	"bytes"
	"encoding/json"
	"mime/multipart"
	"net/http"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/stretchr/testify/assert"
)

func TestCreateTimeDrivenTrigger(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	deploymentId := createTestDeployment(t, e)
	assert.NotEmpty(t, deploymentId)

	createTimeDrivenTrigger(t, e, deploymentId)
}

func createTestDeployment(t *testing.T, e *httpexpect.Expect) string {
	t.Helper()

	var b bytes.Buffer
	w := multipart.NewWriter(&b)

	operations := map[string]interface{}{
		"query": `
			mutation($input: CreateDeploymentInput!) {
				createDeployment(input: $input) {
					deployment {
						id
						workspaceId
						description
					}
				}
			}
		`,
		"variables": map[string]interface{}{
			"input": map[string]interface{}{
				"workspaceId": wId1.String(),
				"description": "Test deployment",
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

	t.Log("Creating test deployment...")
	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
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
			Message string   `json:"message"`
			Path    []string `json:"path"`
		} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Logf("GraphQL Errors: %+v", result.Errors)
		t.FailNow()
	}

	deploymentID := result.Data.CreateDeployment.Deployment.ID
	t.Logf("Created test deployment with ID: %s", deploymentID)

	return deploymentID
}

func createTimeDrivenTrigger(t *testing.T, e *httpexpect.Expect, deploymentId string) {
	t.Helper()

	query := `mutation($input: CreateTriggerInput!) {
        createTrigger(input: $input) {
            id
            workspaceId
            deploymentId
            description
            eventSource
            timeInterval
        }
    }`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"workspaceId":  wId1.String(),
			"deploymentId": deploymentId,
			"description":  "Daily scheduled trigger",
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

	t.Log("Creating time-driven trigger...")
	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CreateTrigger struct {
				ID           string `json:"id"`
				WorkspaceID  string `json:"workspaceId"`
				DeploymentID string `json:"deploymentId"`
				Description  string `json:"description"`
				EventSource  string `json:"eventSource"`
				TimeInterval string `json:"timeInterval"`
			} `json:"createTrigger"`
		} `json:"data"`
		Errors []struct {
			Message string   `json:"message"`
			Path    []string `json:"path"`
		} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Logf("GraphQL Errors: %+v", result.Errors)
		t.FailNow()
	}

	trigger := result.Data.CreateTrigger
	assert.NotEmpty(t, trigger.ID)
	assert.Equal(t, wId1.String(), trigger.WorkspaceID)
	assert.Equal(t, deploymentId, trigger.DeploymentID)
	assert.Equal(t, "Daily scheduled trigger", trigger.Description)
	assert.Equal(t, "TIME_DRIVEN", trigger.EventSource)
	assert.Equal(t, "EVERY_DAY", trigger.TimeInterval)

	t.Logf("Created trigger with ID: %s", trigger.ID)
}

func TestUpdateTrigger(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	deploymentId := createTestDeployment(t, e)
	query := `mutation($input: CreateTriggerInput!) {
		createTrigger(input: $input) {
			id
			deploymentId
		}
	}`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"workspaceId":  wId1.String(),
			"deploymentId": deploymentId,
			"description":  "Initial trigger",
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
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var createResult struct {
		Data struct {
			CreateTrigger struct {
				ID string `json:"id"`
			} `json:"createTrigger"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &createResult)
	assert.NoError(t, err)

	triggerId := createResult.Data.CreateTrigger.ID

	updateQuery := `mutation($input: UpdateTriggerInput!) {
		updateTrigger(input: $input) {
			id
			description
			eventSource
			timeInterval
		}
	}`

	updateVariables := map[string]interface{}{
		"input": map[string]interface{}{
			"triggerId":   triggerId,
			"description": "Updated trigger",
			"timeDriverInput": map[string]interface{}{
				"interval": "EVERY_HOUR",
			},
		},
	}

	updateRequest := GraphQLRequest{
		Query:     updateQuery,
		Variables: updateVariables,
	}

	updateJsonData, err := json.Marshal(updateRequest)
	assert.NoError(t, err)

	updateResp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(updateJsonData).
		Expect().Status(http.StatusOK)

	var updateResult struct {
		Data struct {
			UpdateTrigger struct {
				ID           string `json:"id"`
				Description  string `json:"description"`
				EventSource  string `json:"eventSource"`
				TimeInterval string `json:"timeInterval"`
			} `json:"updateTrigger"`
		} `json:"data"`
	}

	err = json.Unmarshal([]byte(updateResp.Body().Raw()), &updateResult)
	assert.NoError(t, err)

	trigger := updateResult.Data.UpdateTrigger
	assert.Equal(t, triggerId, trigger.ID)
	assert.Equal(t, "Updated trigger", trigger.Description)
	assert.Equal(t, "TIME_DRIVEN", trigger.EventSource)
	assert.Equal(t, "EVERY_HOUR", trigger.TimeInterval)
}

func TestCreateAPIDrivenTrigger(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	deploymentId := createTestDeployment(t, e)
	assert.NotEmpty(t, deploymentId)

	query := `mutation($input: CreateTriggerInput!) {
		createTrigger(input: $input) {
			id
			workspaceId
			deploymentId
			description
			eventSource
			authToken
		}
	}`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"workspaceId":  wId1.String(),
			"deploymentId": deploymentId,
			"description":  "API trigger test",
			"apiDriverInput": map[string]interface{}{
				"token": "test-api-token",
			},
		},
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	t.Log("Creating API-driven trigger...")
	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			CreateTrigger struct {
				ID           string `json:"id"`
				WorkspaceID  string `json:"workspaceId"`
				DeploymentID string `json:"deploymentId"`
				Description  string `json:"description"`
				EventSource  string `json:"eventSource"`
				AuthToken    string `json:"authToken"`
			} `json:"createTrigger"`
		} `json:"data"`
		Errors []struct {
			Message string   `json:"message"`
			Path    []string `json:"path"`
		} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Logf("GraphQL Errors: %+v", result.Errors)
		t.FailNow()
	}

	trigger := result.Data.CreateTrigger
	assert.NotEmpty(t, trigger.ID)
	assert.Equal(t, wId1.String(), trigger.WorkspaceID)
	assert.Equal(t, deploymentId, trigger.DeploymentID)
	assert.Equal(t, "API trigger test", trigger.Description)
	assert.Equal(t, "API_DRIVEN", trigger.EventSource)
	assert.NotEmpty(t, trigger.AuthToken)

	t.Logf("Created API trigger with ID: %s", trigger.ID)
}
