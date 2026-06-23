package gcs

import (
	"context"
	"io"
	"net/http"
	"strings"
	"testing"

	"cloud.google.com/go/storage"
	"google.golang.org/api/option"
)

// faultRT returns a fixed HTTP status for every request, simulating a transient
// non-404 GCS failure (e.g. a 500/503 blip or a quota error).
type faultRT struct{ status int }

func (f faultRT) RoundTrip(*http.Request) (*http.Response, error) {
	return &http.Response{
		StatusCode: f.status,
		Status:     http.StatusText(f.status),
		Body:       io.NopCloser(strings.NewReader(`{"error":{"code":` + http.StatusText(f.status) + `}}`)),
		Header:     make(http.Header),
	}, nil
}

func faultKV(t *testing.T, status int) kv {
	t.Helper()
	client, err := storage.NewClient(context.Background(),
		option.WithHTTPClient(&http.Client{Transport: faultRT{status: status}}),
		option.WithoutAuthentication(),
	)
	if err != nil {
		t.Fatalf("storage.NewClient: %v", err)
	}
	// RetryNever so a retryable status (500/503) surfaces immediately instead of
	// retrying to the context deadline — keeps the test fast and deterministic.
	bucket := client.Bucket("b").Retryer(storage.WithPolicy(storage.RetryNever))
	return kv{bucket: bucket}
}

// Regression guard for the stale-canvas root cause: a transient NON-404 GCS read
// error must propagate, NOT be collapsed into errNotFound. If it were treated as
// "object not found", a caller would read it as "document is empty" and a
// subsequent persist could wipe live canvas state. kv.get must only map a true
// 404 (storage.ErrObjectNotExist) to errNotFound.
func TestKVGet_Non404ErrorIsNotSwallowedAsNotFound(t *testing.T) {
	for _, status := range []int{
		http.StatusInternalServerError, // 500 — the original incident
		http.StatusServiceUnavailable,  // 503
		http.StatusForbidden,           // 403 — non-retryable
	} {
		t.Run(http.StatusText(status), func(t *testing.T) {
			s := faultKV(t, status)
			_, err := s.get(context.Background(), "obj")
			if err == nil {
				t.Fatalf("expected an error for a %d GCS read, got nil", status)
			}
			if err == errNotFound {
				t.Fatalf("%d GCS error was swallowed as errNotFound (stale-canvas regression)", status)
			}
		})
	}
}

// A genuine 404 must still map to errNotFound, so the not-found fast-paths keep
// working. (Pins the other half of the contract.)
func TestKVGet_404MapsToErrNotFound(t *testing.T) {
	s := faultKV(t, http.StatusNotFound)
	_, err := s.get(context.Background(), "missing")
	if err != errNotFound {
		t.Fatalf("404 GCS read = %v, want errNotFound", err)
	}
}
