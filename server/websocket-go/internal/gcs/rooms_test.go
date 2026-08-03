package gcs

import (
	"testing"

	"github.com/reearth/ygo/persistence"
)

func TestRoomListerConformance_GCS(t *testing.T) {
	persistence.RunRoomListerConformance(t, func() persistence.SnapshotVersionedPersistence {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
		if err != nil {
			t.Fatalf("gcs.New: %v", err)
		}
		return a
	})
}
