package app

import (
	"context"
	"errors"
	"net/http"
	"net/http/pprof"

	"github.com/99designs/gqlgen/graphql/playground"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
	"github.com/samber/lo"
	"go.opentelemetry.io/contrib/instrumentation/github.com/labstack/echo/otelecho"
)

func initEcho(ctx context.Context, cfg *ServerConfig) *echo.Echo {
	if cfg.Config == nil {
		log.Fatalf("ServerConfig.Config is nil")
	}

	e := echo.New()
	e.Debug = cfg.Debug
	e.HideBanner = true
	e.HidePort = true
	e.HTTPErrorHandler = errorHandler(e.DefaultHTTPErrorHandler)

	// basic middleware
	logger := log.NewEcho()
	e.Logger = logger
	e.Use(
		middleware.Recover(),
		otelecho.Middleware("reearth-flow"),
		echo.WrapMiddleware(appx.RequestIDMiddleware()),
		logger.AccessLogger(),
		middleware.Gzip(),
	)
	if cfg.Config.HTTPSREDIRECT {
		e.Use(middleware.HTTPSRedirectWithConfig(middleware.RedirectConfig{
			Code: http.StatusTemporaryRedirect,
		}))
	}
	origins := allowedOrigins(cfg)
	if len(origins) > 0 {
		e.Use(
			middleware.CORSWithConfig(middleware.CORSConfig{
				AllowOrigins: origins,
			}),
		)
	}

	// auth config
	authConfig := cfg.Config.JWTProviders()

	log.Infof("auth: config: %#v", authConfig)
	authMiddleware := echo.WrapMiddleware(lo.Must(appx.AuthMiddleware(authConfig, adapter.ContextAuthInfo, true)))
	// TODO: During migration, continue using this temporary middleware for selected GraphQL operations.
	// After migration, this will become the standard authMiddleware and the function will be renamed accordingly.
	tempNewAuthMiddlewares := newTempNewAuthMiddlewares(&tempNewAuthMiddlewaresParam{
		GQLClient: cfg.AccountGQLClient,
	})

	// enable pprof
	if e.Debug {
		pprofGroup := e.Group("/debug/pprof")
		pprofGroup.Any("/cmdline", echo.WrapHandler(http.HandlerFunc(pprof.Cmdline)))
		pprofGroup.Any("/profile", echo.WrapHandler(http.HandlerFunc(pprof.Profile)))
		pprofGroup.Any("/symbol", echo.WrapHandler(http.HandlerFunc(pprof.Symbol)))
		pprofGroup.Any("/trace", echo.WrapHandler(http.HandlerFunc(pprof.Trace)))
		pprofGroup.Any("/*", echo.WrapHandler(http.HandlerFunc(pprof.Index)))
	}

	// GraphQL Playground without auth
	gqldev := cfg.Debug || cfg.Config.Dev
	if gqldev {
		e.GET("/graphql", echo.WrapHandler(
			playground.Handler("reearth-flow", "/api/graphql"),
		))
		log.Infofc(ctx, "gql: GraphQL Playground is available")
	}

	sharedJob := interactor.NewJob(cfg.Repos, cfg.Gateways, cfg.PermissionChecker)
	e.Use(UsecaseMiddleware(cfg.Repos, cfg.Gateways, cfg.AccountRepos, cfg.AccountGateways, cfg.PermissionChecker, cfg.AccountGQLClient, sharedJob, interactor.ContainerConfig{
		SignupSecret:        cfg.Config.SignupSecret,
		AuthSrvUIDomain:     cfg.Config.Host_Web,
		Host:                cfg.Config.Host,
		SharedPath:          cfg.Config.SharedPath,
		SkipPermissionCheck: cfg.Config.SkipPermissionCheck,
	}))

	// auth srv
	authServer(ctx, e, &cfg.Config.AuthSrv, cfg.Repos)

	// apis
	api := e.Group("/api")
	api.GET("/ping", Ping(), privateCache)

	// authenticated routes
	apiPrivate := api.Group("", privateCache)
	apiPrivate.Use(
		conditionalGraphQLAuthMiddleware(
			[]echo.MiddlewareFunc{authMiddleware, attachOpMiddleware(cfg)},
			tempNewAuthMiddlewares,
		),
	)
	apiPrivate.POST("/signup", Signup())
	apiPrivate.Any("/graphql", GraphqlAPI(cfg.Config.GraphQL, gqldev, origins))

	if !cfg.Config.AuthSrv.Disabled {
		apiPrivate.POST("/signup/verify", StartSignupVerify())
		apiPrivate.POST("/signup/verify/:code", SignupVerify())
		apiPrivate.POST("/password-reset", PasswordReset())
	}
	if err := initActionsData(ctx); err != nil {
		log.Errorf("Failed to initialize actions data: %v", err)
	}
	SetupActionRoutes(e)

	SetupTriggerRoutes(e)

	serveFiles(e, cfg.Gateways.File)

	Web(e, cfg.Config.WebConfig(), cfg.Config.AuthForWeb(), cfg.Config.Web_Disabled, nil)

	return e
}

func initActionsData(_ context.Context) error {
	for lang := range supportedLangs {
		if err := loadActionsData(lang); err != nil {
			log.Errorf("Failed to load actions data for language %s: %v", lang, err)
		}
	}
	return nil
}

func errorHandler(next func(error, echo.Context)) func(error, echo.Context) {
	return func(err error, c echo.Context) {
		if c.Response().Committed {
			return
		}

		if errors.Is(err, echo.ErrNotFound) {
			err = rerror.ErrNotFound
		}

		code, msg := errorMessage(err, func(f string, args ...interface{}) {
			log.Errorfc(c.Request().Context(), f, args...)
		})
		if err := c.JSON(code, map[string]string{
			"error": msg,
		}); err != nil {
			next(err, c)
		}
	}
}

func allowedOrigins(cfg *ServerConfig) []string {
	if cfg == nil {
		return nil
	}
	origins := append([]string{}, cfg.Config.Origins...)
	if cfg.Debug {
		origins = append(origins, "http://localhost:*")
	}
	return origins
}

func errorMessage(err error, log func(string, ...interface{})) (int, string) {
	code := http.StatusBadRequest
	msg := err.Error()

	var err2 *echo.HTTPError
	if errors.As(err, &err2) {
		code = err2.Code
		if msg2, ok := err2.Message.(string); ok {
			msg = msg2
		} else if msg2, ok := err2.Message.(error); ok {
			msg = msg2.Error()
		} else {
			msg = "error"
		}
		if err2.Internal != nil {
			log("echo internal err: %+v", err2)
		}
	}

	return code, msg
}

func privateCache(next echo.HandlerFunc) echo.HandlerFunc {
	return func(c echo.Context) error {
		c.Response().Header().Set(echo.HeaderCacheControl, "private, no-store, no-cache, must-revalidate")
		return next(c)
	}
}
