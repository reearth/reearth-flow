package e2e

import (
	"net/http"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
)

func TestParameterOperations(t *testing.T) {
	e := StartServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeeder)

	// Test Declare Parameter
	declareParamBody := GraphQLRequest{
		OperationName: "DeclareParameter",
		Query: `
        mutation DeclareParameter($projectId: ID!, $input: DeclareParameterInput!) {
            declareParameter(projectId: $projectId, input: $input) {
                id
                name
                type
                required
                value
                index
                projectId
                createdAt
                updatedAt
            }
        }`,
		Variables: map[string]interface{}{
			"projectId": pID.String(),
			"input": map[string]interface{}{
				"name":     "test-param",
				"type":     "TEXT",
				"required": true,
				"value":    "initial value",
				"index":    0,
			},
		},
	}

	response := e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(declareParamBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("data").Object().
		Value("declareParameter").Object()

	paramID := response.Value("id").String().Raw()

	// Test Update Parameter Value
	updateValueBody := GraphQLRequest{
		OperationName: "UpdateParameterValue",
		Query: `
        mutation UpdateParameterValue($paramId: ID!, $input: UpdateParameterValueInput!) {
            updateParameterValue(paramId: $paramId, input: $input) {
                id
                value
                updatedAt
            }
        }`,
		Variables: map[string]interface{}{
			"paramId": paramID,
			"input": map[string]interface{}{
				"value": "updated value",
			},
		},
	}

	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(updateValueBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("data").Object().
		Value("updateParameterValue").Object().
		Value("value").IsEqual("updated value")

	// Test Update Parameter Order
	updateOrderBody := GraphQLRequest{
		OperationName: "UpdateParameterOrder",
		Query: `
        mutation UpdateParameterOrder($projectId: ID!, $input: UpdateParameterOrderInput!) {
            updateParameterOrder(projectId: $projectId, input: $input) {
                id
                index
            }
        }`,
		Variables: map[string]interface{}{
			"projectId": pID.String(),
			"input": map[string]interface{}{
				"paramId":  paramID,
				"newIndex": 1,
			},
		},
	}

	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(updateOrderBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("data").Object().
		Value("updateParameterOrder").Array().Value(0).Object().
		Value("index").IsEqual(1)

	// Test Remove Parameter
	removeParamBody := GraphQLRequest{
		OperationName: "RemoveParameter",
		Query: `
        mutation RemoveParameter($input: RemoveParameterInput!) {
            removeParameter(input: $input)
        }`,
		Variables: map[string]interface{}{
			"input": map[string]interface{}{
				"paramId": paramID,
			},
		},
	}

	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(removeParamBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("data").Object().
		Value("removeParameter").Boolean().IsTrue()

	// Verify parameter was removed by trying to update it (should fail)
	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(updateValueBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("errors").Array().NotEmpty()
}

func TestParameterValidations(t *testing.T) {
	e := StartServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeeder)

	// Test invalid parameter type
	invalidTypeBody := GraphQLRequest{
		OperationName: "DeclareParameter",
		Query: `
        mutation DeclareParameter($projectId: ID!, $input: DeclareParameterInput!) {
            declareParameter(projectId: $projectId, input: $input) {
                id
                type
            }
        }`,
		Variables: map[string]interface{}{
			"projectId": pID.String(),
			"input": map[string]interface{}{
				"name":     "test-param",
				"type":     "INVALID_TYPE",
				"required": true,
				"value":    "test",
			},
		},
	}

	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(invalidTypeBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("errors").Array().NotEmpty()

	// Test invalid project ID
	invalidProjectBody := GraphQLRequest{
		OperationName: "DeclareParameter",
		Query: `
        mutation DeclareParameter($projectId: ID!, $input: DeclareParameterInput!) {
            declareParameter(projectId: $projectId, input: $input) {
                id
            }
        }`,
		Variables: map[string]interface{}{
			"projectId": "invalid-project-id",
			"input": map[string]interface{}{
				"name":     "test-param",
				"type":     "TEXT",
				"required": true,
				"value":    "test",
			},
		},
	}

	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(invalidProjectBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("errors").Array().NotEmpty()
}
