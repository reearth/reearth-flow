package e2e

import (
	"bytes"
	"encoding/json"
	"fmt"
	"mime/multipart"
	"net/http"
	"net/textproto"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	pkgworkspace "github.com/reearth/reearth-flow/api/pkg/workspace"
	workspacemockrepo "github.com/reearth/reearth-flow/api/pkg/workspace/mockrepo"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
)

func TestQueryAssets(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operator := factory.NewUser(func(b *pkguser.Builder) {})
	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	// Query assets for the workspace
	query := fmt.Sprintf(`query { assets(workspaceId: "%s", pagination: {page: 1, pageSize: 10}) { nodes { id fileName size contentType } totalCount } }`, wId1)
	request := GraphQLRequest{
		Query: query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()

	o.Value("data").Object().Value("assets").Object().Value("totalCount").Number().IsEqual(0)
	o.Value("data").Object().Value("assets").Object().Value("nodes").Array().IsEmpty()
}

func TestCreateAsset(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operator := factory.NewUser(func(b *pkguser.Builder) {})
	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	// Create multipart form with file upload
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	// Add operations field
	operations := fmt.Sprintf(`{
		"query": "mutation CreateAsset($file: Upload!) { createAsset(input: {workspaceId: \"%s\", file: $file}) { asset { id fileName size contentType url } } }",
		"variables": { "file": null }
	}`, wId1)
	_ = writer.WriteField("operations", operations)

	// Add map field
	_ = writer.WriteField("map", `{ "0": ["variables.file"] }`)

	// Add file field with proper content type
	h := make(textproto.MIMEHeader)
	h.Set("Content-Disposition", `form-data; name="0"; filename="test.png"`)
	h.Set("Content-Type", "image/png")
	fileWriter, err := writer.CreatePart(h)
	assert.NoError(t, err)
	_, err = fileWriter.Write([]byte("fake png content"))
	assert.NoError(t, err)

	err = writer.Close()
	assert.NoError(t, err)

	// Send request
	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithHeader("Content-Type", writer.FormDataContentType()).
		WithBytes(body.Bytes()).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	asset := o.Value("data").Object().Value("createAsset").Object().Value("asset").Object()
	asset.Value("fileName").String().IsEqual("test.png")
	asset.Value("size").Number().Gt(0)
	asset.Value("contentType").String().IsEqual("image/png")
	asset.Value("url").String().NotEmpty()
}

func TestDeleteAsset(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operator := factory.NewUser(func(b *pkguser.Builder) {})
	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).Times(3)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	// Create an asset first
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	operations := fmt.Sprintf(`{
		"query": "mutation CreateAsset($file: Upload!) { createAsset(input: {workspaceId: \"%s\", file: $file}) { asset { id url } } }",
		"variables": { "file": null }
	}`, wId1)
	_ = writer.WriteField("operations", operations)
	_ = writer.WriteField("map", `{ "0": ["variables.file"] }`)

	h := make(textproto.MIMEHeader)
	h.Set("Content-Disposition", `form-data; name="0"; filename="test.png"`)
	h.Set("Content-Type", "image/png")
	fileWriter, err := writer.CreatePart(h)
	assert.NoError(t, err)
	_, err = fileWriter.Write([]byte("fake png content"))
	assert.NoError(t, err)
	err = writer.Close()
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithHeader("Content-Type", writer.FormDataContentType()).
		WithBytes(body.Bytes()).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	assetID := o.Value("data").Object().Value("createAsset").Object().Value("asset").Object().Value("id").String().Raw()

	// Delete the asset
	deleteQuery := fmt.Sprintf(`mutation { deleteAsset(input: {assetId: "%s"}) { assetId } }`, assetID)
	request := GraphQLRequest{
		Query: deleteQuery,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o = e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()

	// The deletion should be successful now
	o.Value("data").Object().Value("deleteAsset").Object().Value("assetId").String().IsEqual(assetID)

	// Verify asset is removed by trying to query it again
	queryAssets := fmt.Sprintf(`query { assets(workspaceId: "%s", pagination: {page: 1, pageSize: 10}) { nodes { id } totalCount } }`, wId1)
	request = GraphQLRequest{
		Query: queryAssets,
	}
	jsonData, err = json.Marshal(request)
	assert.NoError(t, err)

	o = e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()

	// Should have 0 assets after deletion
	o.Value("data").Object().Value("assets").Object().Value("totalCount").Number().IsEqual(0)
}

func TestAssetSorting(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operator := factory.NewUser(func(b *pkguser.Builder) {})
	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).Times(4)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	// Create multiple assets
	fileNames := []string{"b.png", "a.png", "c.png"}
	for _, fileName := range fileNames {
		body := &bytes.Buffer{}
		writer := multipart.NewWriter(body)

		operations := fmt.Sprintf(`{
			"query": "mutation CreateAsset($file: Upload!) { createAsset(input: {workspaceId: \"%s\", file: $file}) { asset { id fileName } } }",
			"variables": { "file": null }
		}`, wId1)
		_ = writer.WriteField("operations", operations)
		_ = writer.WriteField("map", `{ "0": ["variables.file"] }`)

		h := make(textproto.MIMEHeader)
		h.Set("Content-Disposition", fmt.Sprintf(`form-data; name="0"; filename="%s"`, fileName))
		h.Set("Content-Type", "image/png")
		fileWriter, err := writer.CreatePart(h)
		assert.NoError(t, err)
		_, err = fileWriter.Write([]byte("fake content"))
		assert.NoError(t, err)
		err = writer.Close()
		assert.NoError(t, err)

		e.POST("/api/graphql").
			WithHeader("authorization", "Bearer test").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithHeader("Content-Type", writer.FormDataContentType()).
			WithBytes(body.Bytes()).
			Expect().
			Status(http.StatusOK)
	}

	// Query with NAME sort
	query := fmt.Sprintf(`query { assets(workspaceId: "%s", sort: NAME, pagination: {page: 1, pageSize: 10}) { nodes { fileName } } }`, wId1)
	request := GraphQLRequest{
		Query: query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()

	nodes := o.Value("data").Object().Value("assets").Object().Value("nodes").Array()
	nodes.Length().IsEqual(3)
	// Files should be sorted by name (note that name defaults to fileName)
	nodes.Value(0).Object().Value("fileName").String().IsEqual("a.png")
	nodes.Value(1).Object().Value("fileName").String().IsEqual("b.png")
	nodes.Value(2).Object().Value("fileName").String().IsEqual("c.png")
}

func TestAssetKeywordSearch(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operator := factory.NewUser(func(b *pkguser.Builder) {})
	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).Times(4)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	// Create assets with different names
	fileNames := []string{"document.pdf", "image.png", "data.csv"}
	for _, fileName := range fileNames {
		body := &bytes.Buffer{}
		writer := multipart.NewWriter(body)

		operations := fmt.Sprintf(`{
			"query": "mutation CreateAsset($file: Upload!) { createAsset(input: {workspaceId: \"%s\", file: $file}) { asset { id } } }",
			"variables": { "file": null }
		}`, wId1)
		_ = writer.WriteField("operations", operations)
		_ = writer.WriteField("map", `{ "0": ["variables.file"] }`)

		h := make(textproto.MIMEHeader)
		h.Set("Content-Disposition", fmt.Sprintf(`form-data; name="0"; filename="%s"`, fileName))
		h.Set("Content-Type", "image/png")
		fileWriter, err := writer.CreatePart(h)
		assert.NoError(t, err)
		_, err = fileWriter.Write([]byte("fake content"))
		assert.NoError(t, err)
		err = writer.Close()
		assert.NoError(t, err)

		e.POST("/api/graphql").
			WithHeader("authorization", "Bearer test").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithHeader("Content-Type", writer.FormDataContentType()).
			WithBytes(body.Bytes()).
			Expect().
			Status(http.StatusOK)
	}

	// Search with keyword "image"
	query := fmt.Sprintf(`query { assets(workspaceId: "%s", keyword: "image", pagination: {page: 1, pageSize: 10}) { nodes { fileName } totalCount } }`, wId1)
	request := GraphQLRequest{
		Query: query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()

	o.Value("data").Object().Value("assets").Object().Value("totalCount").Number().IsEqual(1)
	nodes := o.Value("data").Object().Value("assets").Object().Value("nodes").Array()
	nodes.Length().IsEqual(1)
	nodes.Value(0).Object().Value("fileName").String().IsEqual("image.png")
}

func TestWorkspaceAssetsQuery(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operator := factory.NewUser(func(b *pkguser.Builder) {})
	w := factory.NewWorkspace(func(b *pkgworkspace.Builder) {})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)

	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).Times(1)
	mockWorkspaceRepo.EXPECT().FindByIDs(gomock.Any(), gomock.Any()).Return(pkgworkspace.List{w}, nil)

	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	// Query workspace with assets field - should return empty
	query := fmt.Sprintf(`query { node(id: "%s", type: WORKSPACE) { ... on Workspace { id assets(pagination: {page: 1, pageSize: 10}) { nodes { id } totalCount } } } }`, wId1)
	request := GraphQLRequest{
		Query: query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()

	// Assets on workspace should return empty (backward compatibility)
	o.Value("data").Object().Value("node").Object().Value("assets").Object().Value("totalCount").Number().IsEqual(0)
	o.Value("data").Object().Value("node").Object().Value("assets").Object().Value("nodes").Array().IsEmpty()
}

func TestCreateAssetUpload_Simple(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operator := factory.NewUser(func(b *pkguser.Builder) {})
	w := factory.NewWorkspace(func(b *pkgworkspace.Builder) {})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)

	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).Times(1)
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), gomock.Any()).Return(w, nil)

	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	// GraphQL: createAssetUpload
	req := GraphQLRequest{
		Query: `mutation ($input: CreateAssetUploadInput!) {
		createAssetUpload(input: $input) {
			token
			url
			contentType
			contentLength
			contentEncoding
			next
		}
    }`,
		Variables: map[string]any{
			"input": map[string]any{
				"workspaceId":   wId1,
				"filename":      "sample.png",
				"contentLength": 12345,
			},
		},
	}
	jsonData, err := json.Marshal(req)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()

	res := o.Value("data").Object().Value("createAssetUpload").Object()
	res.Value("token").String().NotEmpty()
	res.Value("url").String().NotEmpty()
	res.Value("contentType").String().IsEqual("image/png")
	res.Value("contentLength").Number().IsEqual(12345)
	res.Value("contentEncoding").IsNull()
	res.Value("next").IsNull()
}
