package app

import (
	"context"
	"crypto/subtle"
	"errors"
	"net"
	"os"
	"os/signal"
	"runtime/debug"
	"strings"
	"time"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/adapter/internalapi"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/accountsdb"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/permission"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// userIDMetadataKey carries the reearth-accounts ID of the user a call is made for.
const userIDMetadataKey = "user-id"

// anonymousMethods may be called without a user-id. They read public
// projects only, unless a user-id names a workspace member.
var anonymousMethods = map[string]bool{
	flowv1.ReEarthFlow_ListProjects_FullMethodName:       true,
	flowv1.ReEarthFlow_ListPublicProjects_FullMethodName: true,
	flowv1.ReEarthFlow_GetProject_FullMethodName:         true,
}

// startInternalAPI serves only the gRPC internal API until interrupted.
func startInternalAPI(cfg *ServerConfig) {
	if cfg.Config.InternalAPI_Token == "" {
		log.Fatalf("internal API: REEARTH_FLOW_INTERNALAPI_TOKEN is required")
	}
	port := cfg.Config.InternalAPI_Port
	if port == "" {
		port = "50051"
	}

	lis, err := net.Listen("tcp", ":"+port)
	if err != nil {
		log.Fatalf("internal API: failed to listen on %s: %v", port, err)
	}
	s := NewInternalAPIServer(cfg)

	go func() {
		log.Infof("internal API: serving gRPC on :%s", port)
		if err := s.Serve(lis); err != nil {
			log.Fatalf("internal API: %v", err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	<-quit
	log.Info("internal API: shutting down")
	s.GracefulStop()
}

// NewInternalAPIServer builds the gRPC server for the internal API.
func NewInternalAPIServer(cfg *ServerConfig) *grpc.Server {
	uc := newInternalUsecases(cfg)
	s := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			unaryRecoverInterceptor(),
			unaryLogInterceptor(),
			unaryAuthInterceptor(cfg.Config.InternalAPI_Token),
			unaryAttachUserInterceptor(cfg),
			unaryAttachUsecasesInterceptor(uc),
		),
		grpc.StatsHandler(otelgrpc.NewServerHandler()),
	)
	flowv1.RegisterReEarthFlowServer(s, internalapi.NewServer())
	return s
}

// newInternalUsecases builds the usecases with permissions decided from the
// accounts database, since an internal call carries a user ID but none of the
// user's tokens, and the accounts service only checks permissions for the
// user whose token it is given. The accounts client therefore serves only
// workspace reads, from the same database; none of the internal API's
// usecases use its user or Cerbos repositories.
func newInternalUsecases(cfg *ServerConfig) *interfaces.Container {
	checker := permission.NewLocalChecker(cfg.AccountRepos.Workspace)
	accounts := &gqlclient.Client{WorkspaceRepo: accountsdb.NewWorkspaces(cfg.AccountRepos.Workspace)}
	job := interactor.NewJob(cfg.Repos, cfg.Gateways, checker)
	uc := interactor.NewContainer(cfg.Repos, cfg.Gateways, checker, accounts, job, interactor.ContainerConfig{
		SignupSecret:             cfg.Config.SignupSecret,
		AuthSrvUIDomain:          cfg.Config.Host_Web,
		Host:                     cfg.Config.Host,
		SharedPath:               cfg.Config.SharedPath,
		WebsocketThriftServerURL: cfg.Config.WebsocketThriftServerURL,
		WebsocketAPISecret:       cfg.Config.WebsocketAPISecret,
		SkipPermissionCheck:      cfg.Config.SkipPermissionCheck,
	})
	return &uc
}

func unaryRecoverInterceptor() grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (_ any, err error) {
		defer func() {
			if r := recover(); r != nil {
				log.Errorfc(ctx, "internal API: panic in %s: %v\n%s", info.FullMethod, r, debug.Stack())
				err = status.Error(codes.Internal, "internal error")
			}
		}()
		return handler(ctx, req)
	}
}

func unaryLogInterceptor() grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		method := info.FullMethod[strings.LastIndex(info.FullMethod, "/")+1:]
		start := time.Now()
		res, err := handler(ctx, req)
		if err != nil {
			log.Infofc(ctx, "internal API: %s (%s): %s", method, time.Since(start), status.Code(err))
		} else {
			log.Infofc(ctx, "internal API: %s (%s)", method, time.Since(start))
		}
		return res, err
	}
}

// unaryAuthInterceptor requires the internal token on every call.
func unaryAuthInterceptor(token string) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		got, ok := strings.CutPrefix(firstMetadata(ctx, "authorization"), "Bearer ")
		if !ok || subtle.ConstantTimeCompare([]byte(got), []byte(token)) != 1 {
			return nil, status.Error(codes.Unauthenticated, "invalid token")
		}
		return handler(ctx, req)
	}
}

// unaryAttachUserInterceptor attaches the user named by the user-id metadata.
// Only anonymousMethods may be called without one.
func unaryAttachUserInterceptor(cfg *ServerConfig) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		userID := firstMetadata(ctx, userIDMetadataKey)
		if userID == "" {
			if anonymousMethods[info.FullMethod] {
				return handler(ctx, req)
			}
			return nil, status.Error(codes.Unauthenticated, "user-id is required")
		}

		u, err := accountsdb.FindUser(ctx, cfg.AccountRepos.User, userID)
		if errors.Is(err, rerror.ErrNotFound) {
			return nil, status.Error(codes.Unauthenticated, "unknown user")
		}
		if err != nil {
			log.Errorfc(ctx, "internal API: loading user %s: %v", userID, err)
			return nil, status.Error(codes.Internal, "internal error")
		}
		return handler(adapter.AttachUser(ctx, u), req)
	}
}

func unaryAttachUsecasesInterceptor(uc *interfaces.Container) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, _ *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		ctx = adapter.AttachUsecases(ctx, uc)
		ctx = adapter.AttachPermissionVerdictMemo(ctx)
		return handler(ctx, req)
	}
}

// firstMetadata returns the first value of an incoming metadata key, or "".
func firstMetadata(ctx context.Context, key string) string {
	md, _ := metadata.FromIncomingContext(ctx)
	if v := md.Get(key); len(v) > 0 {
		return v[0]
	}
	return ""
}
