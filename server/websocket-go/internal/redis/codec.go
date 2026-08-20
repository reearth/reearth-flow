package redis

import (
	"context"
	"strconv"
	"time"

	goredis "github.com/redis/go-redis/v9"
	"github.com/reearth/ygo/cluster"
)

// streamKind classifies a parsed stream entry by its "type" field.
type streamKind int

const (
	kindUnknown streamKind = iota
	kindSync
	kindAwareness
)

// xaddEntry writes one stream entry with fields in the exact wire order
// type,data,clientId,timestamp (clientId and timestamp decimal-ASCII). The slice
// form preserves field order; a map would reorder.
func xaddEntry(ctx context.Context, c goredis.Cmdable, key, msgType string, data []byte, clientID uint64) error {
	return c.XAdd(ctx, &goredis.XAddArgs{
		Stream: key,
		ID:     "*",
		Values: xaddValues(msgType, data, clientID),
	}).Err()
}

// xaddValues builds the ordered field list for one entry with a per-entry
// timestamp (opaque to readers, so wire-compatible with the batch path).
func xaddValues(msgType string, data []byte, clientID uint64) []any {
	return xaddValuesTS(msgType, data, clientID, strconv.FormatInt(time.Now().UnixMilli(), 10))
}

// xaddValuesTS is xaddValues with a caller-supplied timestamp, used by the batched
// writer so every entry in one flush shares one timestamp.
func xaddValuesTS(msgType string, data []byte, clientID uint64, timestamp string) []any {
	return []any{
		"type", msgType,
		"data", data,
		"clientId", strconv.FormatUint(clientID, 10),
		"timestamp", timestamp,
	}
}

// pipelineXAdd writes every item to the room stream in one pipelined round-trip,
// in order, all sharing one batch timestamp.
func pipelineXAdd(ctx context.Context, c goredis.Cmdable, key string, clientID uint64, items []writeItem) error {
	if len(items) == 0 {
		return nil
	}
	batchTS := strconv.FormatInt(time.Now().UnixMilli(), 10)
	pipe := c.Pipeline()
	for i := range items {
		msgType := msgTypeSync
		if items[i].kind == cluster.KindAwareness {
			msgType = msgTypeAwareness
		}
		pipe.XAdd(ctx, &goredis.XAddArgs{
			Stream: key,
			ID:     "*",
			Values: xaddValuesTS(msgType, items[i].data, clientID, batchTS),
		})
	}
	_, err := pipe.Exec(ctx)
	return err
}

// parsedEntry is one classified stream entry the reader routes or drops.
type parsedEntry struct {
	id     string
	kind   streamKind
	data   []byte
	isSelf bool // clientId == our own id → echo, drop
}

// parseEntry classifies an XMessage and flags self-originated entries (clientId
// decimal-string compare against ownID).
func parseEntry(m goredis.XMessage, ownID uint64) parsedEntry {
	e := parsedEntry{id: m.ID, kind: kindUnknown}

	switch fieldString(m.Values, "type") {
	case msgTypeSync:
		e.kind = kindSync
	case msgTypeAwareness:
		e.kind = kindAwareness
	}

	if d, ok := m.Values["data"]; ok {
		if s, ok := d.(string); ok {
			e.data = []byte(s)
		}
	}

	e.isSelf = fieldString(m.Values, "clientId") == strconv.FormatUint(ownID, 10)
	return e
}

func fieldString(values map[string]any, k string) string {
	if v, ok := values[k]; ok {
		if s, ok := v.(string); ok {
			return s
		}
	}
	return ""
}
