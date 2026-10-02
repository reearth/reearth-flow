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
