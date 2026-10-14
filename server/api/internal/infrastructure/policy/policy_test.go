package policy

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestPermissive(t *testing.T) {
	res, err := NewPermissive().CheckPolicy(context.Background(), gateway.PolicyCheckRequest{
		CheckType: gateway.PolicyCheckPrivateProjectCreation,
		Value:     1,
	})
	require.NoError(t, err)
	assert.True(t, res.Allowed)
}

func TestHTTP(t *testing.T) {
	wid := accountsid.NewWorkspaceID()
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "Bearer secret", r.Header.Get("Authorization"))

		var got map[string]any
		require.NoError(t, json.NewDecoder(r.Body).Decode(&got))
		assert.Equal(t, map[string]any{
			"workspace_id": wid.String(),
			"check_type":   "general_private_project_creation",
			"value":        float64(1),
		}, got)

		// The shape the policy service sends: snake_case keys.
		_, _ = w.Write([]byte(`{"allowed":false,"check_type":"general_private_project_creation","current_limit":"0","message":"upgrade","value":1}`))
	}))
	defer srv.Close()

	res, err := NewHTTP(srv.URL, "secret", time.Second).CheckPolicy(context.Background(), gateway.PolicyCheckRequest{
		CheckType:   gateway.PolicyCheckPrivateProjectCreation,
		WorkspaceID: wid,
		Value:       1,
	})
	require.NoError(t, err)
	assert.Equal(t, &gateway.PolicyCheckResponse{
		Allowed:      false,
		CheckType:    gateway.PolicyCheckPrivateProjectCreation,
		CurrentLimit: "0",
		Message:      "upgrade",
		Value:        1,
	}, res)
}

func TestHTTP_ErrorStatus(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
	}))
	defer srv.Close()

	_, err := NewHTTP(srv.URL, "", time.Second).CheckPolicy(context.Background(), gateway.PolicyCheckRequest{})
	assert.Error(t, err)
}
