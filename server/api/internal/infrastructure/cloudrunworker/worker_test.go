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
	assert.NotErrorIs(t, err, gateway.ErrWorkerBusy, "a render that ran and failed is not a busy pool")
}

// Cloud Run answers 429 itself, without reaching the worker, when every
// instance it may run is busy. The body is Cloud Run's, not the worker's JSON.
func TestRenderView_429IsWorkerBusy(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusTooManyRequests)
		_, _ = w.Write([]byte("Rate exceeded."))
	}))
	defer srv.Close()

	repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
	st, err := repo.RenderView(context.Background(), gateway.RenderViewParam{
		Name: "k", Shape: featureview.ShapeTiles, Options: featureview.DefaultOptions(),
	})

	assert.Equal(t, gateway.JobStatusFailed, st)
	assert.ErrorIs(t, err, gateway.ErrWorkerBusy)
	assert.Contains(t, err.Error(), "render-view", "the log still says which call was turned away")
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

// fakeClock is a settable now, so cache windows and waits can be crossed
// without waiting.
type fakeClock struct {
	t  time.Time
	mu sync.Mutex
}

func (c *fakeClock) now() time.Time {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.t
}

func (c *fakeClock) advance(d time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.t = c.t.Add(d)
}

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

func TestRunJob_TellsARunNotStarted(t *testing.T) {
	t.Parallel()

	answer := func(t *testing.T, code int) error {
		t.Helper()
		srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(code)
			_, _ = w.Write([]byte(`{"status":"FAILED","error":"boom"}`))
		}))
		defer srv.Close()

		repo := &Worker{serviceURL: srv.URL, file: &fakeFile{bucket: "b"}, httpClient: srv.Client()}
		_, err := repo.RunJob(context.Background(), gateway.RunJobParam{JobID: id.NewJobID()})
		require.Error(t, err)
		return err
	}

	t.Run("a request the wrapper refused did not start the run", func(t *testing.T) {
		t.Parallel()

		assert.ErrorIs(t, answer(t, http.StatusBadRequest), gateway.ErrRunNotStarted)
	})

	t.Run("a run that failed may have started", func(t *testing.T) {
		t.Parallel()

		assert.NotErrorIs(t, answer(t, http.StatusInternalServerError), gateway.ErrRunNotStarted)
	})

	t.Run("a worker never reached did not start the run", func(t *testing.T) {
		t.Parallel()

		srv := httptest.NewServer(http.NotFoundHandler())
		url := srv.URL
		srv.Close()
		repo := &Worker{serviceURL: url, file: &fakeFile{bucket: "b"}, httpClient: http.DefaultClient}

		_, err := repo.RunJob(context.Background(), gateway.RunJobParam{JobID: id.NewJobID()})

		assert.ErrorIs(t, err, gateway.ErrRunNotStarted)
	})
}

func callRoute(t *testing.T, route, srvURL string, client *http.Client) error {
	t.Helper()
	return callRouteAt(t, route, srvURL, client, nil)
}

func callRouteAt(t *testing.T, route, srvURL string, client *http.Client, now func() time.Time) error {
	t.Helper()
	w := &Worker{serviceURL: srvURL, file: &fakeFile{bucket: "b"}, httpClient: client, now: now}
	ctx := context.Background()
	var err error
	switch route {
	case "/run":
		_, err = w.RunJob(ctx, gateway.RunJobParam{JobID: id.NewJobID()})
	case "/probe-schema":
		_, err = w.PreviewSchema(ctx, gateway.ProbeSchemaParam{JobID: id.NewJobID()})
	case "/render-view":
		_, err = w.RenderView(ctx, gateway.RenderViewParam{Name: "k", Shape: featureview.ShapeTiles, Options: featureview.DefaultOptions()})
	default:
		t.Fatalf("unknown route %s", route)
	}
	return err
}

func answerOn(t *testing.T, route string, code int, body string) error {
	t.Helper()
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, route, r.URL.Path)
		w.WriteHeader(code)
		_, _ = w.Write([]byte(body))
	}))
	defer srv.Close()
	return callRoute(t, route, srv.URL, srv.Client())
}

// Cloud Run's answer when the pool is full, as observed on the dev Service on
// 2026-11-26.
func cloudRunBusyOn(t *testing.T, route string) error {
	t.Helper()
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, route, r.URL.Path)
		w.Header().Set("Content-Type", "text/html")
		w.WriteHeader(http.StatusTooManyRequests)
		_, _ = w.Write([]byte("Rate exceeded."))
	}))
	defer srv.Close()
	return callRoute(t, route, srv.URL, srv.Client())
}

// Cloud Run's front end refusing a request while instances are still
// starting, as observed on the dev Service on 2026-11-27: an HTML 500 after
// ~10.4 s.
func cloudRunAbortOn(t *testing.T, route string, hold time.Duration) error {
	t.Helper()
	clock := &fakeClock{t: time.Unix(1_000_000, 0)}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, route, r.URL.Path)
		clock.advance(hold)
		w.Header().Set("Content-Type", "text/html; charset=UTF-8")
		w.WriteHeader(http.StatusInternalServerError)
		_, _ = w.Write([]byte(`<html><head>
<meta http-equiv="content-type" content="text/html;charset=utf-8">
<title>500 Server Error</title>
</head>
<body text=#000000 bgcolor=#ffffff>
<h1>Error: Server Error</h1>
<h2>The server encountered an error and could not complete your request.<p>Please try again in 30 seconds.</h2>
<h2></h2>
</body></html>
`))
	}))
	defer srv.Close()
	return callRouteAt(t, route, srv.URL, srv.Client(), clock.now)
}

func TestWorker_OnlyARequestCloudRunRefusedIsABusyPool(t *testing.T) {
	t.Parallel()

	for _, route := range []string{"/run", "/probe-schema", "/render-view"} {
		t.Run(route+": a 429 is a busy pool and nothing ran", func(t *testing.T) {
			t.Parallel()

			err := cloudRunBusyOn(t, route)

			assert.ErrorIs(t, err, gateway.ErrWorkerBusy)
			assert.ErrorIs(t, err, gateway.ErrRunNotStarted)
			assert.Contains(t, err.Error(), route[1:], "the log still says which call was turned away")
		})

		t.Run(route+": a 400 is not a busy pool", func(t *testing.T) {
			t.Parallel()

			err := answerOn(t, route, http.StatusBadRequest, `{"status":"FAILED","error":"bad request"}`)

			require.Error(t, err)
			assert.NotErrorIs(t, err, gateway.ErrWorkerBusy)
		})

		t.Run(route+": the wrapper's 500 is a failed run, not a busy pool", func(t *testing.T) {
			t.Parallel()

			err := answerOn(t, route, http.StatusInternalServerError, `{"status":"FAILED","error":"boom"}`)

			require.Error(t, err)
			assert.NotErrorIs(t, err, gateway.ErrWorkerBusy)
			assert.NotErrorIs(t, err, gateway.ErrRunNotStarted)
		})

		t.Run(route+": the front end's 500 within 30 s is a busy pool and nothing ran", func(t *testing.T) {
			t.Parallel()

			err := cloudRunAbortOn(t, route, 10400*time.Millisecond)

			assert.ErrorIs(t, err, gateway.ErrWorkerBusy)
			assert.ErrorIs(t, err, gateway.ErrRunNotStarted)
			assert.Contains(t, err.Error(), route[1:], "the log still says which call was turned away")
		})

		t.Run(route+": the front end's 500 at exactly 30 s is still a busy pool", func(t *testing.T) {
			t.Parallel()

			err := cloudRunAbortOn(t, route, 30*time.Second)

			assert.ErrorIs(t, err, gateway.ErrWorkerBusy)
			assert.ErrorIs(t, err, gateway.ErrRunNotStarted)
		})

		t.Run(route+": the front end's 500 after 30 s may have started the run", func(t *testing.T) {
			t.Parallel()

			err := cloudRunAbortOn(t, route, 31*time.Second)

			require.Error(t, err)
			assert.NotErrorIs(t, err, gateway.ErrWorkerBusy)
			assert.NotErrorIs(t, err, gateway.ErrRunNotStarted)
		})

		t.Run(route+": the wrapper's 500 cut off in transit is a failed run, not a busy pool", func(t *testing.T) {
			t.Parallel()

			srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				w.Header().Set("Content-Length", "60")
				w.WriteHeader(http.StatusInternalServerError)
				_, _ = w.Write([]byte(`{"status":"FAILED","error":"worker exit`))
				w.(http.Flusher).Flush()
				conn, _, err := w.(http.Hijacker).Hijack()
				if assert.NoError(t, err) {
					_ = conn.Close()
				}
			}))
			defer srv.Close()

			err := callRoute(t, route, srv.URL, srv.Client())

			require.Error(t, err)
			assert.NotErrorIs(t, err, gateway.ErrWorkerBusy)
			assert.NotErrorIs(t, err, gateway.ErrRunNotStarted)
		})

		t.Run(route+": a 503 is not a busy pool", func(t *testing.T) {
			t.Parallel()

			err := answerOn(t, route, http.StatusServiceUnavailable, "Service Unavailable")

			require.Error(t, err)
			assert.NotErrorIs(t, err, gateway.ErrWorkerBusy)
		})

		t.Run(route+": a dropped connection is not a busy pool", func(t *testing.T) {
			t.Parallel()

			srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				conn, _, err := w.(http.Hijacker).Hijack()
				if assert.NoError(t, err) {
					_ = conn.Close()
				}
			}))
			defer srv.Close()

			err := callRoute(t, route, srv.URL, srv.Client())

			require.Error(t, err)
			assert.NotErrorIs(t, err, gateway.ErrWorkerBusy)
		})
	}
}
