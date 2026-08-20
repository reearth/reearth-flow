package redis

import (
	"context"
	"strconv"
	"time"

	goredis "github.com/redis/go-redis/v9"
)

// streamTTLSeconds is the 6h stream EXPIRE the empty-marker refreshes on connect.
const streamTTLSeconds = 21600

// Heartbeat: HSET + EXPIRE 120, refreshed every 30s; active = now-seen < 60.
const (
	heartbeatTTLSeconds = 120
	heartbeatRefresh    = 30 * time.Second
	activeTimeoutSecs   = 60
)

// The Lua below is reproduced verbatim from the Rust server so a coexisting Rust
// runtime makes the identical last-instance decision on shared Redis state.

// releaseLockScript is compare-and-del.
const releaseLockScript = `
if redis.call('get', KEYS[1]) == ARGV[1] then
    return redis.call('del', KEYS[1])
else
    return 0
end
`

// emptyMarkerScript XADDs an empty-data sync entry then EXPIREs the stream,
// setting/refreshing the 6h TTL on connect.
const emptyMarkerScript = `
local stream_key = KEYS[1]
local msg_type = ARGV[1]
local data = ARGV[2]
local client_id = ARGV[3]
local timestamp = ARGV[4]
local ttl = ARGV[5]

redis.call('XADD', stream_key, '*',
    'type', msg_type,
    'data', data,
    'clientId', client_id,
    'timestamp', timestamp)
redis.call('EXPIRE', stream_key, ttl)
return 1
`

const heartbeatScript = `
redis.call('HSET', KEYS[1], ARGV[1], ARGV[2])
return redis.call('EXPIRE', KEYS[1], ARGV[3])
`

const activeInstancesScript = `
local active_count = 0
local instances = redis.call('HGETALL', KEYS[1])
local now = tonumber(ARGV[1])
local timeout = tonumber(ARGV[2])

for i = 1, #instances, 2 do
    local instance_id = instances[i]
    local last_seen = tonumber(instances[i+1])
    if now - last_seen < timeout then
        active_count = active_count + 1
    end
end

return active_count
`

// removeHeartbeatScript HDELs self, DELs the hash if now empty, returns 1 if emptied.
const removeHeartbeatScript = `
redis.call('HDEL', KEYS[1], ARGV[1])
local count = redis.call('HLEN', KEYS[1])
if count == 0 then
    redis.call('DEL', KEYS[1])
    return 1
else
    return 0
end
`

// safeDeleteScript, under the doc lock, atomically re-checks read-lock and
// active-count and DELs the stream only when no instance is active. The election
// lives inside the script so the re-count and DEL are atomic against concurrent
// reconnects. KEYS = lock, instances, stream, read_lock; ARGV = instance_id, now,
// timeout.
const safeDeleteScript = `
local lock_key = KEYS[1]
local instances_key = KEYS[2]
local stream_key = KEYS[3]
local read_lock_key = KEYS[4]

local instance_id = ARGV[1]
local now = tonumber(ARGV[2])
local timeout = tonumber(ARGV[3])

if redis.call('EXISTS', read_lock_key) == 1 then
    return {acquired=0, deleted=0, reason="read_in_progress"}
end

if redis.call('GET', lock_key) ~= instance_id then
    if redis.call('SET', lock_key, instance_id, 'NX', 'EX', 10) == false then
        return {acquired=0, deleted=0, reason="lock_failed"}
    end
end

local active_count = 0
local instances = redis.call('HGETALL', instances_key)

for i = 1, #instances, 2 do
    local inst_id = instances[i]
    local last_seen = tonumber(instances[i+1])
    if now - last_seen < timeout then
        active_count = active_count + 1
    end
end

if active_count <= 0 then
    local exists = redis.call('EXISTS', stream_key)
    if exists == 1 then
        redis.call('DEL', stream_key)
        return {acquired=1, deleted=1, reason="success"}
    else
        return {acquired=1, deleted=0, reason="stream_not_exists"}
    end
else
    return {acquired=1, deleted=0, reason="active_instances", count=active_count}
end
`

func nowSecs() int64 { return time.Now().Unix() }

// updateHeartbeat records this instance's liveness with a 120s EXPIRE.
func updateHeartbeat(ctx context.Context, c goredis.Scripter, docID string, clientID uint64) error {
	return goredis.NewScript(heartbeatScript).Run(ctx, c,
		[]string{instancesKey(docID)},
		strconv.FormatUint(clientID, 10), nowSecs(), heartbeatTTLSeconds,
	).Err()
}

// getActiveInstances counts heartbeats with now-seen < timeoutSecs.
func getActiveInstances(ctx context.Context, c goredis.Scripter, docID string, timeoutSecs int) (int64, error) {
	v, err := goredis.NewScript(activeInstancesScript).Run(ctx, c,
		[]string{instancesKey(docID)}, nowSecs(), timeoutSecs,
	).Int64()
	if err != nil {
		return 0, err
	}
	return v, nil
}

// removeHeartbeat HDELs this instance and reports whether the hash became empty.
func removeHeartbeat(ctx context.Context, c goredis.Scripter, docID string, clientID uint64) (bool, error) {
	v, err := goredis.NewScript(removeHeartbeatScript).Run(ctx, c,
		[]string{instancesKey(docID)}, strconv.FormatUint(clientID, 10),
	).Int64()
	if err != nil {
		return false, err
	}
	return v == 1, nil
}

// publishEmptyMarker XADDs an empty-data sync entry and sets the 6h stream EXPIRE.
func publishEmptyMarker(ctx context.Context, c goredis.Scripter, docID string, clientID uint64) error {
	return goredis.NewScript(emptyMarkerScript).Run(ctx, c,
		[]string{streamKey(docID)},
		msgTypeSync, "", strconv.FormatUint(clientID, 10),
		strconv.FormatInt(time.Now().UnixMilli(), 10), streamTTLSeconds,
	).Err()
}

// acquireLock is SET key value NX EX ttl.
func acquireLock(ctx context.Context, c goredis.Cmdable, key, value string, ttlSecs int) (bool, error) {
	return c.SetNX(ctx, key, value, time.Duration(ttlSecs)*time.Second).Result()
}

// releaseLock is the compare-and-del release.
func releaseLock(ctx context.Context, c goredis.Scripter, key, value string) (bool, error) {
	v, err := goredis.NewScript(releaseLockScript).Run(ctx, c, []string{key}, value).Int64()
	if err != nil {
		return false, err
	}
	return v == 1, nil
}

// safeDeleteStream bails if read:lock exists, else runs the atomic
// election-and-delete Lua and releases the doc lock. instanceValue is the lock
// value ("instance-{clientId}").
func safeDeleteStream(ctx context.Context, c goredis.Cmdable, docID, instanceValue string) error {
	exists, err := c.Exists(ctx, readLockKey(docID)).Result()
	if err != nil {
		return err
	}
	if exists == 1 {
		return nil
	}

	keys := []string{lockKey(docID), instancesKey(docID), streamKey(docID), readLockKey(docID)}
	if err := goredis.NewScript(safeDeleteScript).Run(ctx, c, keys,
		instanceValue, nowSecs(), activeTimeoutSecs,
	).Err(); err != nil {
		return err
	}
	_, _ = releaseLock(ctx, c, lockKey(docID), instanceValue)
	return nil
}
