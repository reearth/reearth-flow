package e2e

import (
	"net/http"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
)

func TestCreateProject(t *testing.T) {
	e := StartServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	},
		true, baseSeeder)

	requestBody := GraphQLRequest{
		OperationName: "CreateProject",
		Query:         "mutation CreateProject($workspaceId: ID!, $name: String!, $description: String!) {\n createProject(\n input: {workspaceId: $workspaceId, name: $name, description: $description}\n ) {\n project {\n id\n name\n description\n __typename\n }\n __typename\n }\n}",
		Variables: map[string]any{
			"name":        "test",
			"description": "abc",
			"workspaceId": wID.String(),
		},
	}

	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		// WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(requestBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("data").Object().
		Value("createProject").Object().
		Value("project").Object().
		HasValue("name", "test").
		HasValue("description", "abc")
}

func TestRunProject(t *testing.T) {
	e := StartServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	},
		true, baseSeeder)

	requestBody := GraphQLRequest{
		OperationName: "RunProject",
		Query:         "mutation RunProject($projectId:ID!, $workspaceId:ID!, $workflows:[InputWorkflow!]!){\n runProject(\n input: {projectId: $projectId, workspaceId:$workspaceId, workflows: $workflows}\n ){\n projectId\n started\n }\n}",
		Variables: map[string]interface{}{
			"projectId":   "01j2g1gj3vjpwaz845es2a4rcm",
			"workspaceId": "01j2g1fkbme3gtd7tvv7dgt4c1",
			"workflows": []interface{}{
				map[string]interface{}{
					"id":   "01j1c1nstedb08bj97y8b7dz6w",
					"name": "Cool",
					"nodes": []interface{}{
						map[string]interface{}{
							"id":   "alskdfj",
							"type": "READER",
							"data": map[string]interface{}{
								"name":     "alsdkfjl",
								"actionId": "someAction",
								"params": []interface{}{
									map[string]interface{}{
										"id":    "paramId",
										"name":  "My Param",
										"type":  "STRING",
										"value": "my value is here",
									},
								},
							},
						},
					},
					"edges":  []interface{}{},
					"isMain": true,
				},
			},
		},
	}

	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		// WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(requestBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("data").Object().
		Value("createProject").Object().
		Value("project").Object().
		HasValue("name", "test").
		HasValue("description", "abc")
}
