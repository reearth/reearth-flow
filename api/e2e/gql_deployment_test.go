package e2e

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"io"
	"mime/multipart"
	"net/http"
	"os"
	"path/filepath"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/stretchr/testify/assert"
)

func TestCreateDeployment(t *testing.T) {
	zipFilePath, zipBuffer, err := createZipFileWithYAML()
	assert.NoError(t, err)
	defer func() {
		if err := os.Remove(zipFilePath); err != nil {
			return
		}
	}()

	metaFileContent := []byte("meta file content")
	metaFilePath, err := os.CreateTemp("", "meta-*.txt")
	assert.NoError(t, err)

	defer func() {
		if err := os.Remove(metaFilePath.Name()); err != nil {
			return
		}
	}()

	_, err = metaFilePath.Write(metaFileContent)
	assert.NoError(t, err)

	err = metaFilePath.Close()
	assert.NoError(t, err)

	body := new(bytes.Buffer)
	writer := multipart.NewWriter(body)

	metaWriter, err := writer.CreateFormFile("metaFile", filepath.Base(metaFilePath.Name()))
	assert.NoError(t, err)
	metaFileData, err := os.Open(metaFilePath.Name())
	assert.NoError(t, err)
	_, err = io.Copy(metaWriter, metaFileData)
	assert.NoError(t, err)

	err = metaFileData.Close()
	assert.NoError(t, err)

	zipWriter, err := writer.CreateFormFile("workflowsZip", filepath.Base(zipFilePath))
	assert.NoError(t, err)
	_, err = io.Copy(zipWriter, zipBuffer)
	assert.NoError(t, err)

	err = writer.Close()
	assert.NoError(t, err)

	query := `mutation($input: CreateDeploymentInput!) {
		createDeployment(input: $input) {
			deployment {
				id
				status
			}
		}
	}`
	request := GraphQLRequest{
		Query: query,
		Variables: map[string]interface{}{
			"input": map[string]interface{}{
				"workspaceId":  "workspace-id",
				"projectId":    "project-id",
				"metaFile":     "metaFile",
				"workflowsZip": "workflowsZip",
			},
		},
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser)
	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", writer.FormDataContentType()). // Use the correct content type
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
	o.Value("data").Object().Value("createDeployment").Object().Value("deployment").Object().Value("status").String().IsEqual("Created")
}

// helper

func createZipFileWithYAML() (string, *bytes.Buffer, error) {
	yaml1 := []byte("key1: value1\nkey2: value2")
	yaml2 := []byte("keyA: valueA\nkeyB: valueB")

	buffer := new(bytes.Buffer)
	zipWriter := zip.NewWriter(buffer)

	f1, err := zipWriter.Create("file1.yaml")
	if err != nil {
		return "", nil, err
	}
	_, err = f1.Write(yaml1)
	if err != nil {
		return "", nil, err
	}

	f2, err := zipWriter.Create("file2.yaml")
	if err != nil {
		return "", nil, err
	}
	_, err = f2.Write(yaml2)
	if err != nil {
		return "", nil, err
	}

	if err := zipWriter.Close(); err != nil {
		return "", nil, err
	}

	tmpFile, err := os.CreateTemp("", "test-*.zip")
	if err != nil {
		return "", nil, err
	}
	defer func() {
		if cerr := tmpFile.Close(); cerr != nil {
			err = cerr // This will overwrite the return error if closing fails
		}
	}()

	_, err = tmpFile.Write(buffer.Bytes())
	if err != nil {
		return "", nil, err
	}

	return tmpFile.Name(), buffer, nil
}
