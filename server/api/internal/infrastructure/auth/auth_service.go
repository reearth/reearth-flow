package auth

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearthx/appx"
)

// TokenVerifyRequest is the websocket server's pre-upgrade check. Room is the
// websocket room ("{projectId}:{workflowId}"); when set, the caller must have
// access to that project, either as a workspace member (Token) or through the
// project's public share token (ShareToken). A request without Room is the
// legacy token-only check the Rust server still sends.
type TokenVerifyRequest struct {
	Token      string `json:"token"`
	Room       string `json:"room,omitempty"`
	ShareToken string `json:"shareToken,omitempty"`
}

type TokenVerifyResponse struct {
	Authorized bool `json:"authorized"`
	ReadOnly   bool `json:"readOnly,omitempty"`
}

// ProjectAccessChecker decides access to a project's live document.
type ProjectAccessChecker interface {
	// UserAccess reports whether the user whose JWT is in ctx may read and edit
	// the project's document.
	UserAccess(ctx context.Context, projectID string) (read, edit bool, err error)
	// SharedAccess reports whether shareToken is the project's live public
	// share token.
	SharedAccess(ctx context.Context, shareToken, projectID string) (bool, error)
}

type AuthHandler struct {
	projects   ProjectAccessChecker
	validate   func(ctx context.Context, token string) error
	authConfig []appx.JWTProvider
}

func NewAuthHandler(authConfig []appx.JWTProvider, projects ProjectAccessChecker) *AuthHandler {
	h := &AuthHandler{
		authConfig: authConfig,
		projects:   projects,
	}
	h.validate = h.validateJWT
	return h
}

func bareToken(token string) string {
	return strings.TrimPrefix(token, "Bearer ")
}

func (h *AuthHandler) validateJWT(ctx context.Context, token string) error {
	validator, err := appx.NewJWTMultipleValidator(h.authConfig)
	if err != nil {
		log.Printf("failed to initialize validator: %v", err)
		return fmt.Errorf("failed to initialize validator: %v", err)
	}
	if _, err := validator.ValidateToken(ctx, token); err != nil {
		log.Printf("failed to validate token: %v", err)
		return fmt.Errorf("invalid token: %v", err)
	}
	return nil
}

func (h *AuthHandler) VerifyToken(ctx context.Context, token string) (bool, error) {
	if err := h.validate(ctx, bareToken(token)); err != nil {
		return false, err
	}
	return true, nil
}

// roomProject returns the project id a websocket room belongs to.
func roomProject(room string) string {
	projectID, _, _ := strings.Cut(room, ":")
	return projectID
}

// verifyRoom returns the HTTP status and verdict for a room-scoped request.
func (h *AuthHandler) verifyRoom(ctx context.Context, req TokenVerifyRequest) (int, TokenVerifyResponse) {
	denied := TokenVerifyResponse{}
	if h.projects == nil {
		return http.StatusForbidden, denied
	}
	projectID := roomProject(req.Room)

	switch {
	case req.Token != "":
		token := bareToken(req.Token)
		if err := h.validate(ctx, token); err != nil {
			return http.StatusUnauthorized, denied
		}
		read, edit, err := h.projects.UserAccess(adapter.AttachJWT(ctx, token), projectID)
		if err != nil {
			log.Printf("websocket project access check failed for project %s: %v", projectID, err)
			return http.StatusInternalServerError, denied
		}
		if !read {
			return http.StatusForbidden, denied
		}
		return http.StatusOK, TokenVerifyResponse{Authorized: true, ReadOnly: !edit}

	case req.ShareToken != "":
		ok, err := h.projects.SharedAccess(ctx, req.ShareToken, projectID)
		if err != nil {
			log.Printf("websocket shared access check failed for project %s: %v", projectID, err)
			return http.StatusInternalServerError, denied
		}
		if !ok {
			return http.StatusForbidden, denied
		}
		return http.StatusOK, TokenVerifyResponse{Authorized: true, ReadOnly: true}
	}
	return http.StatusForbidden, denied
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
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Printf("failed to close body: %v", err)
		}
	}(r.Body)

	if req.Room != "" {
		status, resp := h.verifyRoom(r.Context(), req)
		if status != http.StatusOK && status != http.StatusForbidden {
			http.Error(w, http.StatusText(status), status)
			return
		}
		writeVerifyResponse(w, status, resp)
		return
	}

	authorized, err := h.VerifyToken(r.Context(), req.Token)
	if err != nil {
		http.Error(w, fmt.Sprintf("Token verification failed: %v", err), http.StatusUnauthorized)
		return
	}
	writeVerifyResponse(w, http.StatusOK, TokenVerifyResponse{Authorized: authorized})
}

func writeVerifyResponse(w http.ResponseWriter, status int, resp TokenVerifyResponse) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	if err := json.NewEncoder(w).Encode(resp); err != nil {
		log.Printf("failed to encode verify response: %v", err)
	}
}

type Server struct {
	handler *AuthHandler
}

func NewServer(jwtProviders []appx.JWTProvider, projects ProjectAccessChecker) *Server {
	handler := NewAuthHandler(jwtProviders, projects)
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
