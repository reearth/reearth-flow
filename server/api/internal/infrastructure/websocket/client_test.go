package websocket

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestClient_DeleteDocument(t *testing.T) {
	tests := []struct {
		name       string
		statusCode int
		wantErr    bool
	}{
		{
			name:       "successful deletion returns 204",
			statusCode: http.StatusNoContent,
			wantErr:    false,
		},
		{
			name:       "successful deletion returns 200",
			statusCode: http.StatusOK,
			wantErr:    false,
		},
		{
			name:       "server error returns error",
			statusCode: http.StatusInternalServerError,
			wantErr:    true,
		},
		{
			name:       "not found returns error",
			statusCode: http.StatusNotFound,
			wantErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				assert.Equal(t, http.MethodDelete, r.Method)
				assert.Equal(t, "/api/document/test-doc-id", r.URL.Path)
				w.WriteHeader(tt.statusCode)
			}))
			defer server.Close()

			client, err := NewClient(Config{ServerURL: server.URL})
			assert.NoError(t, err)

			err = client.DeleteDocument(context.Background(), "test-doc-id")
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestClient_DeleteDocument_SetsAPISecretHeader(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "my-secret", r.Header.Get("X-API-Secret"))
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()

	client, err := NewClient(Config{
		ServerURL: server.URL,
		APISecret: "my-secret",
	})
	assert.NoError(t, err)

	err = client.DeleteDocument(context.Background(), "doc-123")
	assert.NoError(t, err)
}

func TestClient_DeleteDocument_NoSecretHeader_WhenNotConfigured(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Empty(t, r.Header.Get("X-API-Secret"))
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()

	client, err := NewClient(Config{ServerURL: server.URL})
	assert.NoError(t, err)

	err = client.DeleteDocument(context.Background(), "doc-123")
	assert.NoError(t, err)
}
