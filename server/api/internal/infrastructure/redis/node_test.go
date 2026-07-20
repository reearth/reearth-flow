package redis_test

import (
	"context"
	"os"
	"testing"

	"github.com/go-redis/redismock/v9"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// nodeStatusTerminalFixturePath is the SHARED fixture (duplicated, by
// design, on the subscriber side at
// subscriber/internal/infrastructure/redis/node_test.go) proving the exact
// flat JSON blob `SaveNodeEventToRedis` writes to the individual
// `node:{jobId}:{nodeId}` Redis key is the same shape `NodeEntry` reads back
// here — i.e. that `NodeMetrics` fields survive subscriber -> Redis -> api.
const nodeStatusTerminalFixturePath = "../../../../testdata/node/node_status_terminal.json"

func TestGetNodeExecution_MetricsSurviveSharedFixtureRoundTrip(t *testing.T) {
	ctx := context.Background()

	fixture, err := os.ReadFile(nodeStatusTerminalFixturePath)
	require.NoError(t, err)

	jobID := id.MustJobID("22222222-2222-2222-2222-222222222222")
	nodeID := "44444444-4444-4444-4444-444444444444"
	key := "node:" + jobID.String() + ":" + nodeID

	client, mock := redismock.NewClientMock()
	mock.ExpectGet(key).SetVal(string(fixture))

	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	nodeExec, err := r.GetNodeExecution(ctx, jobID, nodeID)
	require.NoError(t, err)
	require.NotNil(t, nodeExec)

	assert.Equal(t, jobID, nodeExec.JobID())
	assert.Equal(t, "COMPLETED", string(nodeExec.Status()))

	require.NotNil(t, nodeExec.FeaturesProcessed())
	assert.Equal(t, 42, *nodeExec.FeaturesProcessed())
	require.NotNil(t, nodeExec.FeaturesWritten())
	assert.Equal(t, 0, *nodeExec.FeaturesWritten())
	require.NotNil(t, nodeExec.FinishFeatureCount())
	assert.Equal(t, 7, *nodeExec.FinishFeatureCount())

	assert.NoError(t, mock.ExpectationsWereMet())
}
