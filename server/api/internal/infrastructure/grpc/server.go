package grpc

import (
	"context"
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

func (s *Server) Stop() {
	s.server.GracefulStop()
}

func (s *Server) StartWithContext(ctx context.Context) error {
	lis, err := net.Listen("tcp", fmt.Sprintf(":%s", s.port))
	if err != nil {
		return fmt.Errorf("failed to listen: %v", err)
	}

	go func() {
		<-ctx.Done()
		s.Stop()
	}()

	return s.server.Serve(lis)
}
