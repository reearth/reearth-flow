package e2e

import (
	"encoding/json"
	"net/http"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/stretchr/testify/assert"
)

func createTestProject(t *testing.T, e *httpexpect.Expect) string {
	t.Helper()

	query := `mutation($input: CreateProjectInput!) {
		createProject(input: $input) {
			project {
				id
				name
				description
			}
		}
	}`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"workspaceId": wId1.String(),
			"name":        "Test Project for Parameters",
			"description": "Test project for parameter testing",
			"archived":    false,
		},
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	t.Log("Creating test project...")
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

	projectID := result.Data.CreateProject.Project.ID
	t.Logf("Created test project with ID: %s", projectID)

	return projectID
}

func TestDeclareParameter(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser)

	// Create a test project first
	projectID := createTestProject(t, e)
	t.Logf("Using project ID: %s for parameter tests", projectID)

	testCases := []struct {
		name         string
		paramType    string
		paramName    string
		required     bool
		value        interface{}
		index        int
		expectedCode int
		expectError  bool
		errorMessage string
	}{
		{
			name:         "successful text parameter creation",
			paramType:    "TEXT",
			paramName:    "test_param",
			required:     true,
			value:        "test value",
			index:        0,
			expectedCode: http.StatusOK,
			expectError:  false,
		},
		{
			name:         "successful number parameter creation",
			paramType:    "NUMBER",
			paramName:    "number_param",
			required:     false,
			value:        42,
			index:        1,
			expectedCode: http.StatusOK,
			expectError:  false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			t.Logf("Running test case: %s", tc.name)

			query := `
				mutation DeclareParameter($projectId: ID!, $input: DeclareParameterInput!) {
					declareParameter(projectId: $projectId, input: $input) {
						id
						name
						type
						required
						value
						index
					}
				}
			`

			variables := map[string]interface{}{
				"projectId": projectID,
				"input": map[string]interface{}{
					"name":     tc.paramName,
					"type":     tc.paramType,
					"required": tc.required,
					"value":    tc.value,
					"index":    tc.index,
				},
			}

			request := GraphQLRequest{
				Query:     query,
				Variables: variables,
			}

			jsonData, err := json.Marshal(request)
			assert.NoError(t, err)
			t.Logf("Request payload: %s", string(jsonData))

			response := e.POST("/api/graphql").
				WithHeader("authorization", "Bearer test").
				WithHeader("Content-Type", "application/json").
				WithHeader("X-Reearth-Debug-User", uId1.String()).
				WithBytes(jsonData).
				Expect().
				Status(tc.expectedCode).
				JSON().
				Object()

			// Log raw response for debugging
			t.Logf("Raw response: %v", response.Raw())

			if tc.expectError {
				errorMsg := response.Value("errors").Array().Value(0).Object().Value("message").
					String().Raw()
				t.Logf("Error message received: %s", errorMsg)
				response.Value("errors").Array().Value(0).Object().Value("message").
					String().Contains(tc.errorMessage)
			} else {
				paramData := response.Value("data").Object().Value("declareParameter").Object()
				paramData.Value("name").String().IsEqual(tc.paramName)
				paramData.Value("type").String().IsEqual(tc.paramType)
				paramData.Value("required").Boolean().IsEqual(tc.required)
				paramData.Value("index").Number().IsEqual(float64(tc.index))

				t.Logf("Successfully created parameter: %s", tc.paramName)
			}
		})
	}
}
