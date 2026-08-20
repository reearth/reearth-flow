package interactor

import (
	"context"
	"io"
	stdlog "log"
	"testing"
	"time"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

// checkPermission logs unconditionally via the stdlib "log" package; that
// I/O would otherwise dominate every iteration's timing here.
func discardStdLog(b *testing.B) {
	b.Helper()
	prev := stdlog.Writer()
	stdlog.SetOutput(io.Discard)
	b.Cleanup(func() { stdlog.SetOutput(prev) })
}

// BenchmarkUsecaseContainerAccess measures the cost of NewContainer, which
// UsecaseMiddleware currently calls once per request. This is the baseline
// against which turning the container into a boot-time singleton gets
// judged: per-request construction cost should disappear entirely.
func BenchmarkUsecaseContainerAccess(b *testing.B) {
	r := &repo.Container{}
	g := &gateway.Container{}
	permissionChecker := NewMockPermissionChecker(nil)
	gqlClient := &gqlclient.Client{}
	cfg := ContainerConfig{}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = NewContainer(r, g, permissionChecker, gqlClient, nil, cfg)
	}
}

// BenchmarkPermissionCheck measures checkPermission, the gate every
// resolver runs through before touching a repo or gateway.
func BenchmarkPermissionCheck(b *testing.B) {
	discardStdLog(b)
	ctx := context.Background()
	permissionChecker := NewMockPermissionChecker(func(context.Context, string, string) (bool, error) {
		return true, nil
	})
	wsID := accountsid.NewWorkspaceID()

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if err := checkPermission(ctx, permissionChecker, rbac.ResourceLog, rbac.ActionAny, wsID); err != nil {
			b.Fatal(err)
		}
	}
}

// BenchmarkLogRead measures LogInteractor.GetLogs end to end (permission
// check + job lookup + Redis-backed log fetch), against in-memory fakes so
// it runs without a live Redis/Mongo/Postgres instance.
func BenchmarkLogRead(b *testing.B) {
	discardStdLog(b)
	jobID := id.NewJobID()
	nodeID := log.NodeID(id.NewNodeID())

	logs := make([]*log.Log, 0, 100)
	for i := 0; i < 100; i++ {
		logs = append(logs, log.NewLog(jobID, &nodeID, time.Now(), log.LevelInfo, "benchmark log line"))
	}

	redisMock := &mockLogGateway{logs: logs}
	jobRepoMock := &mockJobRepo{}
	permissionChecker := NewMockPermissionChecker(func(context.Context, string, string) (bool, error) {
		return true, nil
	})
	li := NewLogInteractor(redisMock, jobRepoMock, permissionChecker)

	ctx := context.Background()
	since := time.Now().Add(-30 * time.Minute)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if _, err := li.GetLogs(ctx, since, jobID); err != nil {
			b.Fatal(err)
		}
	}
}
