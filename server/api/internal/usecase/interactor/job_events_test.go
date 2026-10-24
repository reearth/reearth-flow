package interactor

import (
	"bytes"
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func captureLog(t *testing.T) (context.Context, *bytes.Buffer) {
	t.Helper()
	var buf bytes.Buffer
	return log.ContextWith(context.Background(), log.NewWithOutput(&buf)), &buf
}

func TestLogJobEvent(t *testing.T) {
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	did := id.NewDeploymentID()
	started := time.Now().Add(-90 * time.Second)

	j, err := job.New().
		NewID().
		Workspace(wid).
		ProjectID(&pid).
		Deployment(&did).
		Status(job.StatusPending).
		StartedAt(started).
		Build()
	assert.NoError(t, err)

	t.Run("submitted to the cloud run worker", func(t *testing.T) {
		ctx, buf := captureLog(t)
		logJobEvent(ctx, jobEventSubmitted, j, "trigger", jobTriggerDeployment)

		out := buf.String()
		for _, want := range []string{
			jobEventSubmitted,
			j.ID().String(),
			wid.String(),
			pid.String(),
			did.String(),
			`"backend": "cloudrun"`,
			`"mode": "run"`,
			`"trigger": "deployment"`,
		} {
			assert.Contains(t, out, want)
		}
	})

	t.Run("finished on batch", func(t *testing.T) {
		ctx, buf := captureLog(t)
		j.SetGCPJobID("projects/p/locations/r/jobs/x")
		completed := started.Add(90 * time.Second)
		j.SetCompletedAt(&completed)
		logJobFinished(ctx, j)

		out := buf.String()
		assert.Contains(t, out, jobEventFinished)
		assert.Contains(t, out, `"backend": "batch"`)
		assert.Contains(t, out, `"run_sec": 90`)
	})
}

func TestLogJobRejected(t *testing.T) {
	ctx, buf := captureLog(t)
	wid := accountsid.NewWorkspaceID()

	logJobRejected(ctx, wid, jobRejectedConcurrentJobs, "running", 2, "limit", "2")

	out := buf.String()
	assert.Contains(t, out, jobEventRejected)
	assert.Contains(t, out, wid.String())
	assert.Contains(t, out, `"reason": "concurrent_jobs"`)
	assert.Contains(t, out, `"running": 2`)
}

func TestFailJobLogsBatchBackend(t *testing.T) {
	ctx, buf := captureLog(t)
	j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Status(job.StatusPending).StartedAt(time.Now()).MustBuild()

	failJob(ctx, memory.NewJob(), j)

	out := buf.String()
	assert.Contains(t, out, jobEventSubmitFailed)
	assert.Contains(t, out, `"backend": "batch"`, "a failed Batch submission has no Batch job ID yet")
}

func TestJobAdmissionSaveLogsRefusal(t *testing.T) {
	ctx, buf := captureLog(t)
	ws := accountsid.NewWorkspaceID()
	l := planLimits{policyChecker: &planChecker{}, jobRepo: memory.NewJob(), now: time.Now}

	a, err := l.ensureCanRun(ctx, ws)
	require.NoError(t, err)
	b, err := l.ensureCanRun(ctx, ws)
	require.NoError(t, err)

	newJob := func() *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(time.Now()).MustBuild()
	}
	require.NoError(t, a.save(ctx, newJob()))
	assert.NotContains(t, buf.String(), jobEventRejected)

	assert.ErrorIs(t, b.save(ctx, newJob()), interfaces.ErrJobAlreadyRunning)
	out := buf.String()
	assert.Contains(t, out, jobEventRejected)
	assert.Contains(t, out, ws.String())
	assert.Contains(t, out, `"reason": "concurrent_jobs"`)
}
