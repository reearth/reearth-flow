package app

import (
	"context"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
)

func UsecaseMiddleware(uc *interfaces.Container) echo.MiddlewareFunc {
	return ContextMiddleware(func(ctx context.Context) context.Context {
		ctx = adapter.AttachUsecases(ctx, uc)
		// Request-scoped memo for non-GraphQL routes such as job cancel.
		// GraphqlAPI shadows it with a fresh memo per operation.
		ctx = adapter.AttachPermissionVerdictMemo(ctx)
		return ctx
	})
}

func ContextMiddleware(fn func(ctx context.Context) context.Context) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			req := c.Request()
			c.SetRequest(req.WithContext(fn(req.Context())))
			return next(c)
		}
	}
}
