package cloudrunworker

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"sync"
	"sync/atomic"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// fakeFile is a minimal gateway.File for testing the cloudrunworker.
// Only WriteCancelFlag and CancelFlagURI are exercised; all other methods panic.
type fakeFile struct {
	bucket      string
	cancelCalls []string
}

func (f *fakeFile) CancelFlagURI(jobID string) string {
	return fmt.Sprintf("gs://%s/cancel/%s", f.bucket, jobID)
}
func (f *fakeFile) WriteCancelFlag(_ context.Context, jobID string) error {
	f.cancelCalls = append(f.cancelCalls, jobID)
	return nil
}
func (f *fakeFile) ReadAsset(context.Context, string) (io.ReadCloser, error)         { panic("unused") }
func (f *fakeFile) ReadActions(context.Context, string) (io.ReadCloser, error)       { panic("unused") }
func (f *fakeFile) UploadAsset(context.Context, *file.File) (*url.URL, int64, error) { panic("unused") }
func (f *fakeFile) DeleteAsset(context.Context, *url.URL) error                      { panic("unused") }
func (f *fakeFile) ReadWorkflow(context.Context, string) (io.ReadCloser, error)      { panic("unused") }
func (f *fakeFile) UploadWorkflow(context.Context, *file.File) (*url.URL, error)     { panic("unused") }
func (f *fakeFile) RemoveWorkflow(context.Context, *url.URL) error                   { panic("unused") }
func (f *fakeFile) ReadMetadata(context.Context, string) (io.ReadCloser, error)      { panic("unused") }
func (f *fakeFile) UploadMetadata(context.Context, string, []string) (*url.URL, error) {
	panic("unused")
}
func (f *fakeFile) RemoveMetadata(context.Context, *url.URL) error                { panic("unused") }
func (f *fakeFile) ReadArtifact(context.Context, string) (io.ReadCloser, error)   { panic("unused") }
func (f *fakeFile) ListJobArtifacts(context.Context, string) ([]string, error)    { panic("unused") }
func (f *fakeFile) GetJobLogURL(string) string                                    { panic("unused") }
func (f *fakeFile) CheckJobLogExists(context.Context, string) (bool, error)       { panic("unused") }
func (f *fakeFile) GetJobWorkerLogURL(string) string                              { panic("unused") }
func (f *fakeFile) CheckJobWorkerLogExists(context.Context, string) (bool, error) { panic("unused") }
func (f *fakeFile) GetJobUserFacingLogURL(string) string                          { panic("unused") }
func (f *fakeFile) CheckJobUserFacingLogExists(context.Context, string) (bool, error) {
	panic("unused")
}
func (f *fakeFile) GetJobPreviewSchemaURL(string) string       { panic("unused") }
func (f *fakeFile) GetJobPreviewSchemaUploadURI(string) string { panic("unused") }
func (f *fakeFile) CheckJobPreviewSchemaExists(context.Context, string) (bool, error) {
	panic("unused")
}
func (f *fakeFile) GetIntermediateDataURL(context.Context, string, string) string { panic("unused") }
func (f *fakeFile) CheckIntermediateDataExists(context.Context, string, string) (bool, error) {
	panic("unused")
}
func (f *fakeFile) IssueUploadAssetLink(context.Context, gateway.IssueUploadAssetParam) (*gateway.UploadAssetLink, error) {
	panic("unused")
}
func (f *fakeFile) GetPublicAssetURL(string, string) (*url.URL, error)               { panic("unused") }
func (f *fakeFile) UploadedAsset(context.Context, *asset.Upload) (*file.File, error) { panic("unused") }

func TestRunJob_MapsStatus(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"status":"COMPLETED"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	st, err := repo.RunJob(context.Background(), gateway.RunJobParam{
		JobID:       id.NewJobID(),
		WorkflowURL: "https://wf",
		MetadataURL: "gs://md",
	})
	assert.NoError(t, err)
	assert.Equal(t, gateway.JobStatusCompleted, st)
}

func TestRunJob_500IsFailed(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		_, _ = w.Write([]byte(`{"status":"FAILED","error":"boom"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	st, err := repo.RunJob(context.Background(), gateway.RunJobParam{
		JobID:       id.NewJobID(),
		WorkflowURL: "w",
		MetadataURL: "m",
	})
	assert.Equal(t, gateway.JobStatusFailed, st)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "500")
	assert.Contains(t, err.Error(), "boom")
}

func TestRunJob_PostsContractFields(t *testing.T) {
	var gotBody []byte
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotBody, _ = io.ReadAll(r.Body)
		_, _ = w.Write([]byte(`{"status":"COMPLETED"}`))
	}))
	defer srv.Close()

	fake := &fakeFile{bucket: "mybucket"}
	repo := &Worker{serviceURL: srv.URL, file: fake, httpClient: srv.Client()}
	jid := id.NewJobID()
	_, _ = repo.RunJob(context.Background(), gateway.RunJobParam{
		JobID:       jid,
		WorkflowURL: "https://wf",
		MetadataURL: "gs://md",
	})

	assert.Contains(t, string(gotBody), `"workflow_url":"https://wf"`)
	assert.Contains(t, string(gotBody), `"metadata_path":"gs://md"`)
	assert.Contains(t, string(gotBody), "gs://mybucket/cancel/"+jid.String())
}

func TestRunJob_MapsCancelled(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"status":"CANCELLED"}`))
	}))
	defer srv.Close()
	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	st, err := repo.RunJob(context.Background(), gateway.RunJobParam{JobID: id.NewJobID(), WorkflowURL: "w", MetadataURL: "m"})
	assert.NoError(t, err)
	assert.Equal(t, gateway.JobStatusCancelled, st)
}

func TestPreviewSchema_HitsDedicatedRouteWithBody(t *testing.T) {
	var gotPath string
	var gotBody []byte
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotPath = r.URL.Path
		gotBody, _ = io.ReadAll(r.Body)
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"status":"COMPLETED"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	jid := id.NewJobID()
	n := 25
	st, err := repo.PreviewSchema(context.Background(), gateway.ProbeSchemaParam{
		JobID:       jid,
		WorkflowURL: "https://wf",
		ReportURL:   "gs://mybucket/artifacts/" + jid.String() + "/schema/schema-report.json",
		Variables:   map[string]string{"city": "tokyo"},
		SampleSize:  &n,
	})

	assert.NoError(t, err)
	assert.Equal(t, gateway.JobStatusCompleted, st)
	// Dedicated route, NOT /run.
	assert.Equal(t, "/probe-schema", gotPath)
	assert.Contains(t, string(gotBody), `"job_id":"`+jid.String()+`"`)
	assert.Contains(t, string(gotBody), `"workflow_url":"https://wf"`)
	assert.Contains(t, string(gotBody), `"report_url":"gs://mybucket/artifacts/`+jid.String()+`/schema/schema-report.json"`)
	assert.Contains(t, string(gotBody), `"sample_size":25`)
	assert.Contains(t, string(gotBody), `"city":"tokyo"`)
	// The probe request must not carry run-only fields.
	assert.NotContains(t, string(gotBody), "metadata_path")
}

func TestPreviewSchema_500IsFailed(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		_, _ = w.Write([]byte(`{"status":"FAILED","error":"probe boom"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	st, err := repo.PreviewSchema(context.Background(), gateway.ProbeSchemaParam{
		JobID:       id.NewJobID(),
		WorkflowURL: "w",
	})
	assert.Equal(t, gateway.JobStatusFailed, st)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "500")
	assert.Contains(t, err.Error(), "probe boom")
	assert.Contains(t, err.Error(), "probe-schema")
}

func TestPreviewSchema_OmitsSampleSizeWhenNil(t *testing.T) {
	var gotBody []byte
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotBody, _ = io.ReadAll(r.Body)
		_, _ = w.Write([]byte(`{"status":"COMPLETED"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	_, _ = repo.PreviewSchema(context.Background(), gateway.ProbeSchemaParam{
		JobID:       id.NewJobID(),
		WorkflowURL: "w",
	})
	assert.NotContains(t, string(gotBody), "sample_size")
}

func (f *fakeFile) ResolveIntermediateDataURI(context.Context, string, string) (string, bool, error) {
	panic("unused")
}
func (f *fakeFile) GetFeatureViewUploadURI(string, string) string {
	panic("unused")
}
func (f *fakeFile) GetFeatureViewReportUploadURI(string, string, string) string {
	panic("unused")
}
func (f *fakeFile) GetFeatureViewURL(string, string, string) string {
	panic("unused")
}
func (f *fakeFile) ReadFeatureViewReport(context.Context, string, string, string) (io.ReadCloser, error) {
	panic("unused")
}
func (f *fakeFile) DeleteFeatureViewReport(context.Context, string, string, string) error {
	panic("unused")
}
func (f *fakeFile) CheckFeatureViewFileExists(context.Context, string, string, string) (bool, error) {
	panic("unused")
}

// renderViewRequiredFields are the keys of the engine's RenderViewRequest
// (engine/worker/src/wrapper.rs) that have no serde default: a body missing
// one is rejected by the worker. row, filter and tiles_url are the only
// optional keys.
// Keep this list in step with that struct.
var renderViewRequiredFields = []string{
	"input_uri", "output_uri", "report_url", "name", "shape",
	"draco", "texel_size", "texture_codec", "target_tile_size",
	"min_zoom", "max_zoom", "extent", "max_tile_bytes",
}

func bodyKeys(body map[string]any) []string {
	keys := make([]string, 0, len(body))
	for k := range body {
		keys = append(keys, k)
	}
	return keys
}

func TestRenderView_PostsEveryEngineFieldToItsRoute(t *testing.T) {
	var gotPath string
	var gotBody map[string]any
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotPath = r.URL.Path
		_ = json.NewDecoder(r.Body).Decode(&gotBody)
		_, _ = w.Write([]byte(`{"status":"COMPLETED"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	row := 42
	st, err := repo.RenderView(context.Background(), gateway.RenderViewParam{
		InputURI:  "gs://b/artifacts/J/feature-store/n.default.jsonl.zst",
		OutputURI: "gs://b/artifacts/J/feature-view/n.default",
		ReportURI: "gs://b/artifacts/J/feature-view/n.default/k.report.json",
		Name:      "k",
		Shape:     featureview.ShapeGLTF,
		Row:       &row,
		Options:   featureview.DefaultOptions(),
	})

	require.NoError(t, err)
	assert.Equal(t, gateway.JobStatusCompleted, st)
	assert.Equal(t, "/render-view", gotPath, "a dedicated route, not /run")

	assert.ElementsMatch(t, append(renderViewRequiredFields, "row"), bodyKeys(gotBody),
		"exactly the engine's fields: a gltf request carries row and no filter")

	assert.Equal(t, "gs://b/artifacts/J/feature-view/n.default/k.report.json", gotBody["report_url"])
	assert.Equal(t, "gltf", gotBody["shape"])
	assert.EqualValues(t, 42, gotBody["row"])
	assert.EqualValues(t, featureview.DefaultOptions().MaxTileBytes, gotBody["max_tile_bytes"])
}

func TestRenderView_SendsTheFilterForATilesView(t *testing.T) {
	var gotBody map[string]any
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewDecoder(r.Body).Decode(&gotBody)
		_, _ = w.Write([]byte(`{"status":"COMPLETED"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	filter := "attributes.height > 10"
	_, err := repo.RenderView(context.Background(), gateway.RenderViewParam{
		Name: "k", Shape: featureview.ShapeTiles, Filter: &filter, Options: featureview.DefaultOptions(),
	})

	require.NoError(t, err)
	assert.Equal(t, "tiles", gotBody["shape"])
	assert.Equal(t, filter, gotBody["filter"])
	assert.ElementsMatch(t, append(renderViewRequiredFields, "filter"), bodyKeys(gotBody),
		"a tiles request carries filter and no row")
}

func TestRenderView_SendsTheTilesURLVerbatim(t *testing.T) {
	var gotBody map[string]any
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewDecoder(r.Body).Decode(&gotBody)
		_, _ = w.Write([]byte(`{"status":"COMPLETED"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	tilesURL := "https://api.example/artifacts/J/feature-view/n.default/k/{z}/{x}/{y}.mvt"
	_, err := repo.RenderView(context.Background(), gateway.RenderViewParam{
		Name: "k", Shape: featureview.ShapeTiles, TilesURL: &tilesURL, Options: featureview.DefaultOptions(),
	})

	require.NoError(t, err)
	assert.Equal(t, tilesURL, gotBody["tiles_url"])
	assert.ElementsMatch(t, append(renderViewRequiredFields, "tiles_url"), bodyKeys(gotBody))
}

func TestRenderView_500IsFailed(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		_, _ = w.Write([]byte(`{"status":"FAILED","error":"render boom"}`))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	st, err := repo.RenderView(context.Background(), gateway.RenderViewParam{
		Name: "k", Shape: featureview.ShapeTiles, Options: featureview.DefaultOptions(),
	})

	assert.Equal(t, gateway.JobStatusFailed, st)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "500")
	assert.Contains(t, err.Error(), "render boom")
	assert.Contains(t, err.Error(), "render-view")
}

// versionServer answers GET /version with status and body, counting calls.
func versionServer(t *testing.T, status int, body string) (*httptest.Server, *atomic.Int32) {
	t.Helper()
	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls.Add(1)
		assert.Equal(t, http.MethodGet, r.Method)
		assert.Equal(t, "/version", r.URL.Path)
		w.WriteHeader(status)
		_, _ = w.Write([]byte(body))
	}))
	t.Cleanup(srv.Close)
	return srv, &calls
}

// fakeClock is a settable now, so cache windows can be crossed without waiting.
type fakeClock struct{ t time.Time }

func (c *fakeClock) now() time.Time          { return c.t }
func (c *fakeClock) advance(d time.Duration) { c.t = c.t.Add(d) }

func TestEngineVersion_ReadsTheServicesVersion(t *testing.T) {
	srv, _ := versionServer(t, http.StatusOK, `{"engineVersion":"0.0.583"}`)
	w := &Worker{serviceURL: srv.URL, httpClient: srv.Client()}

	v, err := w.EngineVersion(context.Background())
	require.NoError(t, err)
	assert.Equal(t, "0.0.583", v)
}

// A worker deployed before the route existed answers 404, which is expected
// until it is redeployed; any other non-200 is as much an error.
func TestEngineVersion_AnErrorStatusIsAnError(t *testing.T) {
	for _, status := range []int{http.StatusNotFound, http.StatusInternalServerError} {
		srv, _ := versionServer(t, status, `{"engineVersion":"0.0.583"}`)
		w := &Worker{serviceURL: srv.URL, httpClient: srv.Client()}

		v, err := w.EngineVersion(context.Background())
		assert.Error(t, err, "http %d", status)
		assert.Empty(t, v, "http %d", status)
	}
}

func TestEngineVersion_AnEmptyVersionIsAnError(t *testing.T) {
	for _, body := range []string{`{"engineVersion":""}`, `{}`, `not json`} {
		srv, _ := versionServer(t, http.StatusOK, body)
		w := &Worker{serviceURL: srv.URL, httpClient: srv.Client()}

		_, err := w.EngineVersion(context.Background())
		assert.Error(t, err, body)
	}
}

// The Service takes one request per instance, so a version asked on every
// render could start an instance just to answer it.
func TestEngineVersion_AsksOncePerFiveMinutes(t *testing.T) {
	srv, calls := versionServer(t, http.StatusOK, `{"engineVersion":"0.0.583"}`)
	clock := &fakeClock{t: time.Unix(1_000_000, 0)}
	w := &Worker{serviceURL: srv.URL, httpClient: srv.Client(), now: clock.now}

	for range 3 {
		_, err := w.EngineVersion(context.Background())
		require.NoError(t, err)
	}
	assert.EqualValues(t, 1, calls.Load())

	clock.advance(5*time.Minute - time.Second)
	_, _ = w.EngineVersion(context.Background())
	assert.EqualValues(t, 1, calls.Load(), "still inside the window")

	clock.advance(2 * time.Second)
	_, _ = w.EngineVersion(context.Background())
	assert.EqualValues(t, 2, calls.Load(), "the window has passed")
}

func TestEngineVersion_RemembersAFailureForAMinute(t *testing.T) {
	srv, calls := versionServer(t, http.StatusNotFound, ``)
	clock := &fakeClock{t: time.Unix(1_000_000, 0)}
	w := &Worker{serviceURL: srv.URL, httpClient: srv.Client(), now: clock.now}

	_, err := w.EngineVersion(context.Background())
	require.Error(t, err)
	_, err = w.EngineVersion(context.Background())
	require.Error(t, err, "the failure is remembered")
	assert.EqualValues(t, 1, calls.Load())

	clock.advance(time.Minute + time.Second)
	_, _ = w.EngineVersion(context.Background())
	assert.EqualValues(t, 2, calls.Load(), "a failure is retried sooner than a success")
}

// Renders that arrive together share one request rather than each making
// their own.
func TestEngineVersion_ConcurrentCallersShareOneRequest(t *testing.T) {
	var calls atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls.Add(1)
		time.Sleep(100 * time.Millisecond)
		_, _ = w.Write([]byte(`{"engineVersion":"0.0.583"}`))
	}))
	t.Cleanup(srv.Close)
	w := &Worker{serviceURL: srv.URL, httpClient: srv.Client()}

	var wg sync.WaitGroup
	for range 8 {
		wg.Add(1)
		go func() {
			defer wg.Done()
			v, err := w.EngineVersion(context.Background())
			assert.NoError(t, err)
			assert.Equal(t, "0.0.583", v)
		}()
	}
	wg.Wait()
	assert.EqualValues(t, 1, calls.Load())
}

// The client's own timeout is sized for an hour-long run; asking the version
// must not wait anything like that.
func TestEngineVersion_GivesUpAtItsOwnLimit(t *testing.T) {
	release := make(chan struct{})
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		select {
		case <-release:
		case <-r.Context().Done():
		}
	}))
	t.Cleanup(func() { close(release); srv.Close() })
	w := &Worker{serviceURL: srv.URL, httpClient: srv.Client(), versionLimit: 100 * time.Millisecond}

	done := make(chan error, 1)
	go func() {
		_, err := w.EngineVersion(context.Background())
		done <- err
	}()
	select {
	case err := <-done:
		assert.Error(t, err)
	case <-time.After(5 * time.Second):
		t.Fatal("EngineVersion was still waiting long after its limit")
	}
}

// One caller giving up must not be remembered as the Service failing: the
// failure would be cached for everyone for a minute.
func TestEngineVersion_ACancelledCallerDoesNotSpoilTheCache(t *testing.T) {
	srv, calls := versionServer(t, http.StatusOK, `{"engineVersion":"0.0.583"}`)
	w := &Worker{serviceURL: srv.URL, httpClient: srv.Client()}

	cancelled, cancel := context.WithCancel(context.Background())
	cancel()
	v, err := w.EngineVersion(cancelled)
	require.NoError(t, err)
	assert.Equal(t, "0.0.583", v)
	assert.EqualValues(t, 1, calls.Load())
}
