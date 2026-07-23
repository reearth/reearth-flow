package redis

import (
	"context"
	"encoding/json"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

// nodeStatusTerminalFixturePath is shared, by design, with the api side
// (api/internal/infrastructure/redis/node_test.go): it pins that
// `NodeMetrics` fields survive subscriber -> Redis -> api unchanged.
const nodeStatusTerminalFixturePath = "../../../../testdata/node/node_status_terminal.json"

func TestRedisStorage_SaveNodeEventToRedis_WithMetrics_MatchesSharedFixture(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	jobID := "22222222-2222-2222-2222-222222222222"
	nodeID := "44444444-4444-4444-4444-444444444444"
	workflowID := "11111111-1111-1111-1111-111111111111"
	ts := time.Date(2026, 7, 16, 9, 31, 10, 0, time.UTC)

	event := &node.NodeStatusEvent{
		WorkflowID: workflowID,
		JobID:      jobID,
		NodeID:     nodeID,
		Status:     node.StatusCompleted,
		Timestamp:  ts,
		Metrics: &node.NodeMetrics{
			FeaturesProcessed:  42,
			FeaturesWritten:    0,
			FinishFeatureCount: 7,
		},
	}

	jobNodesKey := "nodeEvents:" + jobID
	nodeKey := "node:" + jobID + ":" + nodeID

	mClient.On("LPush", mock.Anything, jobNodesKey, mock.Anything).Return(nil)
	mClient.On("Expire", mock.Anything, jobNodesKey, 12*time.Hour).Return(nil)
	mClient.On("Set", mock.Anything, nodeKey, mock.Anything, 12*time.Hour).Return(nil)

	err := rStorage.SaveNodeEventToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)

	// Compare the individual node key's Set payload against the shared
	// fixture (the wire shape api's NodeEntry unmarshals from).
	var setCall *mock.Call
	for i, c := range mClient.Calls {
		if c.Method == "Set" {
			setCall = &mClient.Calls[i]
		}
	}
	if !assert.NotNil(t, setCall, "expected a Set call for the individual node key") {
		return
	}
	writtenJSON, ok := setCall.Arguments[2].(string)
	if !assert.True(t, ok, "Set's value argument must be a JSON string") {
		return
	}

	var written map[string]interface{}
	assert.NoError(t, json.Unmarshal([]byte(writtenJSON), &written))

	fixtureBytes, err := os.ReadFile(nodeStatusTerminalFixturePath)
	assert.NoError(t, err)
	var fixture map[string]interface{}
	assert.NoError(t, json.Unmarshal(fixtureBytes, &fixture))

	assert.Equal(t, fixture, written)
}
