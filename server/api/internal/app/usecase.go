package app

import (
	"context"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountusecase/accountgateway"
)

func UsecaseMiddleware(r *repo.Container, g *gateway.Container, ag *accountgateway.Container, permissionChecker gateway.PermissionChecker, GQLClient *gql.Client, sharedJob interfaces.Job, config interactor.ContainerConfig) echo.MiddlewareFunc {
	return ContextMiddleware(func(ctx context.Context) context.Context {
		repos := r

		uc := interactor.NewContainer(repos, g, ag, permissionChecker, GQLClient, sharedJob, config)
		ctx = adapter.AttachUsecases(ctx, &uc)
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
