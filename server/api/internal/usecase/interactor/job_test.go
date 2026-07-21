package interactor

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/url"
	"os"
	"sync"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/job/monitor"
	pkglog "github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/notification"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// jobCompleteDiagnosticsFixturePath is the shared wire-shape fixture also
// used by gateway/job_complete_event_test.go and the subscriber module.
const jobCompleteDiagnosticsFixturePath = "../../../../testdata/diagnostics/job_complete_with_diagnostics.json"

func loadJobCompleteEventFixture(t *testing.T) gateway.JobCompleteEvent {
	t.Helper()
	raw, err := os.ReadFile(jobCompleteDiagnosticsFixturePath)
	require.NoError(t, err)
	var event gateway.JobCompleteEvent
	require.NoError(t, json.Unmarshal(raw, &event))
	return event
}

// mockCheckStatusRedis is a dedicated gateway.Redis fake: returns a
// configurable JobCompleteEvent and tracks DeleteJobCompleteEvent calls so
// tests can assert whether the event survives a persist failure.
type mockCheckStatusRedis struct {
	event       *gateway.JobCompleteEvent
	deleteErr   error
	deleteCalls int
}

func (m *mockCheckStatusRedis) GetLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*pkglog.Log, error) {
	return nil, nil
}

func (m *mockCheckStatusRedis) GetUserFacingLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	return nil, nil
}

func (m *mockCheckStatusRedis) GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error) {
	return nil, nil
}

func (m *mockCheckStatusRedis) GetNodeExecution(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	return nil, nil
}

func (m *mockCheckStatusRedis) GetJobCompleteEvent(ctx context.Context, jobID id.JobID) (*gateway.JobCompleteEvent, error) {
	return m.event, nil
}

func (m *mockCheckStatusRedis) DeleteJobCompleteEvent(ctx context.Context, jobID id.JobID) error {
	m.deleteCalls++
	return m.deleteErr
}

func (m *mockCheckStatusRedis) GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	return nil, nil
}

func (m *mockCheckStatusRedis) GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	return nil, nil
}

// mockCheckStatusJobRepo is a dedicated repo.Job fake with a working Save
// (the shared mockJobRepo panics on Save) so checkJobStatus's save calls
// can be counted and inspected.
type mockCheckStatusJobRepo struct {
	job       *job.Job
	saveCalls int
}

func (m *mockCheckStatusJobRepo) FindByID(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	return m.job, nil
}

func (m *mockCheckStatusJobRepo) FindByIDs(ctx context.Context, jobIDs id.JobIDList) ([]*job.Job, error) {
	return nil, nil
}

func (m *mockCheckStatusJobRepo) FindByWorkspace(ctx context.Context, workspaceID accountsid.WorkspaceID, p *interfaces.PaginationParam, keyword *string) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	return nil, nil, nil
}

func (m *mockCheckStatusJobRepo) FindByProject(ctx context.Context, projectID id.ProjectID) ([]*job.Job, error) {
	return nil, nil
}

func (m *mockCheckStatusJobRepo) Save(ctx context.Context, j *job.Job) error {
	m.saveCalls++
	m.job = j
	return nil
}

func (m *mockCheckStatusJobRepo) Filtered(filter repo.WorkspaceFilter) repo.Job {
	return m
}

func (m *mockCheckStatusJobRepo) Remove(ctx context.Context, jobID id.JobID) error {
	return nil
}

func (m *mockCheckStatusJobRepo) RemoveByProject(ctx context.Context, projectID id.ProjectID) error {
	return nil
}

// mockCheckStatusFile is a no-op gateway.File fake: checkJobStatus's terminal
// path calls a handful of its methods, and zero values keep every call
// harmless since no test asserts on the resulting job fields.
type mockCheckStatusFile struct{}

func (m *mockCheckStatusFile) ReadAsset(context.Context, string) (io.ReadCloser, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) ReadActions(context.Context, string) (io.ReadCloser, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) UploadAsset(context.Context, *file.File) (*url.URL, int64, error) {
	return nil, 0, nil
}
func (m *mockCheckStatusFile) DeleteAsset(context.Context, *url.URL) error { return nil }
func (m *mockCheckStatusFile) ReadWorkflow(context.Context, string) (io.ReadCloser, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) UploadWorkflow(context.Context, *file.File) (*url.URL, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) RemoveWorkflow(context.Context, *url.URL) error { return nil }
func (m *mockCheckStatusFile) ReadMetadata(context.Context, string) (io.ReadCloser, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) UploadMetadata(context.Context, string, []string) (*url.URL, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) RemoveMetadata(context.Context, *url.URL) error { return nil }
func (m *mockCheckStatusFile) ReadArtifact(context.Context, string) (io.ReadCloser, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) ListJobArtifacts(context.Context, string) ([]string, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) GetJobLogURL(string) string { return "" }
func (m *mockCheckStatusFile) CheckJobLogExists(context.Context, string) (bool, error) {
	return false, nil
}
func (m *mockCheckStatusFile) GetJobWorkerLogURL(string) string { return "" }
func (m *mockCheckStatusFile) CheckJobWorkerLogExists(context.Context, string) (bool, error) {
	return false, nil
}
func (m *mockCheckStatusFile) GetJobUserFacingLogURL(string) string { return "" }
func (m *mockCheckStatusFile) CheckJobUserFacingLogExists(context.Context, string) (bool, error) {
	return false, nil
}
func (m *mockCheckStatusFile) GetJobPreviewSchemaURL(string) string       { return "" }
func (m *mockCheckStatusFile) GetJobPreviewSchemaUploadURI(string) string { return "" }
func (m *mockCheckStatusFile) CheckJobPreviewSchemaExists(context.Context, string) (bool, error) {
	return false, nil
}
func (m *mockCheckStatusFile) GetIntermediateDataURL(context.Context, string, string) string {
	return ""
}
func (m *mockCheckStatusFile) CheckIntermediateDataExists(context.Context, string, string) (bool, error) {
	return false, nil
}
func (m *mockCheckStatusFile) IssueUploadAssetLink(context.Context, gateway.IssueUploadAssetParam) (*gateway.UploadAssetLink, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) GetPublicAssetURL(string, string) (*url.URL, error) { return nil, nil }
func (m *mockCheckStatusFile) UploadedAsset(context.Context, *asset.Upload) (*file.File, error) {
	return nil, nil
}
func (m *mockCheckStatusFile) WriteCancelFlag(ctx context.Context, jobID string) error { return nil }
func (m *mockCheckStatusFile) CancelFlagURI(jobID string) string                       { return "" }

// newCheckStatusJob builds a Job interactor with just enough collaborators to
// drive checkJobStatus for a Cloud-Run-style job (empty GCPJobID, so Batch
// polling is skipped and Batch/CloudRunWorker stay nil).
func newCheckStatusJob(jobRepo *mockCheckStatusJobRepo, diagRepo repo.NodeDiagnostics, redisMock *mockCheckStatusRedis) *Job {
	return &Job{
		jobRepo:             jobRepo,
		nodeDiagnosticsRepo: diagRepo,
		transaction:         usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
		file:                &mockCheckStatusFile{},
		redis:               redisMock,
		monitor:             monitor.NewMonitor(),
		subscriptions:       subscription.NewJobManager(),
		notifier:            notification.NewHTTPNotifier(),
		permissionChecker:   alwaysAllowPermissionChecker(),
		activeWatchers:      make(map[string]bool),
		jobLocks:            make(map[string]*sync.Mutex),
	}
}

func TestJob_checkJobStatus_TerminalDiagnosticsMerge(t *testing.T) {
	event := loadJobCompleteEventFixture(t)
	jobID := id.MustJobID(event.JobID)
	testJob := job.NewJob(jobID, nil, accountsid.NewWorkspaceID(), "")

	jobRepo := &mockCheckStatusJobRepo{job: testJob}
	diagRepo := &mockDiagnosticsRepo{}
	redisMock := &mockCheckStatusRedis{event: &event}

	i := newCheckStatusJob(jobRepo, diagRepo, redisMock)

	require.NoError(t, i.checkJobStatus(context.Background(), testJob))

	// Rows persisted before the event was deleted.
	assert.Equal(t, 1, diagRepo.saveCalls)
	assert.Equal(t, jobID, diagRepo.lastJobID)
	assert.Equal(t, event.WorkflowID, diagRepo.lastWorkflowID)
	require.Len(t, diagRepo.lastFailedNodes, 2)
	assert.Equal(t, "internal.invariant_violation", diagRepo.lastFailedNodes[0].Code())
	assert.Equal(t, "internal.unclassified", diagRepo.lastFailedNodes[1].Code())
	require.Len(t, diagRepo.lastAggregated, 1)
	assert.Equal(t, "gltf.zero_face_solid", diagRepo.lastAggregated[0].Code())
	require.NotNil(t, diagRepo.lastDropped)
	assert.Equal(t, uint64(2), *diagRepo.lastDropped)

	// Event deleted only after the persist succeeded.
	assert.Equal(t, 1, redisMock.deleteCalls)

	// Job merged to the terminal status the event carried.
	assert.Equal(t, job.StatusFailed, jobRepo.job.Status())
	assert.GreaterOrEqual(t, jobRepo.saveCalls, 1)
}

func TestJob_checkJobStatus_PersistFailure_RetainsEvent(t *testing.T) {
	event := loadJobCompleteEventFixture(t)
	jobID := id.MustJobID(event.JobID)
	testJob := job.NewJob(jobID, nil, accountsid.NewWorkspaceID(), "")

	jobRepo := &mockCheckStatusJobRepo{job: testJob}
	diagRepo := &mockDiagnosticsRepo{saveErr: errors.New("mongo down")}
	redisMock := &mockCheckStatusRedis{event: &event}

	i := newCheckStatusJob(jobRepo, diagRepo, redisMock)

	require.NoError(t, i.checkJobStatus(context.Background(), testJob))

	// Persist was attempted and failed.
	assert.Equal(t, 1, diagRepo.saveCalls)
	// Failure must SKIP the delete, so a later poll safely retries against
	// the same event (deterministic-ID upsert).
	assert.Equal(t, 0, redisMock.deleteCalls)

	// Terminal status merge is NOT aborted by the persist failure.
	assert.Equal(t, job.StatusFailed, jobRepo.job.Status())
	assert.GreaterOrEqual(t, jobRepo.saveCalls, 1)
}

func TestJob_checkJobStatus_OldWireEvent_Unchanged(t *testing.T) {
	// No failedNodes/aggregatedDiagnostics/droppedEventCount (an engine build
	// predating diagnostics): behavior must be identical to before — no rows
	// persisted, event still deleted.
	jobID := id.NewJobID()
	testJob := job.NewJob(jobID, nil, accountsid.NewWorkspaceID(), "")

	// Result "failed", not "success": with GCPJobID=="" batchStatus stays
	// Pending (never polled), and workerStatus=Failed alone is enough to make
	// the job terminal — keeping this test's only variable the absent
	// diagnostics fields.
	event := &gateway.JobCompleteEvent{
		Timestamp: time.Now(),
		JobID:     jobID.String(),
		Result:    "failed",
	}

	jobRepo := &mockCheckStatusJobRepo{job: testJob}
	diagRepo := &mockDiagnosticsRepo{}
	redisMock := &mockCheckStatusRedis{event: event}

	i := newCheckStatusJob(jobRepo, diagRepo, redisMock)

	require.NoError(t, i.checkJobStatus(context.Background(), testJob))

	assert.Equal(t, 0, diagRepo.saveCalls)
	assert.Equal(t, 1, redisMock.deleteCalls)
	assert.Equal(t, job.StatusFailed, jobRepo.job.Status())
}

func TestJob_persistTerminalDiagnostics(t *testing.T) {
	ctx := context.Background()
	jobID := id.NewJobID()

	t.Run("nil event is a no-op", func(t *testing.T) {
		diagRepo := &mockDiagnosticsRepo{}
		i := &Job{nodeDiagnosticsRepo: diagRepo}
		assert.NoError(t, i.persistTerminalDiagnostics(ctx, jobID, nil))
		assert.Equal(t, 0, diagRepo.saveCalls)
	})

	t.Run("old-wire event (all three fields empty) is a no-op", func(t *testing.T) {
		diagRepo := &mockDiagnosticsRepo{}
		i := &Job{nodeDiagnosticsRepo: diagRepo}
		event := &gateway.JobCompleteEvent{JobID: jobID.String(), Result: "success"}
		assert.NoError(t, i.persistTerminalDiagnostics(ctx, jobID, event))
		assert.Equal(t, 0, diagRepo.saveCalls)
	})

	t.Run("nil nodeDiagnosticsRepo is a no-op, not a crash", func(t *testing.T) {
		i := &Job{nodeDiagnosticsRepo: nil}
		event := &gateway.JobCompleteEvent{
			JobID:       jobID.String(),
			Result:      "failed",
			FailedNodes: []gateway.WireDiagnostic{{Code: "internal.unclassified", Category: "internal", Severity: "warn", Message: "x"}},
		}
		assert.NoError(t, i.persistTerminalDiagnostics(ctx, jobID, event))
	})

	t.Run("repo error propagates", func(t *testing.T) {
		diagRepo := &mockDiagnosticsRepo{saveErr: errors.New("boom")}
		i := &Job{nodeDiagnosticsRepo: diagRepo}
		event := &gateway.JobCompleteEvent{
			JobID:       jobID.String(),
			Result:      "failed",
			FailedNodes: []gateway.WireDiagnostic{{Code: "internal.unclassified", Category: "internal", Severity: "warn", Message: "x"}},
		}
		assert.Error(t, i.persistTerminalDiagnostics(ctx, jobID, event))
	})
}
