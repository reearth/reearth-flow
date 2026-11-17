package redis

import (
	"context"
	"errors"
	"hash/fnv"
	"strconv"
	"strings"
	"sync"
	"time"

	goredis "github.com/redis/go-redis/v9"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// changesKey is a room's change record: the user edits since its last version.
// A hash: "first" is the oldest edit's time (Unix ms), and each "i:{instance}"
// field holds the edits one instance recorded (versions.EncodeRecord). Edits
// merge as sets of item IDs, which Lua cannot do, so an instance merges its new
// edits into its field in Go and writes the field back only if nothing changed
// it meanwhile; Take merges the fields. A room has at most one field per
// instance however long it goes without a version. Go-only; the Rust server
// never had one.
func changesKey(docID string) string { return "flow:version:changes:" + docID }

// changesTTLSeconds expires a record nobody took. Every trigger takes the
// record, so one this old belongs to a room nobody has opened since.
const changesTTLSeconds = 30 * 24 * 60 * 60

const instanceFieldPrefix = "i:"

// markAttempts bounds Mark's compare-and-set retries. Only this instance
// writes its field, so a retry follows a Take, and a second one is rare.
const markAttempts = 5

// markChangesScript sets an instance's field if it still holds what the
// caller read. ARGV: the field, the value read ("" when absent), the new
// value, the first edit (ms, kept as given so no float formatting touches it),
// the ttl. Returns 0 when the field changed, 1 when written.
const markChangesScript = `
local key = KEYS[1]
local cur = redis.call('HGET', key, ARGV[1])
if (cur or '') ~= ARGV[2] then
    return 0
end
local first = tonumber(redis.call('HGET', key, 'first'))
if (not first) or tonumber(ARGV[4]) < first then
    redis.call('HSET', key, 'first', ARGV[4])
end
redis.call('HSET', key, ARGV[1], ARGV[3])
redis.call('EXPIRE', key, ARGV[5])
return 1
`

// takeChangesScript reads a record and deletes it in one step. A record holds
// at most one field per instance, so the reply is small.
const takeChangesScript = `
local v = redis.call('HGETALL', KEYS[1])
redis.call('DEL', KEYS[1])
return v
`

var (
	markChanges = goredis.NewScript(markChangesScript)
	takeChanges = goredis.NewScript(takeChangesScript)
)

// errMarkContended reports that Mark kept finding its field changed.
var errMarkContended = errors.New("redis: change record kept changing while being written")

// ChangeStore keeps change records in Redis.
type ChangeStore struct {
	client   goredis.UniversalClient
	instance string
	// stripes serialize this instance's Marks of a room, so the
	// compare-and-set only ever retries after a Take.
	stripes [64]sync.Mutex
}

// NewChangeStore returns a ChangeStore over client for the instance named
// instance, which must be unique across the instances sharing client.
func NewChangeStore(client goredis.UniversalClient, instance string) *ChangeStore {
	return &ChangeStore{client: client, instance: instance}
}

var _ versions.Store = (*ChangeStore)(nil)

// Mark merges batch into this instance's field: it reads the field, merges in
// Go, and writes the result only if the field is unchanged, retrying
// otherwise.
func (s *ChangeStore) Mark(ctx context.Context, room string, batch versions.Record) error {
	key, field := changesKey(room), instanceFieldPrefix+s.instance
	first := batch.FirstEditAt()
	if first.IsZero() {
		first = time.Now()
	}
	mu := &s.stripes[stripeOf(room)]
	mu.Lock()
	defer mu.Unlock()
	for range markAttempts {
		old, err := s.client.HGet(ctx, key, field).Result()
		if err != nil && !errors.Is(err, goredis.Nil) {
			return err
		}
		merged := versions.Record{}
		if old != "" {
			if merged, err = versions.DecodeRecord([]byte(old)); err != nil {
				return err
			}
		}
		merged.Merge(batch)
		value, err := versions.EncodeRecord(merged)
		if err != nil {
			return err
		}
		ok, err := markChanges.Run(ctx, s.client, []string{key},
			field, old, value, first.UnixMilli(), changesTTLSeconds,
		).Int()
		if err != nil {
			return err
		}
		if ok == 1 {
			return nil
		}
	}
	return errMarkContended
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
		if strings.HasPrefix(kv[i], instanceFieldPrefix) {
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
	if errors.Is(err, goredis.Nil) {
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

func stripeOf(room string) int {
	h := fnv.New32a()
	_, _ = h.Write([]byte(room))
	return int(h.Sum32() % 64)
}
