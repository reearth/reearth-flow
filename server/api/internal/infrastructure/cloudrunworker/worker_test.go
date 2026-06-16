package cloudrunworker

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
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
