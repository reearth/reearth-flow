package grpc

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

type WorkspaceService struct {
	proto.UnimplementedWorkspaceServiceServer
}

func NewWorkspaceService() *WorkspaceService {
	return &WorkspaceService{}
}

func (s *WorkspaceService) VerifyWorkspaceToken(ctx context.Context, req *proto.WorkspaceTokenVerifyRequest) (*proto.WorkspaceTokenVerifyResponse, error) {
	// Get auth info from context
	authInfo := adapter.GetAuthInfo(ctx)
	if authInfo == nil {
		return nil, status.Error(codes.Unauthenticated, "unauthorized")
	}

	return &proto.WorkspaceTokenVerifyResponse{
		Authorized: true,
	}, nil
}
