package auth

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strings"

	"github.com/reearth/reearthx/appx"
)

type TokenVerifyRequest struct {
	Token string `json:"token"`
}

type TokenVerifyResponse struct {
	Authorized bool `json:"authorized"`
}

type AuthHandler struct {
	authConfig []appx.JWTProvider
}

func NewAuthHandler(authConfig []appx.JWTProvider) *AuthHandler {
	return &AuthHandler{
		authConfig: authConfig,
	}
}

func (h *AuthHandler) VerifyToken(ctx context.Context, token string) (bool, error) {
	if !strings.HasPrefix(token, "Bearer ") {
		token = fmt.Sprintf("Bearer %s", token)
	}
	token = strings.TrimPrefix(token, "Bearer ")

	validator, err := appx.NewJWTMultipleValidator(h.authConfig)
	if err != nil {
		log.Printf("failed to initialize validator: %v", err)
		return false, fmt.Errorf("failed to initialize validator: %v", err)
	}

	_, err = validator.ValidateToken(ctx, token)
	if err != nil {
		log.Printf("failed to validate token %s: %v", token, err)
		return false, fmt.Errorf("invalid token: %v", err)
	}

	return true, nil
}

func (h *AuthHandler) VerifyAPI(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req TokenVerifyRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, fmt.Sprintf("Failed to decode request: %v", err), http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	authorized, err := h.VerifyToken(r.Context(), req.Token)
	if err != nil {
		http.Error(w, fmt.Sprintf("Token verification failed: %v", err), http.StatusUnauthorized)
		return
	}

	resp := TokenVerifyResponse{
		Authorized: authorized,
	}

	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(resp); err != nil {
		http.Error(w, fmt.Sprintf("Failed to encode response: %v", err), http.StatusInternalServerError)
		return
	}
}

type Server struct {
	handler *AuthHandler
}

func NewServer(jwtProviders []appx.JWTProvider) *Server {
	handler := NewAuthHandler(jwtProviders)
	return &Server{
		handler: handler,
	}
}

func (s *Server) Stop() {
	// HTTP服务器不需要明确关闭
}

func (s *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	path := r.URL.Path
	switch {
	case path == "/auth/verify":
		s.handler.VerifyAPI(w, r)
	default:
		http.NotFound(w, r)
	}
}
