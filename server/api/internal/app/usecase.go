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
		// Request-scoped memo for non-GraphQL routes (e.g. job cancel, which
		// checks permission twice for the same workspace). GraphqlAPI attaches
		// its own memo per operation in AroundOperations, which shadows this
		// one and is what actually scopes GraphQL/websocket verdicts correctly.
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
