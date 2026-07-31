package interactor

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
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
