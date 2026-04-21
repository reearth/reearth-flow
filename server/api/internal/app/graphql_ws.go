package app

import (
	"context"
	"strings"

	"github.com/99designs/gqlgen/graphql/handler/transport"
	"github.com/reearth/reearth-flow/api/internal/adapter"
)

func wsInitFunc(ctx context.Context, initPayload transport.InitPayload) (context.Context, *transport.InitPayload, error) {
	auth := initPayload.Authorization()
	if auth == "" {
		if headers, ok := initPayload["headers"].(map[string]any); ok {
			for _, key := range []string{"authorization", "Authorization"} {
				if v, ok := headers[key].(string); ok && v != "" {
					auth = v
					break
				}
			}
		}
	}
	if auth != "" {
		ctx = adapter.AttachJWT(ctx, strings.TrimPrefix(auth, "Bearer "))
	}
	return ctx, &initPayload, nil
}
