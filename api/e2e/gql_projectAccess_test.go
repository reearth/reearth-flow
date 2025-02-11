package e2e

import (
	"net/http"
	"strings"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/stretchr/testify/assert"
)

func shareProject(e *httpexpect.Expect, projectID string) (GraphQLRequest, *httpexpect.Value, string) {
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
		WithHeader("X-Reearth-Debug-User", uID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(shareProjectRequestBody).
		Expect().
		Status(http.StatusOK).
		JSON()

	sharingUrl := res.Path("$.data.shareProject.sharingUrl").String().Raw()
	return shareProjectRequestBody, res, sharingUrl
}

// func unshareProject(e *httpexpect.Expect, projectID string) (GraphQLRequest, *httpexpect.Value) {
// 	unshareProjectRequestBody := GraphQLRequest{
// 		OperationName: "UnshareProject",
// 		Query: `mutation UnshareProject($input: UnshareProjectInput!) {
//             unshareProject(input: $input) {
//                 projectId
//             }
//         }`,
// 		Variables: map[string]any{
// 			"input": map[string]any{
// 				"projectId": projectID,
// 			},
// 		},
// 	}

// 	res := e.POST("/api/graphql").
// 		WithHeader("Origin", "https://example.com").
// 		WithHeader("authorization", "Bearer test").
// 		WithHeader("X-Reearth-Debug-User", uID.String()).
// 		WithHeader("Content-Type", "application/json").
// 		WithJSON(unshareProjectRequestBody).
// 		Expect().
// 		Status(http.StatusOK).
// 		JSON()

// 	return unshareProjectRequestBody, res
// }

// func fetchSharedProject(e *httpexpect.Expect, token string) (GraphQLRequest, *httpexpect.Value) {
// 	fetchProjectRequestBody := GraphQLRequest{
// 		OperationName: "FetchSharedProject",
// 		Query: `query FetchSharedProject($token: String!) {
//             sharedProject(token: $token) {
//                 project {
//                     id
//                     name
//                 }
//             }
//         }`,
// 		Variables: map[string]any{
// 			"token": token,
// 		},
// 	}

// 	res := e.POST("/api/graphql").
// 		WithHeader("Origin", "https://example.com").
// 		WithHeader("Content-Type", "application/json").
// 		WithJSON(fetchProjectRequestBody).
// 		Expect().
// 		Status(http.StatusOK).
// 		JSON()

// 	return fetchProjectRequestBody, res
// }

func TestProjectShareFlow(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser)

	// create a project
	pId := testCreateProject(t, e)

	// 1. プロジェクトを公開（成功）
	// 上記を英語にして
	// 1. Share the project (success)
	_, res1, sharingUrl := shareProject(e, pId)
	res1.Object().Value("data").Object().
		Value("shareProject").Object().
		Value("projectId").String().IsEqual(pId)
	assert.NotEmpty(t, sharingUrl)

	// // 2. 公開済みプロジェクトを再度公開（失敗）
	// _, res2, _ := shareProject(e, prj.ID().String())
	// res2.Object().Value("errors").Array().NotEmpty()

	// // 3. 公開したプロジェクトを取得（成功）
	// token := extractTokenFromURL(sharingUrl)
	// _, res3 := fetchSharedProject(e, token)
	// res3.Object().Value("data").Object().Value("sharedProject").Object().
	// 	Value("project").Object().Value("id").String().IsEqual(prj.ID().String())

	// // 4. プロジェクトを非公開に設定
	// _, res4 := unshareProject(e, prj.ID().String())
	// res4.Object().Value("data").Object().Value("unshareProject").Object().
	// 	Value("projectId").String().IsEqual(prj.ID().String())

	// // 5. 非公開プロジェクトをトークンで取得（失敗）
	// _, res5 := fetchSharedProject(e, token)
	// res5.Object().Value("errors").Array().NotEmpty()

	// // 6. 存在しないトークンでの取得（失敗）
	// _, res6 := fetchSharedProject(e, "invalid-token")
	// res6.Object().Value("errors").Array().NotEmpty()

	// // 7. 権限のないユーザーによる公開（失敗）
	// e2 := e.Builder(func(req *httpexpect.Request) {
	// 	req.WithHeader("X-Reearth-Debug-User", uId2.String())
	// })
	// _, res7, _ := shareProject(e2, prj.ID().String())
	// res7.Object().Value("errors").Array().NotEmpty()
}

func extractTokenFromURL(url string) string {
	// URLからトークンを抽出する実装
	// 例: "https://example.com/shared/token123" → "token123"
	return url[strings.LastIndex(url, "/")+1:]
}
