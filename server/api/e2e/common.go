package e2e

import (
	"context"
	"net"
	"net/http"
	"net/url"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/google/uuid"
	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	gqluser "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user"
	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	"github.com/reearth/reearth-flow/api/internal/app"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmongo"
	"github.com/reearth/reearthx/account/accountusecase/accountgateway"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/mailer"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/samber/lo"
	"github.com/spf13/afero"
)

type TestMocks struct {
	UserRepo      gqluser.UserRepo
	WorkspaceRepo gqlworkspace.WorkspaceRepo
}

func init() {
	mongotest.Env = "REEARTH_FLOW_DB"
}

func StartServer(t *testing.T, cfg *config.Config, useMongo bool, allowPermission bool, mock *TestMocks) *httpexpect.Expect {
	e, _, _ := StartServerAndRepos(t, cfg, useMongo, allowPermission, mock)
	return e
}

func initRepos(t *testing.T, useMongo bool) (repos *repo.Container) {
	ctx := context.Background()

	if useMongo {
		db := mongotest.Connect(t)(t)
		accountRepos := lo.Must(accountmongo.New(ctx, db.Client(), db.Name(), false, false, nil))
		repos = lo.Must(mongo.New(ctx, db, accountRepos, false))
	} else {
		repos = memory.New()
	}

	return repos
}

func initGateway() *gateway.Container {
	return &gateway.Container{
		File: lo.Must(fs.NewFile(afero.NewMemMapFs(), "https://example.com", "https://example2.com")),
	}
}

func StartServerAndRepos(t *testing.T, cfg *config.Config, useMongo bool, allowPermission bool, mock *TestMocks) (*httpexpect.Expect, *repo.Container, *gateway.Container) {
	repos := initRepos(t, useMongo)
	gateways := initGateway()
	return StartServerWithRepos(t, cfg, repos, gateways, allowPermission, mock), repos, gateways
}

func StartServerWithRepos(t *testing.T, cfg *config.Config, repos *repo.Container, gateways *gateway.Container, allowPermission bool, mock *TestMocks) *httpexpect.Expect {
	t.Helper()

	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}

	ctx := context.Background()

	l, err := net.Listen("tcp", ":0")
	if err != nil {
		t.Fatalf("server failed to listen: %v", err)
	}

	// mockPermissionChecker
	mockPermissionChecker := gateway.NewMockPermissionChecker()
	mockPermissionChecker.Allow = allowPermission

	// mockAccountGQLClient
	var accountGQLClient *gqlclient.Client
	if mock != nil {
		accountGQLClient = gql.NewMockClient(&gql.MockClientParam{
			UserRepo:      mock.UserRepo,
			WorkspaceRepo: mock.WorkspaceRepo,
		})
	}

	srv := app.NewServer(ctx, &app.ServerConfig{
		Config:            cfg,
		Repos:             repos,
		Gateways:          gateways,
		Debug:             true,
		AccountRepos:      repos.AccountRepos(),
		PermissionChecker: mockPermissionChecker,
		AccountGQLClient:  accountGQLClient,
	})

	ch := make(chan error)
	go func() {
		if err := srv.Serve(l); err != http.ErrServerClosed {
			ch <- err
		}
		close(ch)
	}()

	t.Cleanup(func() {
		if err := srv.Shutdown(context.Background()); err != nil {
			t.Fatalf("server shutdown: %v", err)
		}

		if err := <-ch; err != nil {
			t.Fatalf("server serve: %v", err)
		}
	})

	return httpexpect.Default(t, "http://"+l.Addr().String())
}

type GraphQLRequest struct {
	OperationName string         `json:"operationName"`
	Query         string         `json:"query"`
	Variables     map[string]any `json:"variables"`
}

func StartGQLServer(t *testing.T, cfg *config.Config, useMongo bool, allowPermission bool, mock *TestMocks) (*httpexpect.Expect, *accountrepo.Container) {
	e, r := StartGQLServerAndRepos(t, cfg, useMongo, allowPermission, mock)
	return e, r
}

func StartGQLServerAndRepos(t *testing.T, cfg *config.Config, useMongo bool, allowPermission bool, mock *TestMocks) (*httpexpect.Expect, *accountrepo.Container) {
	repos := initRepos(t, useMongo)
	acRepos := repos.AccountRepos()
	return StartGQLServerWithRepos(t, cfg, repos, acRepos, allowPermission, mock), acRepos
}

func StartGQLServerWithRepos(t *testing.T, cfg *config.Config, repos *repo.Container, accountrepos *accountrepo.Container, allowPermission bool, mock *TestMocks) *httpexpect.Expect {
	t.Helper()

	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}

	ctx := context.Background()

	l, err := net.Listen("tcp", ":0")
	if err != nil {
		t.Fatalf("server failed to listen: %v", err)
	}

	// mockPermissionChecker
	mockPermissionChecker := gateway.NewMockPermissionChecker()
	mockPermissionChecker.Allow = allowPermission

	// mockAccountGQLClient
	var accountGQLClient *gqlclient.Client
	if mock != nil {
		accountGQLClient = gql.NewMockClient(&gql.MockClientParam{
			UserRepo:      mock.UserRepo,
			WorkspaceRepo: mock.WorkspaceRepo,
		})
	}

	cfg.SkipPermissionCheck = true
	base := lo.Must(fs.NewFile(afero.NewMemMapFs(), "https://example.com", "https://example2.com"))
	srv := app.NewServer(ctx, &app.ServerConfig{
		Config:       cfg,
		Repos:        repos,
		AccountRepos: accountrepos,
		Gateways: &gateway.Container{
			File: &UploadableFileRepo{
				File:      base,
				DummyBase: "http://localhost:4443",
			},
		},
		AccountGateways: &accountgateway.Container{
			Mailer: mailer.New(ctx, &mailer.Config{}),
		},
		Debug:             true,
		PermissionChecker: mockPermissionChecker,
		AccountGQLClient:  accountGQLClient,
	})

	ch := make(chan error)
	go func() {
		if err := srv.Serve(l); err != http.ErrServerClosed {
			ch <- err
		}
		close(ch)
	}()
	t.Cleanup(func() {
		if err := srv.Shutdown(context.Background()); err != nil {
			t.Fatalf("server shutdown: %v", err)
		}

		if err := <-ch; err != nil {
			t.Fatalf("server serve: %v", err)
		}
	})
	return httpexpect.Default(t, "http://"+l.Addr().String())
}

type UploadableFileRepo struct {
	gateway.File
	DummyBase string
}

func (t *UploadableFileRepo) IssueUploadAssetLink(_ context.Context, p gateway.IssueUploadAssetParam) (*gateway.UploadAssetLink, error) {
	u, _ := url.Parse(t.DummyBase)
	if u.Scheme == "" {
		u.Scheme = "http"
	}
	u.Path = "/dummy-upload/" + uuid.New().String()

	ct := p.GetOrGuessContentType()
	return &gateway.UploadAssetLink{
		URL:             u.String(),
		ContentType:     ct,
		ContentLength:   p.ContentLength,
		ContentEncoding: p.ContentEncoding,
		Next:            "",
	}, nil
}
