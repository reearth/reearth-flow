package interactor

import (
	"context"
	"sync"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// countingJobRepo wraps a real repo.Job and counts calls to FindByIDs, so
// tests can assert a batch of N jobs triggers exactly one lookup call.
type countingJobRepo struct {
	repo.Job
	mu            sync.Mutex
	findByIDsHits int
}

func (r *countingJobRepo) FindByIDs(ctx context.Context, ids id.JobIDList) ([]*job.Job, error) {
	r.mu.Lock()
	r.findByIDsHits++
	r.mu.Unlock()
	return r.Job.FindByIDs(ctx, ids)
}

// countingRedisGateway records which job IDs GetLogs was called for, so tests
// can assert no Redis call is made on behalf of a denied job.
type countingRedisGateway struct {
	calls []id.JobID
	mu    sync.Mutex
}

func (m *countingRedisGateway) GetLogs(_ context.Context, _ time.Time, _ time.Time, jobID id.JobID) ([]*log.Log, error) {
	m.mu.Lock()
	m.calls = append(m.calls, jobID)
	m.mu.Unlock()
	return []*log.Log{log.NewLog(jobID, nil, time.Now().UTC(), log.LevelInfo, "hi")}, nil
}

func (m *countingRedisGateway) GetNodeExecution(_ context.Context, _ id.JobID, _ string) (*graph.NodeExecution, error) {
	panic("unimplemented")
}

func (m *countingRedisGateway) GetNodeExecutions(_ context.Context, _ id.JobID) ([]*graph.NodeExecution, error) {
	panic("unimplemented")
}

func (m *countingRedisGateway) GetUserFacingLogs(_ context.Context, _ time.Time, _ time.Time, _ id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	return nil, nil
}

func (m *countingRedisGateway) GetJobCompleteEvent(_ context.Context, _ id.JobID) (*gateway.JobCompleteEvent, error) {
	return nil, nil
}

func (m *countingRedisGateway) DeleteJobCompleteEvent(_ context.Context, _ id.JobID) error {
	return nil
}

func newJobIn(t *testing.T, jobRepo repo.Job, ws accountsid.WorkspaceID) id.JobID {
	t.Helper()
	j := job.NewJob(id.NewJobID(), nil, ws, "gcp-job")
	require.NoError(t, jobRepo.Save(context.Background(), j))
	return j.ID()
}

func TestLogInteractor_GetLogsBatch_OneJobLookupOnePermissionCallPerWorkspace(t *testing.T) {
	ws := accountsid.NewWorkspaceID()
	jobRepo := &countingJobRepo{Job: memory.NewJob()}
	redis := &countingRedisGateway{}

	var jobIDs []id.JobID
	for i := 0; i < 3; i++ {
		jobIDs = append(jobIDs, newJobIn(t, jobRepo, ws))
	}

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{ws: true}}
	li := &LogInteractor{jobRepo: jobRepo, logsGatewayRedis: redis, permissionChecker: checker}

	res, err := li.GetLogsBatch(context.Background(), time.Now().Add(-time.Hour), jobIDs)
	require.NoError(t, err)

	assert.Equal(t, 1, jobRepo.findByIDsHits, "3 jobs should cost one FindByIDs call, not three")
	assert.Equal(t, 1, checker.calls, "one workspace should cost one permission check, not three")
	require.Len(t, res, 3)
	for _, jid := range jobIDs {
		assert.Contains(t, res, jid)
	}
}

func TestLogInteractor_GetLogsBatch_DeniedWorkspaceOmittedNoRedisCall(t *testing.T) {
	wsAllowed := accountsid.NewWorkspaceID()
	wsDenied := accountsid.NewWorkspaceID()
	jobRepo := &countingJobRepo{Job: memory.NewJob()}
	redis := &countingRedisGateway{}

	allowedJob := newJobIn(t, jobRepo, wsAllowed)
	deniedJob := newJobIn(t, jobRepo, wsDenied)

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{wsAllowed: true, wsDenied: false}}
	li := &LogInteractor{jobRepo: jobRepo, logsGatewayRedis: redis, permissionChecker: checker}

	res, err := li.GetLogsBatch(context.Background(), time.Now().Add(-time.Hour), []id.JobID{allowedJob, deniedJob})
	require.NoError(t, err)

	assert.Contains(t, res, allowedJob)
	assert.NotContains(t, res, deniedJob, "caller was denied on this workspace, its logs must not leak through the batch")

	redis.mu.Lock()
	defer redis.mu.Unlock()
	assert.Equal(t, []id.JobID{allowedJob}, redis.calls, "no Redis call should be made for a job the caller can't see")
}

func TestLogInteractor_GetLogsBatch_EmptyBatch_DeniesAndTouchesNothing(t *testing.T) {
	jobRepo := &countingJobRepo{Job: memory.NewJob()}
	redis := &countingRedisGateway{}
	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{}}
	li := &LogInteractor{jobRepo: jobRepo, logsGatewayRedis: redis, permissionChecker: checker}

	res, err := li.GetLogsBatch(context.Background(), time.Now(), nil)

	require.Error(t, err, "an empty batch must not silently pass through as an empty allow")
	assert.Nil(t, res)
	assert.Equal(t, 0, jobRepo.findByIDsHits, "no repo call should happen for a denied empty batch")
	assert.Empty(t, redis.calls)
}
