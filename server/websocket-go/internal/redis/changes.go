package redis

import (
	"context"
	"fmt"
	"sort"
	"strconv"
	"strings"
	"time"

	goredis "github.com/redis/go-redis/v9"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// changesKey is a room's change record: the user edits since its last version.
// A hash: "first"/"last" are the first and latest edit (Unix ms), "editor" and
// "editorAt" the latest edit that named its sender, and one "u:{userId}" field
// per contributor. Go-only; the Rust server never had one.
func changesKey(docID string) string { return "flow:version:changes:" + docID }

// changesTTLSeconds expires a record nobody took. Every trigger takes the
// record, so one this old belongs to a room nobody has opened since.
const changesTTLSeconds = 30 * 24 * 60 * 60

// markChangesScript merges one batch into a record. ARGV: first, last (ms),
// editor, editorAt (ms), ttl, then the contributors. Times are compared as
// numbers but stored as the strings given, so no float formatting touches them.
const markChangesScript = `
local key = KEYS[1]
local first = tonumber(ARGV[1])
local last = tonumber(ARGV[2])
local editor = ARGV[3]
local editorAt = tonumber(ARGV[4])

local cur = tonumber(redis.call('HGET', key, 'first'))
if (not cur) or first < cur then
    redis.call('HSET', key, 'first', ARGV[1])
end
cur = tonumber(redis.call('HGET', key, 'last'))
if (not cur) or last > cur then
    redis.call('HSET', key, 'last', ARGV[2])
end
if editor ~= '' then
    cur = tonumber(redis.call('HGET', key, 'editorAt'))
    if (not cur) or editorAt >= cur then
        redis.call('HSET', key, 'editor', editor, 'editorAt', ARGV[4])
    end
end
for i = 6, #ARGV do
    redis.call('HSET', key, 'u:' .. ARGV[i], 1)
end
redis.call('EXPIRE', key, ARGV[5])
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

func (s *ChangeStore) Mark(ctx context.Context, room string, m versions.Mark) error {
	args := []any{
		m.FirstAt.UnixMilli(), m.LastAt.UnixMilli(),
		m.LastEditor, m.LastEditorAt.UnixMilli(),
		changesTTLSeconds,
	}
	for _, u := range m.Editors {
		if u != "" {
			args = append(args, u)
		}
	}
	return markChanges.Run(ctx, s.client, []string{changesKey(room)}, args...).Err()
}

func (s *ChangeStore) Take(ctx context.Context, room string) (versions.Record, bool, error) {
	v, err := takeChanges.Run(ctx, s.client, []string{changesKey(room)}).StringSlice()
	if err != nil {
		return versions.Record{}, false, err
	}
	return parseChanges(v)
}

func (s *ChangeStore) Peek(ctx context.Context, room string) (versions.Record, bool, error) {
	m, err := s.client.HGetAll(ctx, changesKey(room)).Result()
	if err != nil {
		return versions.Record{}, false, err
	}
	flat := make([]string, 0, 2*len(m))
	for k, v := range m {
		flat = append(flat, k, v)
	}
	return parseChanges(flat)
}

// parseChanges reads a record from HGETALL's field/value pairs.
func parseChanges(kv []string) (versions.Record, bool, error) {
	if len(kv) == 0 {
		return versions.Record{}, false, nil
	}
	if len(kv)%2 != 0 {
		return versions.Record{}, false, fmt.Errorf("redis: change record has %d fields and values", len(kv))
	}
	var r versions.Record
	for i := 0; i < len(kv); i += 2 {
		k, v := kv[i], kv[i+1]
		switch {
		case k == "first":
			r.FirstEditAt = parseMillis(v)
		case k == "last":
			r.LastEditAt = parseMillis(v)
		case k == "editor":
			r.LastEditor = v
		case k == "editorAt":
			r.LastEditorAt = parseMillis(v)
		case strings.HasPrefix(k, "u:"):
			r.Contributors = append(r.Contributors, strings.TrimPrefix(k, "u:"))
		}
	}
	sort.Strings(r.Contributors)
	return r, true, nil
}

func parseMillis(v string) time.Time {
	ms, err := strconv.ParseInt(v, 10, 64)
	if err != nil {
		return time.Time{}
	}
	return time.UnixMilli(ms)
}
