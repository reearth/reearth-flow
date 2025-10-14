package app

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"

	"github.com/gorilla/websocket"
	echo "github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/log"
)

type authMiddlewares []echo.MiddlewareFunc

type authMiddlewaresParam struct {
	Cfg     *ServerConfig
	SkipOps map[string]struct{}
}

func newAuthMiddlewares(param *authMiddlewaresParam) authMiddlewares {
	jwt, err := appx.AuthMiddleware(param.Cfg.Config.JWTProviders(), adapter.ContextAuthInfo, true)
	if err != nil {
		log.Debug("Failed to create jwt middleware: ", err)
	}

	return []echo.MiddlewareFunc{
		gqlOpNameMiddleware(),
		jwtContextMiddleware(),
		authMiddleware(param.Cfg.AccountGQLClient, param.SkipOps),
		// TODO: Currently, the following middleware is necessary because permission checks such as filterByWorkspaces are performed in mongo.repo.
		// It will be removed when centralized permission checks by the account server are implemented.
		echo.WrapMiddleware(jwt),
		attachOpMiddleware(param.Cfg),
	}
}

func gqlOpNameMiddleware() echo.MiddlewareFunc {
	type bodyShape struct {
		OperationName string `json:"operationName"`
	}
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if c.Path() == "/api/graphql" && c.Request().Method == http.MethodPost {
				data, err := io.ReadAll(c.Request().Body)
				if err == nil && len(data) > 0 {
					_ = c.Request().Body.Close()
					c.Request().Body = io.NopCloser(bytes.NewBuffer(data))

					var b bodyShape
					if json.Unmarshal(data, &b) == nil && b.OperationName != "" {
						ctx := adapter.AttachGQLOperationName(c.Request().Context(), b.OperationName)
						c.SetRequest(c.Request().WithContext(ctx))
					}
				}
			}
			return next(c)
		}
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

func authMiddleware(gqlClient *gql.Client, skipOps map[string]struct{}) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		return func(c echo.Context) error {
			if websocket.IsWebSocketUpgrade(c.Request()) {
				log.Debugfc(c.Request().Context(), "authMiddleware: skip FindMe on WS upgrade (path=%s)", c.Path())
				return next(c)
			}

			if c.Path() == "/api/signup" {
				return next(c)
			}

			if _, skip := skipOps[adapter.GQLOperationName(c.Request().Context())]; skip {
				return next(c)
			}

			ctx := c.Request().Context()

			// TODO: Optimize performance by including necessary user information (userID, email, etc.)
			// directly in the JWT token instead of executing a GQL query on every request.
			// This will eliminate the overhead of making an API call to fetch user data for each request.
			u, err := gqlClient.UserRepo.FindMe(ctx)
			if err != nil {
				log.Debugc(ctx, err, "authMiddleware: FindMe failed; continue as anonymous")
			} else if u == nil {
				log.Debugfc(ctx, "authMiddleware: no user found; continue as anonymous")
			} else {
				ctx = adapter.AttachUser(ctx, u)
				log.Debugfc(ctx, "authMiddleware: user attached: id=%s", u.ID())
			}

			c.SetRequest(c.Request().WithContext(ctx))
			return next(c)
		}
	}
}
