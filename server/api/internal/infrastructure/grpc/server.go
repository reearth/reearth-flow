package grpc

import (
	"net/http"
	"strings"

	"github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/appx"
	"google.golang.org/grpc"
)

type Server struct {
	server      *grpc.Server
	authService *AuthService
}

func NewServer(_ string, jwtProviders []appx.JWTProvider) *Server {
	server := grpc.NewServer()
	authService := NewAuthService(jwtProviders)

	proto.RegisterAuthServiceServer(server, authService)

	return &Server{
		server:      server,
		authService: authService,
	}
}

func (s *Server) Stop() {
	s.server.GracefulStop()
}

func (s *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.ProtoMajor == 2 && strings.Contains(r.Header.Get("Content-Type"), "application/grpc") {
		s.server.ServeHTTP(w, r)
		return
	}
	http.Error(w, "unsupported protocol", http.StatusBadRequest)
}
