package grpc

import (
	"context"
	"fmt"
	"strings"

	"github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/appx"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

type AuthService struct {
	proto.UnimplementedAuthServiceServer
	authConfig []appx.JWTProvider
}

func NewAuthService(authConfig []appx.JWTProvider) *AuthService {
	return &AuthService{
		authConfig: authConfig,
	}
}

func (s *AuthService) VerifyAPIToken(ctx context.Context, req *proto.APITokenVerifyRequest) (*proto.APITokenVerifyResponse, error) {
	token := req.GetToken()
	if !strings.HasPrefix(token, "Bearer ") {
		token = fmt.Sprintf("Bearer %s", token)
	}
	token = strings.TrimPrefix(token, "Bearer ")

	validator, err := appx.NewJWTMultipleValidator(s.authConfig)
	if err != nil {
		return nil, status.Error(codes.Unauthenticated, "failed to initialize validator")
	}

	_, err = validator.ValidateToken(ctx, token)
	if err != nil {
		return nil, status.Error(codes.Unauthenticated, "invalid token")
	}

	return &proto.APITokenVerifyResponse{
		Authorized: true,
	}, nil
}
