package grpc

import (
	"fmt"
	"net"

	"github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/appx"
	"google.golang.org/grpc"
)

type Server struct {
	server      *grpc.Server
	port        string
	authService *AuthService
}

func NewServer(port string, jwtProviders []appx.JWTProvider) *Server {
	server := grpc.NewServer()
	authService := NewAuthService(jwtProviders)

	proto.RegisterAuthServiceServer(server, authService)

	return &Server{
		server:      server,
		port:        port,
		authService: authService,
	}
}

func (s *Server) Start() error {
	lis, err := net.Listen("tcp", fmt.Sprintf(":%s", s.port))
	if err != nil {
		return fmt.Errorf("failed to listen: %v", err)
	}

	return s.server.Serve(lis)
}

func (s *Server) Stop() {
	s.server.GracefulStop()
}
