package e2e

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/pkg/id"
	ws "github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/stretchr/testify/assert"
)

var (
	docUId = user.NewID()
	docPId = id.NewProjectID()

	rollbackResults = map[int]*ws.Document{
		1: {
			ID:        docPId.String(),
			Updates:   []int{1},
			Version:   1,
			Timestamp: time.Now(),
		},
		2: {
			ID:        docPId.String(),
			Updates:   []int{1, 2, 3},
			Version:   2,
			Timestamp: time.Now(),
		},
	}
)

func documentTestInterceptor(next http.Handler) http.Handler {
	doc := &ws.Document{
		ID:        docPId.String(),
		Updates:   []int{1, 2, 3, 4, 5},
		Version:   3,
		Timestamp: time.Date(2023, 2, 1, 10, 0, 0, 0, time.UTC),
	}

	history := []*ws.History{
		{
			Updates:   []int{1},
			Version:   1,
			Timestamp: time.Date(2023, 1, 15, 9, 0, 0, 0, time.UTC),
		},
		{
			Updates:   []int{1, 2, 3},
			Version:   2,
			Timestamp: time.Date(2023, 1, 20, 14, 0, 0, 0, time.UTC),
		},
		{
			Updates:   []int{1, 2, 3, 4, 5},
			Version:   3,
			Timestamp: time.Date(2023, 2, 1, 10, 0, 0, 0, time.UTC),
		},
	}

	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if (r.URL.Path == "/api/graphql" || r.URL.Path == "api/graphql") && r.Method == "POST" {
			var gqlRequest struct {
				Query     string         `json:"query"`
				Variables map[string]any `json:"variables"`
			}

			bodyBytes, err := io.ReadAll(r.Body)
			if err != nil {
				http.Error(w, "Failed to read request body", http.StatusBadRequest)
				return
			}
			if err := r.Body.Close(); err != nil {
				http.Error(w, "Failed to close request body: "+err.Error(), http.StatusInternalServerError)
				return
			}

			r.Body = io.NopCloser(strings.NewReader(string(bodyBytes)))

			if err := json.Unmarshal(bodyBytes, &gqlRequest); err != nil {
				http.Error(w, "Invalid request body: "+err.Error(), http.StatusBadRequest)
				return
			}

			if isLatestProjectSnapshotQuery(gqlRequest.Query) {
				projectID, ok := getProjectIDFromVariables(gqlRequest.Variables)
				if !ok || projectID != docPId.String() {
					http.Error(w, "Invalid project ID", http.StatusBadRequest)
					return
				}

				resp := map[string]any{
					"data": map[string]any{
						"latestProjectSnapshot": map[string]any{
							"id":        doc.ID,
							"updates":   doc.Updates,
							"version":   doc.Version,
							"timestamp": doc.Timestamp,
						},
					},
				}
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(http.StatusOK)
				if err := json.NewEncoder(w).Encode(resp); err != nil {
					http.Error(w, "Failed to encode response: "+err.Error(), http.StatusInternalServerError)
					return
				}
				return

			} else if isProjectHistoryQuery(gqlRequest.Query) {
				projectID, ok := getProjectIDFromVariables(gqlRequest.Variables)
				if !ok || projectID != docPId.String() {
					http.Error(w, "Invalid project ID", http.StatusBadRequest)
					return
				}

				historyResp := make([]map[string]any, len(history))
				for i, h := range history {
					historyResp[i] = map[string]any{
						"updates":   h.Updates,
						"version":   h.Version,
						"timestamp": h.Timestamp,
					}
				}

				resp := map[string]any{
					"data": map[string]any{
						"projectHistory": historyResp,
					},
				}
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(http.StatusOK)
				if err := json.NewEncoder(w).Encode(resp); err != nil {
					http.Error(w, "Failed to encode response: "+err.Error(), http.StatusInternalServerError)
					return
				}
				return

			} else if isProjectHistoryMetadataQuery(gqlRequest.Query) {
				projectID, ok := getProjectIDFromVariables(gqlRequest.Variables)
				if !ok || projectID != docPId.String() {
					http.Error(w, "Invalid project ID", http.StatusBadRequest)
					return
				}

				metadataResp := make([]map[string]any, len(history))
				for i, h := range history {
					metadataResp[i] = map[string]any{
						"version":   h.Version,
						"timestamp": h.Timestamp,
					}
				}

				resp := map[string]any{
					"data": map[string]any{
						"projectHistoryMetadata": metadataResp,
					},
				}
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(http.StatusOK)
				if err := json.NewEncoder(w).Encode(resp); err != nil {
					http.Error(w, "Failed to encode response: "+err.Error(), http.StatusInternalServerError)
					return
				}
				return

			} else if isRollbackProjectMutation(gqlRequest.Query) {
				projectID, version, ok := getProjectIDAndVersionFromVariables(gqlRequest.Variables)
				if !ok || projectID != docPId.String() {
					http.Error(w, "Invalid project ID or version", http.StatusBadRequest)
					return
				}

				rollbackDoc, ok := rollbackResults[version]
				if !ok {
					http.Error(w, "Invalid version for rollback", http.StatusBadRequest)
					return
				}

				resp := map[string]any{
					"data": map[string]any{
						"rollbackProject": map[string]any{
							"id":        rollbackDoc.ID,
							"updates":   rollbackDoc.Updates,
							"version":   rollbackDoc.Version,
							"timestamp": rollbackDoc.Timestamp,
						},
					},
				}
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(http.StatusOK)
				if err := json.NewEncoder(w).Encode(resp); err != nil {
					http.Error(w, "Failed to encode response: "+err.Error(), http.StatusInternalServerError)
					return
				}
				return
			}
		}

		next.ServeHTTP(w, r)
	})
}

func isLatestProjectSnapshotQuery(query string) bool {
	return strings.Contains(query, "latestProjectSnapshot")
}

func isProjectHistoryQuery(query string) bool {
	return strings.Contains(query, "projectHistory") && !strings.Contains(query, "projectHistoryMetadata")
}

func isProjectHistoryMetadataQuery(query string) bool {
	return strings.Contains(query, "projectHistoryMetadata")
}

func isRollbackProjectMutation(query string) bool {
	return strings.Contains(query, "rollbackProject")
}

func getProjectIDFromVariables(vars map[string]any) (string, bool) {
	projectIDVar, ok := vars["projectId"]
	if !ok {
		return "", false
	}

	projectID, ok := projectIDVar.(string)
	return projectID, ok
}

func getProjectIDAndVersionFromVariables(vars map[string]any) (string, int, bool) {
	projectID, ok := getProjectIDFromVariables(vars)
	if !ok {
		return "", 0, false
	}

	versionVar, ok := vars["version"]
	if !ok {
		return "", 0, false
	}

	var version int
	switch v := versionVar.(type) {
	case int:
		version = v
	case float64:
		version = int(v)
	default:
		return "", 0, false
	}

	return projectID, version, true
}

func TestDocumentOperations(t *testing.T) {
	srv := httptest.NewServer(documentTestInterceptor(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		errMsg := fmt.Sprintf("Path not found: %s (Method: %s)", r.URL.Path, r.Method)
		_, err := w.Write([]byte(errMsg))
		if err != nil {
			t.Errorf("Failed to write response: %v", err)
		}
		t.Logf("Unhandled request: %s", errMsg)
	})))
	defer srv.Close()

	testClient := httpexpect.Default(t, srv.URL)

	testLatestProjectSnapshot(t, testClient, docPId.String())

	testProjectHistory(t, testClient, docPId.String())

	testProjectHistoryMetadata(t, testClient, docPId.String())

	testRollbackProject(t, testClient, docPId.String(), 1)
}

func testLatestProjectSnapshot(t *testing.T, e *httpexpect.Expect, projectId string) {
	query := `query($projectId: ID!) {
		latestProjectSnapshot(projectId: $projectId) {
			id
			timestamp
			updates
			version
		}
	}`

	variables := fmt.Sprintf(`{
		"projectId": "%s"
	}`, projectId)

	var variablesMap map[string]any
	err := json.Unmarshal([]byte(variables), &variablesMap)
	assert.NoError(t, err)

	request := GraphQLRequest{
		Query:     query,
		Variables: variablesMap,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", docUId.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			LatestProjectSnapshot *struct {
				ID        string    `json:"id"`
				Timestamp time.Time `json:"timestamp"`
				Updates   []int     `json:"updates"`
				Version   int       `json:"version"`
			} `json:"latestProjectSnapshot"`
		} `json:"data"`
		Errors []map[string]interface{} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Fatalf("GraphQL errors: %v", result.Errors)
	}

	snapshot := result.Data.LatestProjectSnapshot
	assert.NotNil(t, snapshot, "snapshot should not be nil")
	if snapshot != nil {
		assert.Equal(t, projectId, snapshot.ID)
		assert.Equal(t, 3, snapshot.Version)
		assert.Equal(t, []int{1, 2, 3, 4, 5}, snapshot.Updates)
		assert.WithinDuration(t, time.Date(2023, 2, 1, 10, 0, 0, 0, time.UTC), snapshot.Timestamp, time.Second)
	}
}

func testProjectHistory(t *testing.T, e *httpexpect.Expect, projectId string) {
	query := `query($projectId: ID!) {
		projectHistory(projectId: $projectId) {
			timestamp
			updates
			version
		}
	}`

	variables := fmt.Sprintf(`{
		"projectId": "%s"
	}`, projectId)

	var variablesMap map[string]any
	err := json.Unmarshal([]byte(variables), &variablesMap)
	assert.NoError(t, err)

	request := GraphQLRequest{
		Query:     query,
		Variables: variablesMap,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", docUId.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			ProjectHistory []struct {
				Timestamp time.Time `json:"timestamp"`
				Updates   []int     `json:"updates"`
				Version   int       `json:"version"`
			} `json:"projectHistory"`
		} `json:"data"`
		Errors []map[string]interface{} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Fatalf("GraphQL errors: %v", result.Errors)
	}

	history := result.Data.ProjectHistory
	assert.NotNil(t, history, "history should not be nil")
	assert.Equal(t, 3, len(history), "should return 3 history records")

	if len(history) >= 3 {
		assert.Equal(t, 1, history[0].Version)
		assert.Equal(t, []int{1}, history[0].Updates)
		assert.WithinDuration(t, time.Date(2023, 1, 15, 9, 0, 0, 0, time.UTC), history[0].Timestamp, time.Second)

		assert.Equal(t, 2, history[1].Version)
		assert.Equal(t, []int{1, 2, 3}, history[1].Updates)
		assert.WithinDuration(t, time.Date(2023, 1, 20, 14, 0, 0, 0, time.UTC), history[1].Timestamp, time.Second)

		assert.Equal(t, 3, history[2].Version)
		assert.Equal(t, []int{1, 2, 3, 4, 5}, history[2].Updates)
		assert.WithinDuration(t, time.Date(2023, 2, 1, 10, 0, 0, 0, time.UTC), history[2].Timestamp, time.Second)
	}
}

func testProjectHistoryMetadata(t *testing.T, e *httpexpect.Expect, projectId string) {
	query := `query($projectId: ID!) {
		projectHistoryMetadata(projectId: $projectId) {
			version
			timestamp
		}
	}`

	variables := fmt.Sprintf(`{
		"projectId": "%s"
	}`, projectId)

	var variablesMap map[string]any
	err := json.Unmarshal([]byte(variables), &variablesMap)
	assert.NoError(t, err)

	request := GraphQLRequest{
		Query:     query,
		Variables: variablesMap,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", docUId.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			ProjectHistoryMetadata []struct {
				Version   int       `json:"version"`
				Timestamp time.Time `json:"timestamp"`
			} `json:"projectHistoryMetadata"`
		} `json:"data"`
		Errors []map[string]interface{} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Fatalf("GraphQL errors: %v", result.Errors)
	}

	metadata := result.Data.ProjectHistoryMetadata
	assert.NotNil(t, metadata, "metadata should not be nil")
	assert.Equal(t, 3, len(metadata), "should return 3 metadata records")

	if len(metadata) >= 3 {
		assert.Equal(t, 1, metadata[0].Version)
		assert.WithinDuration(t, time.Date(2023, 1, 15, 9, 0, 0, 0, time.UTC), metadata[0].Timestamp, time.Second)

		assert.Equal(t, 2, metadata[1].Version)
		assert.WithinDuration(t, time.Date(2023, 1, 20, 14, 0, 0, 0, time.UTC), metadata[1].Timestamp, time.Second)

		assert.Equal(t, 3, metadata[2].Version)
		assert.WithinDuration(t, time.Date(2023, 2, 1, 10, 0, 0, 0, time.UTC), metadata[2].Timestamp, time.Second)
	}
}

func testRollbackProject(t *testing.T, e *httpexpect.Expect, projectId string, version int) {
	query := `mutation($projectId: ID!, $version: Int!) {
		rollbackProject(projectId: $projectId, version: $version) {
			id
			timestamp
			updates
			version
		}
	}`

	variables := fmt.Sprintf(`{
		"projectId": "%s",
		"version": %d
	}`, projectId, version)

	var variablesMap map[string]any
	err := json.Unmarshal([]byte(variables), &variablesMap)
	assert.NoError(t, err)

	request := GraphQLRequest{
		Query:     query,
		Variables: variablesMap,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	resp := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", docUId.String()).
		WithBytes(jsonData).
		Expect().Status(http.StatusOK)

	var result struct {
		Data struct {
			RollbackProject *struct {
				ID        string    `json:"id"`
				Timestamp time.Time `json:"timestamp"`
				Updates   []int     `json:"updates"`
				Version   int       `json:"version"`
			} `json:"rollbackProject"`
		} `json:"data"`
		Errors []map[string]interface{} `json:"errors"`
	}

	err = json.Unmarshal([]byte(resp.Body().Raw()), &result)
	assert.NoError(t, err)

	if len(result.Errors) > 0 {
		t.Fatalf("GraphQL errors: %v", result.Errors)
	}

	rollback := result.Data.RollbackProject
	assert.NotNil(t, rollback, "rollback result should not be nil")
	if rollback != nil {
		assert.Equal(t, projectId, rollback.ID)
		assert.Equal(t, version, rollback.Version)
		assert.Equal(t, rollbackResults[version].Updates, rollback.Updates)
		assert.NotZero(t, rollback.Timestamp)
	}
}
