package gql

import (
	"context"
	"sync"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
)

// countingDeploymentUsecase counts calls to FindByProjects; other methods panic
// if called, since these tests only exercise the batched path.
type countingDeploymentUsecase struct {
	interfaces.Deployment
	result map[id.ProjectID]*deployment.Deployment
	err    error
	mu     sync.Mutex
	calls  int
}

func (c *countingDeploymentUsecase) FindByProjects(_ context.Context, ids []id.ProjectID) (map[id.ProjectID]*deployment.Deployment, error) {
	c.mu.Lock()
	c.calls++
	c.mu.Unlock()
	return c.result, c.err
}

func TestDeploymentLoader_FetchByProjects_NKeysOneUsecaseCall(t *testing.T) {
	pid1, pid2, pid3 := id.NewProjectID(), id.NewProjectID(), id.NewProjectID()
	dep1 := deployment.New().NewID().Workspace(accountsid.NewWorkspaceID()).Project(&pid1).IsHead(true).MustBuild()

	fake := &countingDeploymentUsecase{result: map[id.ProjectID]*deployment.Deployment{pid1: dep1}}
	loader := NewDeploymentLoader(fake)

	keys := []gqlmodel.ID{
		gqlmodel.ID(pid1.String()),
		gqlmodel.ID(pid2.String()),
		gqlmodel.ID(pid3.String()),
	}
	res, errs := loader.FetchByProjects(context.Background(), keys)

	require.Empty(t, errs)
	require.Len(t, res, 3)
	assert.Equal(t, 1, fake.calls, "3 parents in one batch should cost exactly one usecase call")
	assert.NotNil(t, res[0], "pid1 has a deployment")
	assert.Nil(t, res[1], "pid2 has no deployment: nil, not an error, and keeps position aligned")
	assert.Nil(t, res[2], "pid3 has no deployment: nil, not an error, and keeps position aligned")
}

func TestDeploymentByProjectDataLoader_ConcurrentLoadsCoalesceIntoOneCall(t *testing.T) {
	pid1, pid2, pid3 := id.NewProjectID(), id.NewProjectID(), id.NewProjectID()
	dep1 := deployment.New().NewID().Workspace(accountsid.NewWorkspaceID()).Project(&pid1).IsHead(true).MustBuild()

	fake := &countingDeploymentUsecase{result: map[id.ProjectID]*deployment.Deployment{pid1: dep1}}
	loader := NewDeploymentLoader(fake).ByProjectDataLoader(context.Background())

	var wg sync.WaitGroup
	keys := []gqlmodel.ID{gqlmodel.ID(pid1.String()), gqlmodel.ID(pid2.String()), gqlmodel.ID(pid3.String())}
	results := make([]*gqlmodel.Deployment, len(keys))
	for i, k := range keys {
		wg.Add(1)
		go func(i int, k gqlmodel.ID) {
			defer wg.Done()
			d, err := loader.Load(k)
			assert.NoError(t, err)
			results[i] = d
		}(i, k)
	}
	wg.Wait()

	fake.mu.Lock()
	defer fake.mu.Unlock()
	assert.Equal(t, 1, fake.calls, "3 concurrent Load calls for 3 sibling Project.deployment fields should hit the usecase once")
	assert.NotNil(t, results[0])
	assert.Nil(t, results[1])
	assert.Nil(t, results[2])
}

// countingParameterUsecase counts calls to FetchByProjects.
type countingParameterUsecase struct {
	interfaces.Parameter
	result map[id.ProjectID]*parameter.ParameterList
	err    error
	mu     sync.Mutex
	calls  int
}

func (c *countingParameterUsecase) FetchByProjects(_ context.Context, ids []id.ProjectID) (map[id.ProjectID]*parameter.ParameterList, error) {
	c.mu.Lock()
	c.calls++
	c.mu.Unlock()
	return c.result, c.err
}

func TestParameterLoader_FetchByProjects_NKeysOneUsecaseCall(t *testing.T) {
	pid1, pid2, pid3 := id.NewProjectID(), id.NewProjectID(), id.NewProjectID()
	param, err := parameter.New().ProjectID(pid1).Name("p").Type(parameter.TypeText).Build()
	require.NoError(t, err)
	list := parameter.NewParameterList([]*parameter.Parameter{param})

	fake := &countingParameterUsecase{result: map[id.ProjectID]*parameter.ParameterList{pid1: list}}
	loader := NewParameterLoader(fake)

	keys := []gqlmodel.ID{
		gqlmodel.ID(pid1.String()),
		gqlmodel.ID(pid2.String()),
		gqlmodel.ID(pid3.String()),
	}
	res, errs := loader.FetchByProjects(context.Background(), keys)

	require.Empty(t, errs)
	require.Len(t, res, 3)
	assert.Equal(t, 1, fake.calls, "3 parents in one batch should cost exactly one usecase call")
	assert.Len(t, res[0], 1, "pid1 has one parameter")
	assert.Nil(t, res[1], "pid2 has no parameters: nil, not an error, and keeps position aligned")
	assert.Nil(t, res[2], "pid3 has no parameters: nil, not an error, and keeps position aligned")
}

func TestLogLoader_FetchByJobs_NKeysOneUsecaseCall(t *testing.T) {
	jid1, jid2, jid3 := id.NewJobID(), id.NewJobID(), id.NewJobID()
	// LogsBatchKey round-trips since through UTC/RFC3339Nano; compare against
	// the same normalized value the mock will actually observe.
	since := time.Now().Add(-time.Hour).UTC()
	logs1 := []*log.Log{log.NewLog(jid1, nil, time.Now().UTC(), log.LevelInfo, "hi")}

	wantJobIDs := map[id.JobID]bool{jid1: true, jid2: true, jid3: true}
	fake := &MockLogUsecase{}
	fake.On("GetLogsBatch", context.Background(), since, mock.MatchedBy(func(jobIDs []id.JobID) bool {
		if len(jobIDs) != len(wantJobIDs) {
			return false
		}
		for _, j := range jobIDs {
			if !wantJobIDs[j] {
				return false
			}
		}
		return true
	})).Return(map[id.JobID][]*log.Log{jid1: logs1}, nil)
	loader := NewLogLoader(fake)

	keys := []string{
		LogsBatchKey(gqlmodel.ID(jid1.String()), since),
		LogsBatchKey(gqlmodel.ID(jid2.String()), since),
		LogsBatchKey(gqlmodel.ID(jid3.String()), since),
	}
	res, errs := loader.FetchByJobs(context.Background(), keys)

	require.Empty(t, errs)
	require.Len(t, res, 3)
	fake.AssertNumberOfCalls(t, "GetLogsBatch", 1)
	assert.Len(t, res[0], 1, "jid1 has one log")
	assert.Nil(t, res[1], "jid2 has no logs: nil, not an error, and keeps position aligned")
	assert.Nil(t, res[2], "jid3 has no logs: nil, not an error, and keeps position aligned")
}
