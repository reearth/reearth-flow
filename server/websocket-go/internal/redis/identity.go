package redis

import (
	"crypto/rand"
	"encoding/binary"
)

// newInstanceID draws the per-process stream clientId: one crypto/rand u64 shared
// by all rooms on this node, used only for self-filtering (never as an authz or
// author signal).
func newInstanceID() (uint64, error) {
	var b [8]byte
	if _, err := rand.Read(b[:]); err != nil {
		return 0, err
	}
	id := binary.BigEndian.Uint64(b[:])
	if id == 0 {
		id = 1
	}
	return id, nil
}
