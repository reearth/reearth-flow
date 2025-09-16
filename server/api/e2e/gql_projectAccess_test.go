package e2e

import (
	"net/http"
	"strings"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app/config"
)

func fetchSharedProject(e *httpexpect.Expect, token string) (GraphQLRequest, *httpexpect.Value) {
	fetchProjectRequestBody := GraphQLRequest{
		OperationName: "FetchSharedProject",
		Query: `query FetchSharedProject($token: String!) {
			sharedProject(token: $token) {
				project {
					id
					name
				}
			}
		}`,
		Variables: map[string]any{
			"token": token,
		},
	}

	res := e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("Content-Type", "application/json").
		WithJSON(fetchProjectRequestBody).
		Expect().
		Status(http.StatusOK).
		JSON()

	return fetchProjectRequestBody, res
}

func shareProject(e *httpexpect.Expect, projectID string) (GraphQLRequest, *httpexpect.Value) {
	shareProjectRequestBody := GraphQLRequest{
		OperationName: "ShareProject",
		Query: `mutation ShareProject($input: ShareProjectInput!) {
      shareProject(input: $input) {
        projectId
        sharingUrl
      }
    }`,
		Variables: map[string]any{
			"input": map[string]any{
				"projectId": projectID,
			},
		},
	}

	res := e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(shareProjectRequestBody).
		Expect().
		Status(http.StatusOK).
		JSON()

	return shareProjectRequestBody, res
}

func unshareProject(e *httpexpect.Expect, projectID string) (GraphQLRequest, *httpexpect.Value) {
	unshareProjectRequestBody := GraphQLRequest{
		OperationName: "UnshareProject",
		Query: `mutation UnshareProject($input: UnshareProjectInput!) {
			unshareProject(input: $input) {
				projectId
			}
		}`,
		Variables: map[string]any{
			"input": map[string]any{
				"projectId": projectID,
			},
		},
	}

	res := e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(unshareProjectRequestBody).
		Expect().
		Status(http.StatusOK).
		JSON()

	return unshareProjectRequestBody, res
}

func TestProjectShareFlow(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
		Host:       "https://example.com",
		SharedPath: "shared",
	}, true, baseSeederUser, true, nil)

	// create a project
	pId := testCreateProject(t, e)

	// 1. Share the project (success)
	_, res1 := shareProject(e, pId)
	res1.Object().Value("data").Object().
		Value("shareProject").Object().
		Value("projectId").String().IsEqual(pId)
	res1.Object().Value("data").Object().
		Value("shareProject").Object().
		Value("sharingUrl").String().NotEmpty()
	sharingUrl := res1.Path("$.data.shareProject.sharingUrl").String().Raw()

	// 2. Share the project that has already been shared (failure)
	_, res2 := shareProject(e, pId)
	res2.Object().Value("errors").Array().NotEmpty()

	// 3. Fetch the shared project (success)
	token := extractTokenFromURL(sharingUrl)
	_, res3 := fetchSharedProject(e, token)
	res3.Object().Value("data").Object().
		Value("sharedProject").Object().
		Value("project").Object().
		Value("id").String().IsEqual(pId)

	// 4. Set the project to private (success)
	_, res4 := unshareProject(e, pId)
	res4.Object().Value("data").Object().
		Value("unshareProject").Object().
		Value("projectId").String().IsEqual(pId)

	// 5. Fetch the private project with a token (failure)
	_, res5 := fetchSharedProject(e, token)
	res5.Object().Value("errors").Array().NotEmpty()

	// 6. Fetch the project with an invalid token (failure)
	_, res6 := fetchSharedProject(e, "invalid-token")
	res6.Object().Value("errors").Array().NotEmpty()
}

func extractTokenFromURL(url string) string {
	return url[strings.LastIndex(url, "/")+1:]
}
