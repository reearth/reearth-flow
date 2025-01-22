package app

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/stretchr/testify/assert"
)

func TestVerifyAuth(t *testing.T) {
	tests := []struct {
		name           string
		requestBody    interface{}
		setupContext   func(context.Context) context.Context
		expectedStatus int
		expectedAuth   bool
	}{
		{
			name: "valid token with operator",
			requestBody: AuthRequest{
				Token: "valid-token",
			},
			setupContext: func(ctx context.Context) context.Context {
				uid := user.NewID()
				op := &usecase.Operator{
					AcOperator: &accountusecase.Operator{
						User: &uid,
					},
				}
				return adapter.AttachOperator(ctx, op)
			},
			expectedStatus: http.StatusOK,
			expectedAuth:   true,
		},
		{
			name: "invalid token without operator",
			requestBody: AuthRequest{
				Token: "invalid-token",
			},
			setupContext:   func(ctx context.Context) context.Context { return ctx },
			expectedStatus: http.StatusUnauthorized,
			expectedAuth:   false,
		},
		{
			name:           "empty request body",
			setupContext:   func(ctx context.Context) context.Context { return ctx },
			expectedStatus: http.StatusBadRequest,
			expectedAuth:   false,
		},
		{
			name: "empty token",
			requestBody: AuthRequest{
				Token: "",
			},
			setupContext:   func(ctx context.Context) context.Context { return ctx },
			expectedStatus: http.StatusUnauthorized,
			expectedAuth:   false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Setup
			e := echo.New()
			var body []byte
			if tt.requestBody != nil {
				body, _ = json.Marshal(tt.requestBody)
			}
			req := httptest.NewRequest(http.MethodPost, "/auth/verify", bytes.NewReader(body))
			if tt.requestBody != nil {
				req.Header.Set(echo.HeaderContentType, echo.MIMEApplicationJSON)
			}

			rec := httptest.NewRecorder()
			c := e.NewContext(req, rec)
			c.SetRequest(req.WithContext(tt.setupContext(context.Background())))

			// Execute
			h := VerifyAuth()
			err := h(c)

			// Assert
			assert.NoError(t, err)
			assert.Equal(t, tt.expectedStatus, rec.Code)

			var response map[string]bool
			err = json.Unmarshal(rec.Body.Bytes(), &response)
			assert.NoError(t, err)
			assert.Equal(t, tt.expectedAuth, response["authorized"])
		})
	}
}
