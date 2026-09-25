package pg

import (
	"crypto/rand"
	"encoding/binary"
)

// newInstanceID draws the per-process client id: one value shared by all rooms on
// this node, used only for self-filtering. Never an authorization or authorship
// signal.
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
