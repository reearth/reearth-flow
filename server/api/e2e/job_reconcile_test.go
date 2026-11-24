package e2e

import (
	"context"
	"encoding/json"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/app"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmongo"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type webhookReceiver struct {
	bodies []map[string]any
	mu     sync.Mutex
}

func (w *webhookReceiver) ServeHTTP(rw http.ResponseWriter, r *http.Request) {
	b, _ := io.ReadAll(r.Body)
	var body map[string]any
	_ = json.Unmarshal(b, &body)
	w.mu.Lock()
	w.bodies = append(w.bodies, body)
	w.mu.Unlock()
	rw.WriteHeader(http.StatusOK)
}

func (w *webhookReceiver) count() int {
	w.mu.Lock()
	defer w.mu.Unlock()
	return len(w.bodies)
}

// TestJobReconcile runs the internal API on two instances over one Postgres, as prod runs after a scale-down or deploy.
func TestJobReconcile(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}
	ctx := context.Background()
	pool := pgtest.Connect(t)(t)
	mdb := mongotest.Connect(t)(t)
	acRepos := lo.Must(accountmongo.New(ctx, mdb.Client(), mdb.Name(), false, false, nil))
	repos := lo.Must(postgres.New(ctx, pool, acRepos))
	live := postgres.NewLiveStore(pool)

	writer := user.New().NewID().Name("writer").Email("writer@local.test").Workspace(workspace.NewID()).MustBuild()
	require.NoError(t, acRepos.User.Save(ctx, writer))
	ws := workspace.New().NewID().Name("team").Alias("team").Members(map[user.ID]workspace.Member{
		writer.ID(): {Role: workspace.RoleWriter},
	}).MustBuild()
	require.NoError(t, acRepos.Workspace.Save(ctx, ws))
	wid := accountsid.MustWorkspaceID(ws.ID().String())

	hook := &webhookReceiver{}
	hookSrv := httptest.NewServer(hook)
	t.Cleanup(hookSrv.Close)

	// Each call builds a separate server instance: its own interactor, locks and loops.
	newInstance := func() flowv1.ReEarthFlowClient {
		srv := app.NewInternalAPIServer(&app.ServerConfig{
			Config:       &config.Config{InternalAPI_Token: internalAPIToken},
			Repos:        repos,
			AccountRepos: acRepos,
			Gateways:     &gateway.Container{File: initGateway().File, LiveStore: live},
		})
		l, err := net.Listen("tcp", "127.0.0.1:0")
		require.NoError(t, err)
		go func() { _ = srv.Serve(l) }()
		t.Cleanup(srv.Stop)
		conn, err := grpc.NewClient(l.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
		require.NoError(t, err)
		t.Cleanup(func() { _ = conn.Close() })
		return flowv1.NewReEarthFlowClient(conn)
	}

	putEvent := func(jid id.JobID, result string, at time.Time) {
		b, _ := json.Marshal(gateway.JobCompleteEvent{Timestamp: at, JobID: jid.String(), WorkflowID: "wf", Result: result})
		_, err := pool.Exec(ctx, `INSERT INTO job_complete_events (job_id, data) VALUES ($1, $2)`, jid.String(), string(b))
		require.NoError(t, err)
	}
	eventsLeft := func(jid id.JobID) int {
		var n int
		require.NoError(t, pool.QueryRow(ctx, `SELECT count(*) FROM job_complete_events WHERE job_id = $1`, jid.String()).Scan(&n))
		return n
	}
	storedJob := func(jid id.JobID) *job.Job {
		j, err := repos.Job.FindByID(ctx, jid)
		require.NoError(t, err)
		return j
	}
	dayAgo := time.Now().Add(-24 * time.Hour).UTC().Truncate(time.Second)

	t.Run("a debug run that finished while nobody watched gets the worker's finish time", func(t *testing.T) {
		debug := true
		j := job.New().NewID().Workspace(wid).Debug(&debug).Status(job.StatusRunning).StartedAt(dayAgo.Add(-time.Hour)).MustBuild()
		require.NoError(t, repos.Job.Save(ctx, j))
		putEvent(j.ID(), "success", dayAgo)
		before, err := repos.Job.CountRunningByWorkspace(ctx, wid, time.Now().Add(-48*time.Hour))
		require.NoError(t, err)

		res, err := newInstance().GetJob(as(writer), &flowv1.GetJobRequest{JobId: j.ID().String()})

		require.NoError(t, err)
		assert.Equal(t, flowv1.JobStatus_JOB_STATUS_COMPLETED, res.GetJob().GetStatus())
		got := storedJob(j.ID())
		require.NotNil(t, got.CompletedAt())
		assert.True(t, dayAgo.Equal(*got.CompletedAt()), "completedAt %v, want %v", got.CompletedAt(), dayAgo)
		assert.Zero(t, eventsLeft(j.ID()), "the event is deleted")
		after, err := repos.Job.CountRunningByWorkspace(ctx, wid, time.Now().Add(-48*time.Hour))
		require.NoError(t, err)
		assert.Equal(t, before-1, after, "the plan-limit slot is freed")
	})

	t.Run("a trigger run's webhook is sent once while two instances race", func(t *testing.T) {
		did := id.NewDeploymentID()
		url := hookSrv.URL
		j := job.New().NewID().Workspace(wid).Deployment(&did).Status(job.StatusRunning).
			StartedAt(dayAgo.Add(-time.Hour)).NotificationURL(&url).MustBuild()
		require.NoError(t, repos.Job.Save(ctx, j))
		putEvent(j.ID(), "failed", dayAgo)
		a, b := newInstance(), newInstance()

		var wg sync.WaitGroup
		var errs atomic.Int32
		for n := range 20 {
			c := a
			if n%2 == 1 {
				c = b
			}
			wg.Go(func() {
				if _, err := c.GetJob(as(writer), &flowv1.GetJobRequest{JobId: j.ID().String()}); err != nil {
					errs.Add(1)
				}
			})
		}
		wg.Wait()

		assert.Zero(t, errs.Load())
		assert.Equal(t, 1, hook.count(), "exactly one webhook")
		got := storedJob(j.ID())
		assert.Equal(t, job.StatusFailed, got.Status())
		assert.True(t, dayAgo.Equal(*got.CompletedAt()))
		assert.NotNil(t, got.NotifiedAt())
		hook.mu.Lock()
		assert.Equal(t, "failed", hook.bodies[0]["status"])
		assert.Equal(t, j.ID().String(), hook.bodies[0]["runId"])
		hook.mu.Unlock()

		_, err := newInstance().GetJob(as(writer), &flowv1.GetJobRequest{JobId: j.ID().String()})
		require.NoError(t, err)
		assert.Equal(t, 1, hook.count(), "a later instance sends no second webhook")
	})

	t.Run("a run with no event yet stays running", func(t *testing.T) {
		debug := true
		j := job.New().NewID().Workspace(wid).Debug(&debug).Status(job.StatusRunning).StartedAt(time.Now()).MustBuild()
		require.NoError(t, repos.Job.Save(ctx, j))

		res, err := newInstance().GetJob(as(writer), &flowv1.GetJobRequest{JobId: j.ID().String()})

		require.NoError(t, err)
		assert.Equal(t, flowv1.JobStatus_JOB_STATUS_RUNNING, res.GetJob().GetStatus())
		assert.Nil(t, storedJob(j.ID()).CompletedAt())
	})
}
