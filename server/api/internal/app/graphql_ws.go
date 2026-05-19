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
	if token := strings.TrimPrefix(auth, "Bearer "); token != "" {
		ctx = adapter.AttachJWT(ctx, token)
	}
	return ctx, &initPayload, nil
}
