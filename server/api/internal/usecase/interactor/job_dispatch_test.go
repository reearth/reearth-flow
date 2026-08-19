package interactor

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
)

// retryingTransactor runs the callback twice, the way pgxx re-runs a closure
// after a serialization failure (40001/40P01) when WithTxRetry is enabled.
//
// Anything the closure does to the outside world therefore happens twice, which
// is the whole reason cloud-job submission has to live outside it.
type retryingTransactor struct{ runs int }

func (t *retryingTransactor) WithinTransaction(ctx context.Context, fn func(ctx context.Context) error) error {
	t.runs++
	if err := fn(ctx); err != nil {
		return err
	}
	t.runs++
	return fn(ctx)
}

// A retried transaction must not dispatch a second cloud job. Before submission
// moved out of the closure this submitted twice, under two different job IDs,
// leaving the first job running with no row and no monitoring.
func TestPreviewSchema_SerializationRetryDoesNotDoubleSubmit(t *testing.T) {
	ctx := context.Background()

	projectRepo := memory.NewProject()
	jobRepo := memory.NewJob()
	ws := &fakeWebsocket{}
	ff := &previewFakeFile{}
	fj := &previewFakeJob{}
	batch := &previewFakeBatch{}
	tx := &retryingTransactor{}

	prj := newPreviewProject(t, projectRepo)

	uc := &Project{
		projectRepo:       projectRepo,
		jobRepo:           jobRepo,
		websocket:         ws,
		file:              ff,
		batch:             batch,
		cloudRunWorker:    nil, // nil => Batch dispatch path
		job:               fj,
		transaction:       tx,
		permissionChecker: NewMockPermissionChecker(nil),
	}

	got, err := uc.PreviewSchema(ctx, interfaces.PreviewSchemaParam{
		ProjectID:  prj.ID(),
		Workflow:   newWorkflowFile(),
		SampleSize: lo.ToPtr(25),
	})
	assert.NoError(t, err)
	assert.NotNil(t, got)

	// The closure really did run twice...
	assert.Equal(t, 2, tx.runs)

	// ...but exactly one probe job reached the batch gateway.
	assert.Equal(t, 1, batch.probeCalls)

	// And the workflow object was uploaded once, so the retry left no orphan.
	assert.Equal(t, 1, ff.uploadWorkflowCalls)
}

// ctxRecordingJobRepo embeds a nil repo.Job so only Save needs implementing,
// and records the ctx.Err() it observed when Save was invoked.
type ctxRecordingJobRepo struct {
	repo.Job
	saveCtxErr error
	saved      *job.Job
	saveCalled bool
}

func (r *ctxRecordingJobRepo) Save(ctx context.Context, j *job.Job) error {
	r.saveCalled = true
	r.saveCtxErr = ctx.Err()
	r.saved = j
	return nil
}

// failJob is the recovery path run right after the caller's ctx died (e.g. a
// trigger's HTTP deadline), so it must not silently drop the write by reusing
// that same, already-cancelled ctx.
func TestFailJob_DetachesFromCancelledContext(t *testing.T) {
	parentCtx, cancel := context.WithCancel(context.Background())
	cancel()
	assert.Error(t, parentCtx.Err())

	jobRepo := &ctxRecordingJobRepo{}
	j := job.New().NewID().Status(job.StatusPending).MustBuild()

	failJob(parentCtx, jobRepo, j)

	assert.True(t, jobRepo.saveCalled)
	assert.NoError(t, jobRepo.saveCtxErr)
	assert.Equal(t, job.StatusFailed, jobRepo.saved.Status())
}
