package grpc

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearthx/appx"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

func AuthInterceptor(providers []appx.JWTProvider) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			return nil, status.Error(codes.Unauthenticated, "missing metadata")
		}

		// Get authorization header
		authHeader := md.Get("authorization")
		if len(authHeader) == 0 {
			return nil, status.Error(codes.Unauthenticated, "missing authorization header")
		}

		// Verify token
		authInfo, err := appx.CheckJWTToken(authHeader[0], providers)
		if err != nil {
			return nil, status.Error(codes.Unauthenticated, "invalid token")
		}

		// Add auth info to context
		newCtx := adapter.AttachAuthInfo(ctx, authInfo)
		return handler(newCtx, req)
	}
}
