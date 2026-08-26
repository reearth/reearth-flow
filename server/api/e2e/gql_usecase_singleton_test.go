package e2e

import (
	"context"
	"sync/atomic"
	"testing"
	"time"

	gqlgenclient "github.com/99designs/gqlgen/client"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/app"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	infragql "github.com/reearth/reearth-flow/api/internal/infrastructure/gql"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/samber/lo"
	"github.com/spf13/afero"
	"github.com/stretchr/testify/require"
)

// countingRedisGateway counts GetLogs calls so the test can tell a single
// shared poller ticking on a job apart from two independent ones.
type countingRedisGateway struct {
	getLogsCalls atomic.Int64
}

func (g *countingRedisGateway) GetLogs(_ context.Context, _, _ time.Time, _ id.JobID) ([]*log.Log, error) {
	g.getLogsCalls.Add(1)
	return nil, nil
}

func (g *countingRedisGateway) GetUserFacingLogs(_ context.Context, _, _ time.Time, _ id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	return nil, nil
}

func (g *countingRedisGateway) GetJobCompleteEvent(_ context.Context, _ id.JobID) (*gateway.JobCompleteEvent, error) {
	return nil, nil
}

func (g *countingRedisGateway) DeleteJobCompleteEvent(_ context.Context, _ id.JobID) error {
	return nil
}

func (g *countingRedisGateway) GetNodeDiagnostics(_ context.Context, _ id.JobID, _ string) ([]*diagnostic.Diagnostic, error) {
	return nil, nil
}

func (g *countingRedisGateway) GetJobDiagnostics(_ context.Context, _ id.JobID) ([]*diagnostic.Diagnostic, error) {
	return nil, nil
}

const logsSubscriptionQuery = `subscription($jobId: ID!) { logs(jobId: $jobId) { jobId message } }`

// TestUsecaseContainer_SharedAcrossRequests_LogPollerIsNotDuplicated boots the
// real server the same way main.Start does -- one interactor.Container built
// once, reused by every request -- and proves the fix end-to-end: two
// independent GraphQL subscriptions on the same running job collapse onto a
// single background log poller.
//
// Before this fix, UsecaseMiddleware called interactor.NewContainer per
// request, so each subscription got its own LogInteractor with a fresh
// watchers map and its own 15s poller. No unit test on the interactor alone
// can see that regression: it never has two requests sharing, or failing to
// share, one container. Only booting the real server and driving it with two
// separate connections proves the container the server actually serves is
// the singleton.
//
// Memory-backed on purpose (no live MongoDB needed) so this runs anywhere.
func TestUsecaseContainer_SharedAcrossRequests_LogPollerIsNotDuplicated(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}

	ctx := context.Background()
	repos := memory.New()
	// memory.New doesn't wire a Job repo (it's not needed by the e2e suites
	// that already exist); this test needs one to persist a running job.
	repos.Job = memory.NewJob()

	runningJob, err := job.New().
		NewID().
		Deployment(id.NewDeploymentID().Ref()).
		Workspace(accountsid.NewWorkspaceID()).
		Status(job.StatusRunning).
		StartedAt(time.Now()).
		Build()
	require.NoError(t, err)
	require.NoError(t, repos.Job.Save(ctx, runningJob))

	redisGateway := &countingRedisGateway{}
	gateways := &gateway.Container{
		File:  lo.Must(fs.NewFile(afero.NewMemMapFs(), "https://example.com", "https://example2.com")),
		Redis: redisGateway,
	}

	mockPermissionChecker := gateway.NewMockPermissionChecker()
	mockPermissionChecker.Allow = true

	srv := app.NewServer(ctx, &app.ServerConfig{
		Config:            &config.Config{AuthSrv: config.AuthSrvConfig{Disabled: true}},
		Repos:             repos,
		Gateways:          gateways,
		AccountRepos:      repos.AccountRepos(),
		PermissionChecker: mockPermissionChecker,
		AccountGQLClient:  infragql.NewMockClient(&infragql.MockClientParam{}),
		Debug:             true,
	})

	cli := gqlgenclient.New(srv, gqlgenclient.Path("/api/graphql"))

	sub1 := cli.Websocket(logsSubscriptionQuery, gqlgenclient.Var("jobId", runningJob.ID().String()))
	defer func() { _ = sub1.Close() }()
	sub2 := cli.Websocket(logsSubscriptionQuery, gqlgenclient.Var("jobId", runningJob.ID().String()))
	defer func() { _ = sub2.Close() }()

	// Both subscriptions trigger their own "initial missed logs" fetch on top
	// of the periodic poller; that is not the behaviour under test, so let it
	// settle first.
	time.Sleep(500 * time.Millisecond)
	afterSubscribe := redisGateway.getLogsCalls.Load()

	// Wait for the first monitoring tick (internal/usecase/interactor/log.go,
	// 15s ticker) rather than sleeping a fixed window: this is both faster in
	// the common case and avoids CI scheduling jitter around tick alignment.
	var afterFirstTick int64
	require.Eventually(t, func() bool {
		afterFirstTick = redisGateway.getLogsCalls.Load()
		return afterFirstTick-afterSubscribe >= 1
	}, 20*time.Second, 100*time.Millisecond, "expected at least one GetLogs call from the poller")

	// A single shared poller issues exactly one GetLogs call for the first
	// tick; two independent pollers -- the bug -- issue two.
	require.Equal(t, int64(1), afterFirstTick-afterSubscribe,
		"expected exactly one poller ticking for the shared job, observed %d GetLogs call(s) by the first tick",
		afterFirstTick-afterSubscribe)

	// Give a duplicated poller with a slightly offset ticker a chance to fire
	// its own first tick too; a genuine second poller starts within
	// milliseconds of the first, so this window need not be long.
	time.Sleep(1 * time.Second)
	afterGrace := redisGateway.getLogsCalls.Load()

	require.Equal(t, afterFirstTick, afterGrace,
		"expected no additional GetLogs call within the grace period, observed %d additional call(s)",
		afterGrace-afterFirstTick)
}
