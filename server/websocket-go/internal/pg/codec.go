package pg

import (
	"time"

	"github.com/reearth/ygo/cluster"
)

// Wire values for ws_stream.kind. These are persisted, so they are fixed forever:
// a coexisting instance on an older build reads rows written by a newer one.
const (
	kindSync      int16 = 0
	kindAwareness int16 = 1
)

// encodeKind maps a cluster.Kind onto its column value.
func encodeKind(k cluster.Kind) int16 {
	if k == cluster.KindAwareness {
		return kindAwareness
	}
	return kindSync
}

// decodeKind maps a column value back. ok is false for anything unrecognised, so
// a row written by a future build with a new kind is skipped rather than
// misapplied as a document update.
func decodeKind(v int16) (cluster.Kind, bool) {
	switch v {
	case kindSync:
		return cluster.KindSync, true
	case kindAwareness:
		return cluster.KindAwareness, true
	default:
		return 0, false
	}
}

// entry is one decoded ws_stream row.
type entry struct {
	id     int64
	kind   cluster.Kind
	data   []byte
	known  bool // false => unrecognised kind, skip but still advance the cursor
	isSelf bool // written by this instance: an echo, skip on apply
	// age is how long the row had existed when this reader fetched it, measured
	// entirely by the database clock. Recorded only for rows actually delivered.
	age time.Duration
}
