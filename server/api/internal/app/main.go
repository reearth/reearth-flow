package app

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"strings"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	authserver "github.com/reearth/reearth-flow/api/internal/infrastructure/auth"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountusecase/accountgateway"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	cerbosClient "github.com/reearth/reearthx/cerbos/client"
	"github.com/reearth/reearthx/log"
	"golang.org/x/net/http2"
	"golang.org/x/net/http2/h2c"
)

func Start(debug bool, version string) {
	log.Infof("reerath-flow %s", version)

	ctx := context.Background()

	conf, cerr := config.ReadConfig(debug)
	if cerr != nil {
		log.Fatalf("failed to load config: %v", cerr)
	}
	log.Infof("config: %s", conf.Print())

	initProfiler(conf.Profiler, version)

	closer := initTracer(ctx, conf)
	defer func() {
		if closer != nil {
			if err := closer.Close(); err != nil {
				log.Errorf("Failed to close tracer: %s\n", err.Error())
			}
		}
	}()

	repos, gateways, acRepos, acGateways := initReposAndGateways(ctx, conf, debug)

	// PermissionChecker
	if conf.AccountsApiHost == "" {
		log.Fatalf("accounts host configuration is required")
	}
	if _, err := url.Parse(conf.AccountsApiHost); err != nil {
		log.Fatalf("invalid accounts host URL: %v", err)
	}
	permissionChecker := cerbosClient.NewPermissionChecker(rbac.ServiceName, conf.AccountsApiHost)
	if permissionChecker == nil {
		log.Fatalf("failed to initialize permission checker")
	}

	serverCfg := &ServerConfig{
		Config:            conf,
		Debug:             debug,
		Repos:             repos,
		AccountRepos:      acRepos,
		Gateways:          gateways,
		AccountGateways:   acGateways,
		PermissionChecker: permissionChecker,
	}

	httpServer := NewServer(ctx, serverCfg)
	authService := authserver.NewServer(conf.JWTProviders())

	mainHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if strings.HasPrefix(r.URL.Path, "/auth") {
			authService.ServeHTTP(w, r)
			return
		}
		httpServer.ServeHTTP(w, r)
	})

	log.Infof("Starting server on %s", httpServer.address)

	h2s := &http2.Server{}
	handler := h2c.NewHandler(mainHandler, h2s)

	server := &http.Server{
		Addr:    httpServer.address,
		Handler: handler,
	}

	go func() {
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("failed to run server: %v", err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	<-quit

	log.Info("Shutting down server...")
	if err := server.Shutdown(ctx); err != nil {
		log.Errorf("Server forced to shutdown: %v", err)
	}
}

type WebServer struct {
	address   string
	appServer *echo.Echo
}

type ServerConfig struct {
	Config            *config.Config
	Debug             bool
	Repos             *repo.Container
	AccountRepos      *accountrepo.Container
	Gateways          *gateway.Container
	AccountGateways   *accountgateway.Container
	PermissionChecker gateway.PermissionChecker
}

func NewServer(ctx context.Context, cfg *ServerConfig) *WebServer {
	port := cfg.Config.Port
	if port == "" {
		port = "8080"
	}

	host := cfg.Config.ServerHost
	if host == "" {
		if cfg.Debug {
			host = "localhost"
		} else {
			host = "0.0.0.0"
		}
	}
	address := fmt.Sprintf("%s:%s", host, port)

	e := initEcho(ctx, cfg)

	authServer(ctx, e, &cfg.Config.AuthSrv, cfg.Repos)

	return &WebServer{
		address:   address,
		appServer: e,
	}
}

func (w *WebServer) ServeHTTP(wr http.ResponseWriter, r *http.Request) {
	w.appServer.ServeHTTP(wr, r)
}

func (w *WebServer) Serve(l net.Listener) error {
	return w.appServer.Server.Serve(l)
}

func (w *WebServer) Shutdown(ctx context.Context) error {
	return w.appServer.Shutdown(ctx)
}
