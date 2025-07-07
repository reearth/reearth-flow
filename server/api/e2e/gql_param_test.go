package e2e

import (
	"encoding/json"
	"net/http"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/pkg/id"
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
	}, true, baseSeederUser, true)

	// Create a test project first
	projectID := createTestProject(t, e)
	t.Logf("Using project ID: %s for parameter tests", projectID)

	testCases := []struct {
		name         string
		paramType    string
		paramName    string
		required     bool
		public       bool
		defaultValue interface{}
		index        *int
		expectedCode int
		expectError  bool
		errorMessage string
	}{
		{
			name:         "successful text parameter creation",
			paramType:    "TEXT",
			paramName:    "test_param",
			required:     true,
			public:       true,
			defaultValue: "test value",
			index:        intPtr(0),
			expectedCode: http.StatusOK,
			expectError:  false,
		},
		{
			name:         "successful number parameter creation",
			paramType:    "NUMBER",
			paramName:    "number_param",
			required:     false,
			public:       false,
			defaultValue: 42,
			index:        intPtr(1),
			expectedCode: http.StatusOK,
			expectError:  false,
		},
		{
			name:         "parameter creation without index (auto-increment)",
			paramType:    "TEXT",
			paramName:    "auto_index_param",
			required:     true,
			public:       true,
			defaultValue: "auto index",
			index:        nil,
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
						public
						defaultValue
						index
						projectId
						createdAt
						updatedAt
					}
				}
			`

			input := map[string]interface{}{
				"name":         tc.paramName,
				"type":         tc.paramType,
				"required":     tc.required,
				"public":       tc.public,
				"defaultValue": tc.defaultValue,
			}

			if tc.index != nil {
				input["index"] = *tc.index
			}

			variables := map[string]interface{}{
				"projectId": projectID,
				"input":     input,
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
				paramData.Value("public").Boolean().IsEqual(tc.public)
				paramData.Value("defaultValue").IsEqual(tc.defaultValue)
				paramData.Value("projectId").String().IsEqual(projectID)

				if tc.index != nil {
					paramData.Value("index").Number().IsEqual(float64(*tc.index))
				} else {
					// Should auto-increment to next available index
					paramData.Value("index").Number().Ge(0)
				}

				t.Logf("Successfully created parameter: %s", tc.paramName)
			}
		})
	}
}

func TestUpdateParameter(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	// Create a test project first
	projectID := createTestProject(t, e)

	// Create a parameter to update
	paramID := createTestParameter(t, e, projectID, "original_param", "TEXT", true, true, "original value")

	testCases := []struct {
		name         string
		paramID      string
		newName      string
		newType      string
		newRequired  bool
		newPublic    bool
		newDefault   interface{}
		expectedCode int
		expectError  bool
		errorMessage string
	}{
		{
			name:         "successful parameter update",
			paramID:      paramID,
			newName:      "updated_param",
			newType:      "NUMBER",
			newRequired:  false,
			newPublic:    false,
			newDefault:   123,
			expectedCode: http.StatusOK,
			expectError:  false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			query := `
				mutation UpdateParameter($paramId: ID!, $input: UpdateParameterInput!) {
					updateParameter(paramId: $paramId, input: $input) {
						id
						name
						type
						required
						public
						defaultValue
						updatedAt
					}
				}
			`

			variables := map[string]interface{}{
				"paramId": tc.paramID,
				"input": map[string]interface{}{
					"name":         tc.newName,
					"type":         tc.newType,
					"required":     tc.newRequired,
					"public":       tc.newPublic,
					"defaultValue": tc.newDefault,
				},
			}

			request := GraphQLRequest{
				Query:     query,
				Variables: variables,
			}

			jsonData, err := json.Marshal(request)
			assert.NoError(t, err)

			response := e.POST("/api/graphql").
				WithHeader("authorization", "Bearer test").
				WithHeader("Content-Type", "application/json").
				WithHeader("X-Reearth-Debug-User", uId1.String()).
				WithBytes(jsonData).
				Expect().
				Status(tc.expectedCode).
				JSON().
				Object()

			if tc.expectError {
				response.Value("errors").Array().Value(0).Object().Value("message").
					String().Contains(tc.errorMessage)
			} else {
				paramData := response.Value("data").Object().Value("updateParameter").Object()
				paramData.Value("id").String().IsEqual(tc.paramID)
				paramData.Value("name").String().IsEqual(tc.newName)
				paramData.Value("type").String().IsEqual(tc.newType)
				paramData.Value("required").Boolean().IsEqual(tc.newRequired)
				paramData.Value("public").Boolean().IsEqual(tc.newPublic)
				paramData.Value("defaultValue").IsEqual(tc.newDefault)

				t.Logf("Successfully updated parameter: %s", tc.newName)
			}
		})
	}
}

func TestUpdateParameterOrder(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	// Create a test project first
	projectID := createTestProject(t, e)

	// Create multiple parameters to test ordering
	param1ID := createTestParameter(t, e, projectID, "param1", "TEXT", true, true, "value1")
	param2ID := createTestParameter(t, e, projectID, "param2", "TEXT", true, true, "value2")
	param3ID := createTestParameter(t, e, projectID, "param3", "TEXT", true, true, "value3")

	testCases := []struct {
		name         string
		paramID      string
		newIndex     int
		expectedCode int
		expectError  bool
		errorMessage string
	}{
		{
			name:         "move parameter to different position",
			paramID:      param1ID,
			newIndex:     2,
			expectedCode: http.StatusOK,
			expectError:  false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			query := `
				mutation UpdateParameterOrder($projectId: ID!, $input: UpdateParameterOrderInput!) {
					updateParameterOrder(projectId: $projectId, input: $input) {
						id
						name
						index
					}
				}
			`

			variables := map[string]interface{}{
				"projectId": projectID,
				"input": map[string]interface{}{
					"paramId":  tc.paramID,
					"newIndex": tc.newIndex,
				},
			}

			request := GraphQLRequest{
				Query:     query,
				Variables: variables,
			}

			jsonData, err := json.Marshal(request)
			assert.NoError(t, err)

			response := e.POST("/api/graphql").
				WithHeader("authorization", "Bearer test").
				WithHeader("Content-Type", "application/json").
				WithHeader("X-Reearth-Debug-User", uId1.String()).
				WithBytes(jsonData).
				Expect().
				Status(tc.expectedCode).
				JSON().
				Object()

			if tc.expectError {
				response.Value("errors").Array().Value(0).Object().Value("message").
					String().Contains(tc.errorMessage)
			} else {
				paramsArray := response.Value("data").Object().Value("updateParameterOrder").Array()
				paramsArray.Length().IsEqual(3) // Should return all 3 parameters

				// Find the moved parameter and verify its new index
				found := false
				for i := 0; i < 3; i++ {
					param := paramsArray.Value(i).Object()
					if param.Value("id").String().Raw() == tc.paramID {
						param.Value("index").Number().IsEqual(float64(tc.newIndex))
						found = true
						break
					}
				}
				assert.True(t, found, "Moved parameter should be found in response")

				t.Logf("Successfully updated parameter order")
			}
		})
	}

	// Cleanup - remove the test parameters
	removeTestParameter(t, e, param1ID)
	removeTestParameter(t, e, param2ID)
	removeTestParameter(t, e, param3ID)
}

func TestRemoveParameter(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	// Create a test project first
	projectID := createTestProject(t, e)

	// Create a parameter to remove
	paramID := createTestParameter(t, e, projectID, "param_to_remove", "TEXT", true, true, "remove me")

	// Generate a valid but non-existent Parameter ID
	nonExistentParamID := id.NewParameterID().String()

	testCases := []struct {
		name         string
		paramID      string
		expectedCode int
		expectError  bool
		errorMessage string
	}{
		{
			name:         "successful parameter removal",
			paramID:      paramID,
			expectedCode: http.StatusOK,
			expectError:  false,
		},
		{
			name:         "remove non-existent parameter",
			paramID:      nonExistentParamID,
			expectedCode: http.StatusOK,
			expectError:  true,
			errorMessage: "not found",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			query := `
				mutation RemoveParameter($input: RemoveParameterInput!) {
					removeParameter(input: $input)
				}
			`

			variables := map[string]interface{}{
				"input": map[string]interface{}{
					"paramId": tc.paramID,
				},
			}

			request := GraphQLRequest{
				Query:     query,
				Variables: variables,
			}

			jsonData, err := json.Marshal(request)
			assert.NoError(t, err)

			response := e.POST("/api/graphql").
				WithHeader("authorization", "Bearer test").
				WithHeader("Content-Type", "application/json").
				WithHeader("X-Reearth-Debug-User", uId1.String()).
				WithBytes(jsonData).
				Expect().
				Status(tc.expectedCode).
				JSON().
				Object()

			if tc.expectError {
				response.Value("errors").Array().Value(0).Object().Value("message").
					String().Contains(tc.errorMessage)
			} else {
				response.Value("data").Object().Value("removeParameter").Boolean().IsTrue()
				t.Logf("Successfully removed parameter")
			}
		})
	}
}

func TestParametersQuery(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true)

	// Create a test project first
	projectID := createTestProject(t, e)

	// Create some test parameters
	param1ID := createTestParameter(t, e, projectID, "query_param1", "TEXT", true, true, "value1")
	param2ID := createTestParameter(t, e, projectID, "query_param2", "NUMBER", false, false, 42)

	query := `
		query GetParameters($projectId: ID!) {
			parameters(projectId: $projectId) {
				id
				name
				type
				required
				public
				defaultValue
				index
				projectId
				createdAt
				updatedAt
			}
		}
	`

	variables := map[string]interface{}{
		"projectId": projectID,
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	response := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	paramsArray := response.Value("data").Object().Value("parameters").Array()
	paramsArray.Length().Ge(2) // Should have at least our 2 test parameters

	// Verify parameters are sorted by index
	for i := 0; i < int(paramsArray.Length().Raw())-1; i++ {
		currentIndex := paramsArray.Value(i).Object().Value("index").Number().Raw()
		nextIndex := paramsArray.Value(i + 1).Object().Value("index").Number().Raw()
		assert.True(t, currentIndex <= nextIndex, "Parameters should be sorted by index")
	}

	t.Logf("Successfully queried parameters")

	// Cleanup
	removeTestParameter(t, e, param1ID)
	removeTestParameter(t, e, param2ID)
}

// Helper functions

func intPtr(i int) *int {
	return &i
}

func createTestParameter(t *testing.T, e *httpexpect.Expect, projectID, name, paramType string, required, public bool, defaultValue interface{}) string {
	t.Helper()

	query := `
		mutation DeclareParameter($projectId: ID!, $input: DeclareParameterInput!) {
			declareParameter(projectId: $projectId, input: $input) {
				id
			}
		}
	`

	variables := map[string]interface{}{
		"projectId": projectID,
		"input": map[string]interface{}{
			"name":         name,
			"type":         paramType,
			"required":     required,
			"public":       public,
			"defaultValue": defaultValue,
		},
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	response := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	paramID := response.Value("data").Object().Value("declareParameter").Object().Value("id").String().Raw()
	t.Logf("Created test parameter with ID: %s", paramID)
	return paramID
}

func removeTestParameter(t *testing.T, e *httpexpect.Expect, paramID string) {
	t.Helper()

	query := `
		mutation RemoveParameter($input: RemoveParameterInput!) {
			removeParameter(input: $input)
		}
	`

	variables := map[string]interface{}{
		"input": map[string]interface{}{
			"paramId": paramID,
		},
	}

	request := GraphQLRequest{
		Query:     query,
		Variables: variables,
	}

	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).
		Expect().
		Status(http.StatusOK)

	t.Logf("Removed test parameter with ID: %s", paramID)
}
