package app

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"

	echo "github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql"
	"github.com/reearth/reearthx/log"
)

type tempNewAuthMiddlewares []echo.MiddlewareFunc

type tempNewAuthMiddlewaresParam struct {
	GQLClient *gql.Client
}

func newTempNewAuthMiddlewares(param *tempNewAuthMiddlewaresParam) tempNewAuthMiddlewares {
	return []echo.MiddlewareFunc{
		jwtContextMiddleware(),
		tempNewAuthMiddleware(param.GQLClient),
	}
}

func jwtContextMiddleware() echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			authHeader := c.Request().Header.Get("Authorization")
			if authHeader != "" {
				// Remove the "Bearer " prefix from the Authorization header to extract the token
				const bearerPrefix = "Bearer "
				if len(authHeader) > len(bearerPrefix) && authHeader[:len(bearerPrefix)] == bearerPrefix {
					token := authHeader[len(bearerPrefix):]
					ctx := adapter.AttachJWT(c.Request().Context(), token)
					c.SetRequest(c.Request().WithContext(ctx))
				}
			}
			return next(c)
		}
	}
}

func tempNewAuthMiddleware(gqlClient *gql.Client) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			ctx := c.Request().Context()

			// TODO: Optimize performance by including necessary user information (userID, email, etc.)
			// directly in the JWT token instead of executing a GQL query on every request.
			// This will eliminate the overhead of making an API call to fetch user data for each request.
			u, err := gqlClient.UserRepo.FindMe(ctx)
			if err != nil {
				log.Errorc(ctx, err, "failed to fetch user")
				return echo.NewHTTPError(http.StatusInternalServerError, "server error: failed to fetch user")
			}
			if u == nil {
				return echo.NewHTTPError(http.StatusUnauthorized, "unauthorized: user not found")
			}

			ctx = adapter.AttachFlowUser(ctx, u)
			c.SetRequest(c.Request().WithContext(ctx))
			return next(c)
		}
	}
}

// TODO: This function is in the process of migrating the task "Replace user management in API with reearth accounts".
// Once completed, only `tempNewAuthMWs` will be used, making this function unnecessary.
func conditionalGraphQLAuthMiddleware(
	defaultMWs []echo.MiddlewareFunc,
	tempNewAuthMWs []echo.MiddlewareFunc,
) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			middlewares := defaultMWs

			if c.Path() == "/api/graphql" && c.Request().Method == http.MethodPost {
				var body struct {
					OperationName string `json:"operationName"`
				}
				data, err := io.ReadAll(c.Request().Body)
				if err == nil {
					_ = json.Unmarshal(data, &body)
					c.Request().Body = io.NopCloser(bytes.NewBuffer(data))
				}

				switch body.OperationName {
				case "GetMe":
					middlewares = tempNewAuthMWs
				case "GetWorkspaceById", "GetWorkspaces", "SearchUser", "UpdateMe", "CreateWorkspace", "UpdateWorkspace", "DeleteWorkspace":
					middlewares = append(defaultMWs, tempNewAuthMWs...)
				}
			}

			composed := next
			for i := len(middlewares) - 1; i >= 0; i-- {
				composed = middlewares[i](composed)
			}
			return composed(c)
		}
	}
}
