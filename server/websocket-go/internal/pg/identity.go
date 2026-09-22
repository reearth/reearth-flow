package pg

import (
	"crypto/rand"
	"encoding/binary"
)

// newInstanceID draws the per-process client id: one value shared by all rooms on
// this node, used only for self-filtering. Never an authorization or authorship
// signal.
//
// Unlike the Redis relay's uint64 this must be a POSITIVE int64, because the column
// is bigint (signed) and pgx would reject a uint64 above math.MaxInt64. The top bit
// is cleared rather than the value being rejected and redrawn, which keeps this
// branch-free; 63 bits of entropy is ample for distinguishing a handful of
// instances.
func newInstanceID() (int64, error) {
	var b [8]byte
	if _, err := rand.Read(b[:]); err != nil {
		return 0, err
	}
	id := int64(binary.BigEndian.Uint64(b[:]) &^ (1 << 63))
	if id == 0 {
		id = 1
	}
	return id, nil
}
