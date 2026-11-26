package gcs

import (
	"context"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"testing"

	"cloud.google.com/go/storage"
	"github.com/fsouza/fake-gcs-server/fakestorage"
	"github.com/reearth/ygo/persistence"
	"google.golang.org/api/option"
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

	hc := srv.HTTPClient()
	hc.Transport = deletePreconditions{srv: srv, next: hc.Transport}
	client, err := storage.NewClient(context.Background(),
		option.WithHTTPClient(hc), option.WithoutAuthentication())
	if err != nil {
		t.Fatalf("storage.NewClient: %v", err)
	}
	t.Cleanup(func() { _ = client.Close() })
	return client, bucket
}

// deletePreconditions enforces ifGenerationMatch on object deletes, which
// fake-gcs-server accepts but ignores. Real GCS answers a mismatch with 412.
type deletePreconditions struct {
	srv  *fakestorage.Server
	next http.RoundTripper
}

func (d deletePreconditions) RoundTrip(req *http.Request) (*http.Response, error) {
	want := req.URL.Query().Get("ifGenerationMatch")
	if req.Method != http.MethodDelete || want == "" {
		return d.next.RoundTrip(req)
	}
	// JSON API: /storage/v1/b/{bucket}/o/{object}, the object name path-escaped.
	p := req.URL.EscapedPath()
	bi, oi := strings.Index(p, "/b/"), strings.Index(p, "/o/")
	if bi < 0 || oi < bi {
		return d.next.RoundTrip(req)
	}
	bucket, err := url.PathUnescape(p[bi+3 : oi])
	if err != nil {
		return nil, err
	}
	name, err := url.PathUnescape(p[oi+3:])
	if err != nil {
		return nil, err
	}
	obj, err := d.srv.GetObject(bucket, name)
	if err != nil { // absent: let the server answer 404
		return d.next.RoundTrip(req)
	}
	if strconv.FormatInt(obj.Generation, 10) == want {
		return d.next.RoundTrip(req)
	}
	body := `{"error":{"code":412,"message":"conditionNotMet"}}`
	return &http.Response{
		StatusCode: http.StatusPreconditionFailed,
		Status:     "412 Precondition Failed",
		Header:     http.Header{"Content-Type": []string{"application/json"}},
		Body:       io.NopCloser(strings.NewReader(body)),
		Request:    req,
	}, nil
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
