package e2e

import (
	"bytes"
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
	yamlContent1 := `
key1: value1
key2: value2
`

	// Create a temporary YAML file
	file, err := createTempYAML(yamlContent1)
	assert.NoError(t, err)

	defer func() {
		if err := os.Remove(file); err != nil {
			return
		}
	}()

	query := `mutation($input: CreateDeploymentInput!) {
		createDeployment(input: $input) {
			deployment {
				id
				status
			}
		}
	}`

	yamlFile, err := os.Open(file)
	assert.NoError(t, err)
	defer func() {
		if err := yamlFile.Close(); err != nil {
			return
		}
	}()

	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	part, err := writer.CreateFormFile("workflowYaml", filepath.Base(file))
	assert.NoError(t, err)

	_, err = io.Copy(part, yamlFile)
	assert.NoError(t, err)

	err = writer.WriteField("query", query)
	assert.NoError(t, err)

	variables := `{
		"input": {
			"workspaceId": "workspace-id",
			"projectId": "project-id",
			"metaFile": "metaFile"
		}
	}`
	err = writer.WriteField("variables", variables)
	assert.NoError(t, err)

	err = writer.Close()
	assert.NoError(t, err)

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", uId1.String()).
		WithHeader("Content-Type", writer.FormDataContentType()).
		WithBytes(body.Bytes()).Expect().Status(http.StatusOK).JSON().Object()

	o.Value("data").Object().Value("createDeployment").Object().Value("deployment").Object().Value("status").String().IsEqual("Created")
}

// helper

func createTempYAML(content string) (string, error) {
	tmpFile, err := os.CreateTemp("", "*.yaml")
	if err != nil {
		return "", err
	}

	defer func() {
		if err := tmpFile.Close(); err != nil {
			return
		}
	}()

	_, err = tmpFile.Write([]byte(content))
	if err != nil {
		return "", err
	}

	return tmpFile.Name(), nil
}
