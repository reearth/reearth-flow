package thrift

import (
	"context"
	"fmt"
	"log"
	"strings"

	"github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/appx"
)

type AuthServiceHandler struct {
	authConfig []appx.JWTProvider
}

func NewAuthServiceHandler(authConfig []appx.JWTProvider) *AuthServiceHandler {
	return &AuthServiceHandler{
		authConfig: authConfig,
	}
}

func (s *AuthServiceHandler) VerifyAPIToken(ctx context.Context, req *proto.APITokenVerifyRequest) (*proto.APITokenVerifyResponse, error) {
	token := req.Token
	if !strings.HasPrefix(token, "Bearer ") {
		token = fmt.Sprintf("Bearer %s", token)
	}
	token = strings.TrimPrefix(token, "Bearer ")

	validator, err := appx.NewJWTMultipleValidator(s.authConfig)
	if err != nil {
		log.Printf("failed to initialize validator: %v", err)
		return nil, fmt.Errorf("failed to initialize validator: %v", err)
	}

	_, err = validator.ValidateToken(ctx, token)
	if err != nil {
		log.Printf("failed to validate token %s: %v", token, err)
		return nil, fmt.Errorf("invalid token: %v", err)
	}

	return &proto.APITokenVerifyResponse{
		Authorized: true,
	}, nil
}
