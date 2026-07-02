package server

import (
	"context"
	"errors"

	"github.com/reearth/ygo/crdt"
	ygws "github.com/reearth/ygo/provider/websocket"
)

// metadataMapName / rollbackInProgressKey is the doc-content path the UI
// observes to hide the canvas during a rollback.
const (
	metadataMapName       = "metadata"
	rollbackInProgressKey = "rollbackInProgress"
)

// TransientMapKeys returns the doc map keys carrying ephemeral UI state that must
// never be persisted into a GCS snapshot (metadata.rollbackInProgress). Wire this
// into gcs.Options.TransientMapKeys so every doc_v2 write strips them.
func TransientMapKeys() map[string][]string {
	return map[string][]string{metadataMapName: {rollbackInProgressKey}}
}

// SignalRollback sets or clears metadata.rollbackInProgress on the live room's
// doc and broadcasts the change to connected peers. The flag is transient and
// must never be persisted into a GCS snapshot.
func (s *Server) SignalRollback(ctx context.Context, room string, inProgress bool) error {
	err := s.ws.Apply(ctx, room, func(doc *crdt.Doc, transact func(func(*crdt.Transaction))) {
		// GetMap must be called outside transact, which already holds the doc lock.
		m := doc.GetMap(metadataMapName)
		transact(func(txn *crdt.Transaction) {
			if inProgress {
				m.Set(txn, rollbackInProgressKey, true)
			} else {
				m.Delete(txn, rollbackInProgressKey)
			}
		})
	})
	if errors.Is(err, ygws.ErrNoChanges) {
		return nil // idempotent: clearing an absent flag is a no-op success
	}
	return err
}
