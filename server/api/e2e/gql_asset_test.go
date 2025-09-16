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
	"github.com/stretchr/testify/assert"
)

func TestQueryAssets(t *testing.T) {
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true, nil)

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
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true, nil)

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
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true, nil)

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
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true, nil)

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
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true, nil)

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
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true, nil)

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
