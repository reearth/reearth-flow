package redis

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"strconv"
	"strings"
	"time"

	goredis "github.com/redis/go-redis/v9"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// changesKey is a room's change record: the user edits since its last version.
// A hash: "first" is the oldest edit's time (Unix ms), and each "b:{id}" field
// holds one batch an instance wrote (versions.EncodeRecord). Batches are kept
// apart because they merge as sets of item IDs, which Lua cannot do; Take
// merges them. Go-only; the Rust server never had one.
func changesKey(docID string) string { return "flow:version:changes:" + docID }

// changesTTLSeconds expires a record nobody took. Every trigger takes the
// record, so one this old belongs to a room nobody has opened since.
const changesTTLSeconds = 30 * 24 * 60 * 60

const batchFieldPrefix = "b:"

// markChangesScript adds one batch. ARGV: the batch's first edit (ms, kept as
// given so no float formatting touches it), its field, its value, the ttl.
const markChangesScript = `
local key = KEYS[1]
local cur = tonumber(redis.call('HGET', key, 'first'))
if (not cur) or tonumber(ARGV[1]) < cur then
    redis.call('HSET', key, 'first', ARGV[1])
end
redis.call('HSET', key, ARGV[2], ARGV[3])
redis.call('EXPIRE', key, ARGV[4])
return 1
`

// takeChangesScript reads a record and deletes it in one step.
const takeChangesScript = `
local v = redis.call('HGETALL', KEYS[1])
redis.call('DEL', KEYS[1])
return v
`

var (
	markChanges = goredis.NewScript(markChangesScript)
	takeChanges = goredis.NewScript(takeChangesScript)
)

// ChangeStore keeps change records in Redis.
type ChangeStore struct {
	client goredis.UniversalClient
}

// NewChangeStore returns a ChangeStore over client.
func NewChangeStore(client goredis.UniversalClient) *ChangeStore {
	return &ChangeStore{client: client}
}

var _ versions.Store = (*ChangeStore)(nil)

func (s *ChangeStore) Mark(ctx context.Context, room string, batch versions.Record) error {
	value, err := versions.EncodeRecord(batch)
	if err != nil {
		return err
	}
	var id [8]byte
	if _, err := rand.Read(id[:]); err != nil {
		return err
	}
	return markChanges.Run(ctx, s.client, []string{changesKey(room)},
		batch.FirstEditAt().UnixMilli(), batchFieldPrefix+hex.EncodeToString(id[:]), value, changesTTLSeconds,
	).Err()
}

func (s *ChangeStore) Take(ctx context.Context, room string) (versions.Record, bool, error) {
	kv, err := takeChanges.Run(ctx, s.client, []string{changesKey(room)}).StringSlice()
	if err != nil {
		return nil, false, err
	}
	if len(kv) == 0 {
		return nil, false, nil
	}
	var batches [][]byte
	for i := 0; i+1 < len(kv); i += 2 {
		if strings.HasPrefix(kv[i], batchFieldPrefix) {
			batches = append(batches, []byte(kv[i+1]))
		}
	}
	rec, err := versions.DecodeRecords(batches)
	if err != nil {
		return nil, false, err
	}
	return rec, true, nil
}

func (s *ChangeStore) FirstEdit(ctx context.Context, room string) (time.Time, bool, error) {
	v, err := s.client.HGet(ctx, changesKey(room), "first").Result()
	if err == goredis.Nil {
		return time.Time{}, false, nil
	}
	if err != nil {
		return time.Time{}, false, err
	}
	ms, err := strconv.ParseInt(v, 10, 64)
	if err != nil {
		return time.Time{}, false, err
	}
	return time.UnixMilli(ms), true, nil
}
