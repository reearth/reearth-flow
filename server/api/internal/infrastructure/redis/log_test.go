package redis_test

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/go-redis/redismock/v9"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewRedisLog(t *testing.T) {
	t.Run("nil client", func(t *testing.T) {
		r, err := redis.NewRedisLog(nil)
		assert.Nil(t, r)
		assert.EqualError(t, err, "client is nil")
	})

	t.Run("valid client", func(t *testing.T) {
		client, _ := redismock.NewClientMock()
		r, err := redis.NewRedisLog(client)
		require.NoError(t, err)
		assert.NotNil(t, r)
	})
}

func TestToLogEntry(t *testing.T) {
	t.Run("valid log", func(t *testing.T) {
		jid := id.NewJobID()
		nid := id.NewNodeID()
		now := time.Now()

		l := log.NewLog(
			jid,
			&nid,
			now,
			log.LevelInfo,
			"test message",
		)

		entry := redis.ToLogEntry(l)
		require.NotNil(t, entry)
		assert.Equal(t, jid.String(), entry.JobID)
		assert.NotNil(t, entry.NodeID)
		assert.Equal(t, nid.String(), *entry.NodeID)

		// Converted to UTC on the ToLogEntry side
		assert.Equal(t, now.UTC(), entry.Timestamp)
		assert.Equal(t, log.LevelInfo, entry.LogLevel)
		assert.Equal(t, "test message", entry.Message)
	})

	t.Run("nil log", func(t *testing.T) {
		entry := redis.ToLogEntry(nil)
		assert.Nil(t, entry)
	})
}

func TestLogEntry_ToDomain(t *testing.T) {
	t.Run("valid entry", func(t *testing.T) {
		wid := id.NewWorkflowID().String()
		jid := id.NewJobID().String()
		nid := id.NewNodeID().String()
		now := time.Now()

		entry := &redis.LogEntry{
			WorkflowID: wid,
			JobID:      jid,
			NodeID:     &nid,
			Timestamp:  now,
			LogLevel:   log.LevelInfo,
			Message:    "hello",
		}

		dl, err := entry.ToDomain()
		require.NoError(t, err)
		assert.Equal(t, jid, dl.JobID().String())
		assert.Equal(t, nid, dl.NodeID().String())

		// Convert to UTC on the ToDomain side
		assert.Equal(t, now.UTC(), dl.Timestamp())
		assert.Equal(t, log.LevelInfo, dl.Level())
		assert.Equal(t, "hello", dl.Message())
	})

	t.Run("invalid job id", func(t *testing.T) {
		entry := &redis.LogEntry{
			WorkflowID: id.NewWorkflowID().String(),
			JobID:      "invalid",
		}
		dl, err := entry.ToDomain()
		assert.Nil(t, dl)
		assert.Error(t, err)
	})

	t.Run("invalid node id", func(t *testing.T) {
		invalidNodeID := "invalid_node"
		entry := &redis.LogEntry{
			WorkflowID: id.NewWorkflowID().String(),
			JobID:      id.NewJobID().String(),
			NodeID:     &invalidNodeID,
		}
		dl, err := entry.ToDomain()
		assert.NotNil(t, dl)
		assert.Nil(t, dl.NodeID())
		assert.NoError(t, err)
	})
}

func TestRedisLog_GetLogs(t *testing.T) {
	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)
	require.NotNil(t, r)

	ctx := context.Background()
	wid := id.NewWorkflowID()
	jid := id.NewJobID()

	now := time.Now().UTC()
	since := now.Add(-1 * time.Hour) // 1 hour ago
	until := now                     // Set the current time as until

	scanKey1 := "log:" + wid.String() + ":" + jid.String() + ":key1"
	scanKey2 := "log:" + wid.String() + ":" + jid.String() + ":key2"

	// Key 1: A valid JSON string
	entry1 := redis.LogEntry{
		WorkflowID: wid.String(),
		JobID:      jid.String(),
		Timestamp:  now,
		LogLevel:   log.LevelInfo,
		Message:    "test1",
	}
	bytes1, _ := json.Marshal(entry1)

	// Key 2: JSON is corrupted
	brokenJSON := `{"jobId":?????`

	// 1st SCAN
	mock.ExpectScan(uint64(0), "log:*:"+jid.String()+":*", int64(100)).
		SetVal([]string{scanKey1, scanKey2}, 999)

	// Next key 1 to get (JSON string)
	mock.ExpectGet(scanKey1).SetVal(string(bytes1))

	// Next key 2 (broken JSON)
	mock.ExpectGet(scanKey2).SetVal(brokenJSON)

	// 2nd SCAN -> no more keys
	mock.ExpectScan(uint64(999), "log:*:"+jid.String()+":*", int64(100)).
		SetVal([]string{}, 0)

	// Test execution: Pass the since & until that you determined first.
	result, getErr := r.GetLogs(ctx, since, until, jid)
	require.NoError(t, getErr)

	// Assert
	assert.Len(t, result, 1, "Only one valid log, excluding corrupted JSON.")
	if len(result) == 1 {
		assert.Equal(t, entry1.Message, result[0].Message())
	}
}

func TestRedisLog_GetLogs_Empty(t *testing.T) {
	// No keys at all
	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)
	require.NotNil(t, r)

	ctx := context.Background()
	jid := id.NewJobID()
	since := time.Now().Add(-1 * time.Hour)
	until := time.Now().UTC()

	// SCAN returns no keys
	mock.ExpectScan(uint64(0), "log:*:"+jid.String()+":*", int64(100)).
		SetVal([]string{}, 0)

	result, getErr := r.GetLogs(ctx, since, until, jid)
	require.NoError(t, getErr)
	assert.Empty(t, result)
}

func TestRedisLog_GetLogs_OldData(t *testing.T) {
	// Logs older than since are rejected
	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)
	require.NotNil(t, r)

	ctx := context.Background()
	wid := id.NewWorkflowID()
	jid := id.NewJobID()
	// Do not retrieve logs prior to since
	since := time.Now().UTC()
	until := since.Add(1 * time.Hour) // For testing purposes, set "until" to 1 hour later

	scanKey := "log:" + wid.String() + ":" + jid.String() + ":key1"

	mock.ExpectScan(uint64(0), "log:*:"+jid.String()+":*", int64(100)).
		SetVal([]string{scanKey}, 0)

	entry := redis.LogEntry{
		WorkflowID: wid.String(),
		JobID:      jid.String(),
		// 1 minute before since
		Timestamp: since.Add(-1 * time.Minute),
		LogLevel:  log.LevelInfo,
		Message:   "old log",
	}
	bytes, _ := json.Marshal(entry)
	mock.ExpectGet(scanKey).SetVal(string(bytes))

	result, getErr := r.GetLogs(ctx, since, until, jid)
	require.NoError(t, getErr)
	assert.Empty(t, result, "It's older than since so it shouldn't be returned.")
}

func TestRedisLog_GetLogs_Error(t *testing.T) {
	// Test if an error occurs in Scan
	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)
	require.NotNil(t, r)

	ctx := context.Background()
	jid := id.NewJobID()

	mock.ExpectScan(uint64(0), "log:*:"+jid.String()+":*", int64(100)).
		SetErr(errors.New("scan error"))

	result, getErr := r.GetLogs(ctx, time.Now().UTC(), time.Now().UTC(), jid)
	assert.Nil(t, result)
	assert.EqualError(t, getErr, "failed to scan redis keys: scan error")
}

func TestRedisLog_GetLogs_Boundary(t *testing.T) {
	// Test what happens if the log is just since, just until, or after until
	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)
	require.NotNil(t, r)

	ctx := context.Background()
	wid := id.NewWorkflowID()
	jid := id.NewJobID()

	// First, determine the base time
	now := time.Now().UTC().Truncate(time.Second)
	since := now
	until := now.Add(10 * time.Second) // Set since+10 seconds as until

	scanKey1 := "log:" + wid.String() + ":" + jid.String() + ":key1"
	scanKey2 := "log:" + wid.String() + ":" + jid.String() + ":key2"
	scanKey3 := "log:" + wid.String() + ":" + jid.String() + ":key3"

	// key1: just since
	entry1 := redis.LogEntry{
		WorkflowID: wid.String(),
		JobID:      jid.String(),
		Timestamp:  since, // Just like since
		LogLevel:   log.LevelInfo,
		Message:    "at since",
	}
	bytes1, _ := json.Marshal(entry1)

	// key2: just until
	entry2 := redis.LogEntry{
		WorkflowID: wid.String(),
		JobID:      jid.String(),
		Timestamp:  until, // Just like since
		LogLevel:   log.LevelInfo,
		Message:    "at until",
	}
	bytes2, _ := json.Marshal(entry2)

	// key3: after until
	entry3 := redis.LogEntry{
		WorkflowID: wid.String(),
		JobID:      jid.String(),
		// 1 second after until
		Timestamp: until.Add(1 * time.Second),
		LogLevel:  log.LevelInfo,
		Message:   "after until",
	}
	bytes3, _ := json.Marshal(entry3)

	mock.ExpectScan(uint64(0), "log:*:"+jid.String()+":*", int64(100)).
		SetVal([]string{scanKey1, scanKey2, scanKey3}, 0)

	mock.ExpectGet(scanKey1).SetVal(string(bytes1))
	mock.ExpectGet(scanKey2).SetVal(string(bytes2))
	mock.ExpectGet(scanKey3).SetVal(string(bytes3))

	// execution
	result, err := r.GetLogs(ctx, since, until, jid)
	require.NoError(t, err)
	assert.Len(t, result, 2, "Only two matches should be included: since and until")

	foundMsgs := make(map[string]bool)
	for _, dl := range result {
		foundMsgs[dl.Message()] = true
	}

	assert.True(t, foundMsgs["at since"])
	assert.True(t, foundMsgs["at until"])
	assert.False(t, foundMsgs["after until"], "Not included because it is after until")
}
