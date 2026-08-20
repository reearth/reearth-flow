package gcs

import (
	"testing"

	"cloud.google.com/go/storage"
	"github.com/fsouza/fake-gcs-server/fakestorage"
	"github.com/reearth/ygo/persistence"
)

// newFakeGCS spins up an in-process fake-gcs-server with an empty bucket and
// returns a client pointed at it. Runs offline (no real GCS).
func newFakeGCS(t *testing.T) (*storage.Client, string) {
	t.Helper()
	const bucket = "yrs-test"
	srv, err := fakestorage.NewServerWithOptions(fakestorage.Options{
		InitialObjects: []fakestorage.Object{},
		Scheme:         "http",
	})
	if err != nil {
		t.Fatalf("fakestorage.NewServer: %v", err)
	}
	t.Cleanup(srv.Stop)
	srv.CreateBucketWithOpts(fakestorage.CreateBucketOpts{Name: bucket})

	client := srv.Client()
	t.Cleanup(func() { _ = client.Close() })
	return client, bucket
}

// gcsFactory builds a fresh Phase-1 Adapter on its own fake-gcs bucket so each
// conformance subtest is isolated.
func gcsFactory(t *testing.T) func() persistence.VersionedPersistence {
	return func() persistence.VersionedPersistence {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
		if err != nil {
			t.Fatalf("gcs.New: %v", err)
		}
		return a
	}
}

// TestRunConformance runs ygo's conformance gate (incl. the PruneAfter crash
// subtest) against fake-gcs.
func TestRunConformance(t *testing.T) {
	persistence.RunConformance(t, gcsFactory(t))
}

// TestRunConformancePhase2 runs the same gate against the Phase-2 layout.
func TestRunConformancePhase2(t *testing.T) {
	factory := func() persistence.VersionedPersistence {
		client, bucket := newFakeGCS(t)
		a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock(), Phase2: true})
		if err != nil {
			t.Fatalf("gcs.New phase2: %v", err)
		}
		return a
	}
	persistence.RunConformance(t, factory)
}
