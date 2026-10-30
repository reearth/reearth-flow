package redis_test

import (
	"context"
	"encoding/json"
	"errors"
	"strconv"
	"testing"
	"time"

	"github.com/go-redis/redismock/v9"
	goredis "github.com/redis/go-redis/v9"
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

func streamStart(since time.Time) string {
	return strconv.FormatInt(since.Add(-time.Minute).UnixMilli(), 10)
}

func logMessage(t *testing.T, id string, v any) goredis.XMessage {
	t.Helper()
	b, err := json.Marshal(v)
	require.NoError(t, err)
	return goredis.XMessage{ID: id, Values: map[string]any{"data": string(b)}}
}

func TestRedisLog_GetLogs(t *testing.T) {
	t.Parallel()

	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	ctx := context.Background()
	wid := id.NewWorkflowID()
	jid := id.NewJobID()

	now := time.Now().UTC()
	since := now.Add(-1 * time.Hour)
	until := now

	entry1 := redis.LogEntry{
		WorkflowID: wid.String(),
		JobID:      jid.String(),
		Timestamp:  now,
		LogLevel:   log.LevelInfo,
		Message:    "test1",
	}

	mock.ExpectXRange("log:"+jid.String(), streamStart(since), "+").SetVal([]goredis.XMessage{
		logMessage(t, "1-0", entry1),
		{ID: "2-0", Values: map[string]any{"data": `{"jobId":?????`}},
		{ID: "3-0", Values: map[string]any{"other": "x"}},
	})

	result, err := r.GetLogs(ctx, since, until, jid)
	require.NoError(t, err)
	require.Len(t, result, 1, "only the valid entry is returned")
	assert.Equal(t, entry1.Message, result[0].Message())
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestRedisLog_GetLogs_Empty(t *testing.T) {
	t.Parallel()

	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	jid := id.NewJobID()
	since := time.Now().Add(-1 * time.Hour)

	mock.ExpectXRange("log:"+jid.String(), streamStart(since), "+").SetVal([]goredis.XMessage{})

	result, err := r.GetLogs(context.Background(), since, time.Now().UTC(), jid)
	require.NoError(t, err)
	assert.Empty(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestRedisLog_GetLogs_ZeroSince(t *testing.T) {
	t.Parallel()

	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	jid := id.NewJobID()

	mock.ExpectXRange("log:"+jid.String(), "-", "+").SetVal([]goredis.XMessage{})

	result, err := r.GetLogs(context.Background(), time.Time{}, time.Now().UTC(), jid)
	require.NoError(t, err)
	assert.Empty(t, result)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestRedisLog_GetLogs_OldData(t *testing.T) {
	t.Parallel()

	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	wid := id.NewWorkflowID()
	jid := id.NewJobID()
	since := time.Now().UTC()
	until := since.Add(1 * time.Hour)

	// Inside the clock-skew window of the XRANGE bound, but before since.
	entry := redis.LogEntry{
		WorkflowID: wid.String(),
		JobID:      jid.String(),
		Timestamp:  since.Add(-30 * time.Second),
		LogLevel:   log.LevelInfo,
		Message:    "old log",
	}
	mock.ExpectXRange("log:"+jid.String(), streamStart(since), "+").SetVal([]goredis.XMessage{
		logMessage(t, "1-0", entry),
	})

	result, err := r.GetLogs(context.Background(), since, until, jid)
	require.NoError(t, err)
	assert.Empty(t, result, "older than since, so it should not be returned")
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestRedisLog_GetLogs_Error(t *testing.T) {
	t.Parallel()

	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	jid := id.NewJobID()
	since := time.Now().UTC()

	mock.ExpectXRange("log:"+jid.String(), streamStart(since), "+").SetErr(errors.New("xrange error"))

	result, err := r.GetLogs(context.Background(), since, since, jid)
	assert.Nil(t, result)
	assert.EqualError(t, err, "failed to xrange redis stream=log:"+jid.String()+": xrange error")
}

func TestRedisLog_GetLogs_Boundary(t *testing.T) {
	t.Parallel()

	client, mock := redismock.NewClientMock()
	r, err := redis.NewRedisLog(client)
	require.NoError(t, err)

	wid := id.NewWorkflowID()
	jid := id.NewJobID()

	now := time.Now().UTC().Truncate(time.Second)
	since := now
	until := now.Add(10 * time.Second)

	newEntry := func(ts time.Time, msg string) redis.LogEntry {
		return redis.LogEntry{
			WorkflowID: wid.String(),
			JobID:      jid.String(),
			Timestamp:  ts,
			LogLevel:   log.LevelInfo,
			Message:    msg,
		}
	}

	mock.ExpectXRange("log:"+jid.String(), streamStart(since), "+").SetVal([]goredis.XMessage{
		logMessage(t, "1-0", newEntry(since, "at since")),
		logMessage(t, "2-0", newEntry(until, "at until")),
		logMessage(t, "3-0", newEntry(until.Add(1*time.Second), "after until")),
	})

	result, err := r.GetLogs(context.Background(), since, until, jid)
	require.NoError(t, err)
	require.Len(t, result, 2, "only since and until are included")
	assert.Equal(t, "at since", result[0].Message())
	assert.Equal(t, "at until", result[1].Message())
	assert.NoError(t, mock.ExpectationsWereMet())
}
