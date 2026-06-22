package interactor

import (
	"context"
	"io"
	"net/url"
	"testing"

	"github.com/google/uuid"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/usecasex"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
)

// --- fakes ---------------------------------------------------------------

type fakeWebsocket struct {
	latestDoc *websocket.Document
	flushed   []string
}

func (w *fakeWebsocket) GetLatest(_ context.Context, docID string) (*websocket.Document, error) {
	if w.latestDoc != nil {
		return w.latestDoc, nil
	}
	return &websocket.Document{ID: docID, Version: 7}, nil
}
func (w *fakeWebsocket) FlushToGCS(_ context.Context, id string) error {
	w.flushed = append(w.flushed, id)
	return nil
}
func (w *fakeWebsocket) GetHistory(context.Context, string) ([]*websocket.History, error) {
	return nil, nil
}
func (w *fakeWebsocket) GetHistoryByVersion(context.Context, string, int) (*websocket.History, error) {
	return nil, nil
}
func (w *fakeWebsocket) GetHistoryMetadata(context.Context, string) ([]*websocket.HistoryMetadata, error) {
	return nil, nil
}
func (w *fakeWebsocket) Rollback(context.Context, string, int) (*websocket.Document, error) {
	return nil, nil
}
func (w *fakeWebsocket) CreateSnapshot(context.Context, string, int, string) (*websocket.Document, error) {
	return nil, nil
}
func (w *fakeWebsocket) CopyDocument(context.Context, string, string) error   { return nil }
func (w *fakeWebsocket) ImportDocument(context.Context, string, []byte) error { return nil }
func (w *fakeWebsocket) DeleteDocument(context.Context, string) error         { return nil }
func (w *fakeWebsocket) Close() error                                         { return nil }

// previewFakeFile records calls so the test can assert metadata is NOT uploaded.
type previewFakeFile struct {
	uploadWorkflowCalls int
	uploadMetadataCalls int
}

func (f *previewFakeFile) UploadWorkflow(context.Context, *file.File) (*url.URL, error) {
	f.uploadWorkflowCalls++
	u, _ := url.Parse("gs://bucket/workflows/wf.json")
	return u, nil
}
func (f *previewFakeFile) UploadMetadata(context.Context, string, []string) (*url.URL, error) {
	f.uploadMetadataCalls++
	u, _ := url.Parse("gs://bucket/metadata/md.json")
	return u, nil
}
func (f *previewFakeFile) GetJobPreviewSchemaURL(jobID string) string {
	return "gs://bucket/artifacts/" + jobID + "/schema/schema-report.json"
}
func (f *previewFakeFile) GetJobPreviewSchemaUploadURI(jobID string) string {
	return "gs://bucket/artifacts/" + jobID + "/schema/schema-report.json"
}
func (f *previewFakeFile) CheckJobPreviewSchemaExists(context.Context, string) (bool, error) {
	return true, nil
}
func (f *previewFakeFile) ReadAsset(context.Context, string) (io.ReadCloser, error) { panic("unused") }
func (f *previewFakeFile) ReadActions(context.Context, string) (io.ReadCloser, error) {
	panic("unused")
}
func (f *previewFakeFile) UploadAsset(context.Context, *file.File) (*url.URL, int64, error) {
	panic("unused")
}
func (f *previewFakeFile) DeleteAsset(context.Context, *url.URL) error { panic("unused") }
func (f *previewFakeFile) ReadWorkflow(context.Context, string) (io.ReadCloser, error) {
	panic("unused")
}
func (f *previewFakeFile) RemoveWorkflow(context.Context, *url.URL) error { panic("unused") }
func (f *previewFakeFile) ReadMetadata(context.Context, string) (io.ReadCloser, error) {
	panic("unused")
}
func (f *previewFakeFile) RemoveMetadata(context.Context, *url.URL) error { panic("unused") }
func (f *previewFakeFile) ReadArtifact(context.Context, string) (io.ReadCloser, error) {
	panic("unused")
}
func (f *previewFakeFile) ListJobArtifacts(context.Context, string) ([]string, error) {
	panic("unused")
}
func (f *previewFakeFile) GetJobLogURL(string) string                              { panic("unused") }
func (f *previewFakeFile) CheckJobLogExists(context.Context, string) (bool, error) { panic("unused") }
func (f *previewFakeFile) GetJobWorkerLogURL(string) string                        { panic("unused") }
func (f *previewFakeFile) CheckJobWorkerLogExists(context.Context, string) (bool, error) {
	panic("unused")
}
func (f *previewFakeFile) GetJobUserFacingLogURL(string) string { panic("unused") }
func (f *previewFakeFile) CheckJobUserFacingLogExists(context.Context, string) (bool, error) {
	panic("unused")
}
func (f *previewFakeFile) GetIntermediateDataURL(context.Context, string, string) string {
	panic("unused")
}
func (f *previewFakeFile) CheckIntermediateDataExists(context.Context, string, string) (bool, error) {
	panic("unused")
}
func (f *previewFakeFile) IssueUploadAssetLink(context.Context, gateway.IssueUploadAssetParam) (*gateway.UploadAssetLink, error) {
	panic("unused")
}
func (f *previewFakeFile) GetPublicAssetURL(string, string) (*url.URL, error) { panic("unused") }
func (f *previewFakeFile) UploadedAsset(context.Context, *asset.Upload) (*file.File, error) {
	panic("unused")
}
func (f *previewFakeFile) WriteCancelFlag(context.Context, string) error { panic("unused") }
func (f *previewFakeFile) CancelFlagURI(string) string                   { panic("unused") }

// previewFakeJob records which dispatch seam the interactor used.
type previewFakeJob struct {
	previewCalls []gateway.ProbeSchemaParam
	runCalls     []gateway.RunJobParam
	monitored    int
}

func (j *previewFakeJob) PreviewSchemaCloudRunWorker(_ *job.Job, p gateway.ProbeSchemaParam) {
	j.previewCalls = append(j.previewCalls, p)
}
func (j *previewFakeJob) RunCloudRunWorker(_ *job.Job, p gateway.RunJobParam) {
	j.runCalls = append(j.runCalls, p)
}
func (j *previewFakeJob) StartMonitoring(context.Context, *job.Job, *string) error {
	j.monitored++
	return nil
}
func (j *previewFakeJob) Cancel(context.Context, id.JobID) (*job.Job, error) { panic("unused") }
func (j *previewFakeJob) Fetch(context.Context, []id.JobID) ([]*job.Job, error) {
	panic("unused")
}
func (j *previewFakeJob) FindByID(context.Context, id.JobID) (*job.Job, error) { panic("unused") }
func (j *previewFakeJob) FindByWorkspace(context.Context, accountsid.WorkspaceID, *interfaces.PaginationParam, *string) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	panic("unused")
}
func (j *previewFakeJob) GetStatus(context.Context, id.JobID) (job.Status, error) {
	panic("unused")
}
func (j *previewFakeJob) Subscribe(context.Context, id.JobID) (chan job.Status, error) {
	panic("unused")
}
func (j *previewFakeJob) Unsubscribe(id.JobID, chan job.Status) { panic("unused") }

// previewFakeBatch records SubmitProbeJob calls for the fallback test.
type previewFakeBatch struct {
	probeCalls int
	runCalls   int
}

func (b *previewFakeBatch) SubmitProbeJob(context.Context, id.JobID, string, map[string]string, *int, string, id.ProjectID, accountsid.WorkspaceID) (string, error) {
	b.probeCalls++
	return "projects/p/locations/l/jobs/probe", nil
}
func (b *previewFakeBatch) SubmitJob(context.Context, id.JobID, string, string, map[string]string, id.ProjectID, accountsid.WorkspaceID, *id.JobID, *uuid.UUID) (string, error) {
	b.runCalls++
	return "", nil
}
func (b *previewFakeBatch) GetJobStatus(context.Context, string) (gateway.JobStatus, error) {
	return gateway.JobStatusPending, nil
}
func (b *previewFakeBatch) ListJobs(context.Context, id.ProjectID) ([]gateway.JobInfo, error) {
	return nil, nil
}
func (b *previewFakeBatch) CancelJob(context.Context, string) error { return nil }

// --- tests ---------------------------------------------------------------

func previewTestContext() context.Context {
	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, &appx.AuthInfo{Token: "token"})
	return ctx
}

func newPreviewProject(t *testing.T, projectRepo repo.Project) *project.Project {
	t.Helper()
	prj := project.New().
		NewID().
		Workspace(project.NewWorkspaceID()).
		MustBuild()
	assert.NoError(t, projectRepo.Save(context.Background(), prj))
	return prj
}

func newWorkflowFile() *file.File {
	return &file.File{
		Content: io.NopCloser(nil),
		Path:    "workflow.json",
		Size:    1,
	}
}

func TestProject_PreviewSchema_CloudRunWorker(t *testing.T) {
	ctx := previewTestContext()
	projectRepo := memory.NewProject()
	jobRepo := memory.NewJob()
	ws := &fakeWebsocket{}
	ff := &previewFakeFile{}
	fj := &previewFakeJob{}
	crw := &stubCloudRunWorker{}

	prj := newPreviewProject(t, projectRepo)

	uc := &Project{
		projectRepo:       projectRepo,
		jobRepo:           jobRepo,
		websocket:         ws,
		file:              ff,
		cloudRunWorker:    crw, // non-nil => Cloud Run path
		job:               fj,
		transaction:       &usecasex.NopTransaction{},
		permissionChecker: NewMockPermissionChecker(nil),
	}

	got, err := uc.PreviewSchema(ctx, interfaces.PreviewSchemaParam{
		ProjectID:  prj.ID(),
		Workflow:   newWorkflowFile(),
		SampleSize: lo.ToPtr(25),
	})
	assert.NoError(t, err)
	assert.NotNil(t, got)

	// Job tagged as preview-schema and debug.
	assert.Equal(t, job.ModePreviewSchema, got.Mode())
	assert.NotNil(t, got.Debug())
	assert.True(t, *got.Debug())
	assert.Equal(t, prj.Workspace(), got.Workspace())
	assert.Equal(t, 7, *got.ProjectVersion())

	// Dedicated probe-schema dispatch invoked; run dispatch NOT invoked.
	assert.Len(t, fj.previewCalls, 1)
	assert.Len(t, fj.runCalls, 0)
	assert.Equal(t, got.ID(), fj.previewCalls[0].JobID)
	assert.Equal(t, 25, *fj.previewCalls[0].SampleSize)
	assert.Equal(t, "gs://bucket/workflows/wf.json", fj.previewCalls[0].WorkflowURL)
	assert.Equal(t, "gs://bucket/artifacts/"+got.ID().String()+"/schema/schema-report.json", fj.previewCalls[0].ReportURL)
	assert.Equal(t, 1, fj.monitored)

	// Metadata NOT uploaded; workflow uploaded once.
	assert.Equal(t, 0, ff.uploadMetadataCalls)
	assert.Equal(t, 1, ff.uploadWorkflowCalls)

	// Yjs flushed for the project.
	assert.Equal(t, []string{prj.ID().String()}, ws.flushed)

	// Persisted job exists and carries the mode.
	saved, err := jobRepo.FindByID(ctx, got.ID())
	assert.NoError(t, err)
	assert.Equal(t, job.ModePreviewSchema, saved.Mode())

	// Preview job is excluded from the project's run history.
	hist, err := jobRepo.FindByProject(ctx, prj.ID())
	assert.NoError(t, err)
	assert.Empty(t, hist)
}

func TestProject_PreviewSchema_BatchFallback(t *testing.T) {
	ctx := previewTestContext()
	projectRepo := memory.NewProject()
	jobRepo := memory.NewJob()
	ws := &fakeWebsocket{}
	ff := &previewFakeFile{}
	fj := &previewFakeJob{}
	fb := &previewFakeBatch{}

	prj := newPreviewProject(t, projectRepo)

	uc := &Project{
		projectRepo:       projectRepo,
		jobRepo:           jobRepo,
		websocket:         ws,
		file:              ff,
		batch:             fb,
		cloudRunWorker:    nil, // nil => Batch fallback
		job:               fj,
		transaction:       &usecasex.NopTransaction{},
		permissionChecker: NewMockPermissionChecker(nil),
	}

	got, err := uc.PreviewSchema(ctx, interfaces.PreviewSchemaParam{
		ProjectID: prj.ID(),
		Workflow:  newWorkflowFile(),
	})
	assert.NoError(t, err)
	assert.NotNil(t, got)

	// Batch probe submission used; run submission NOT used.
	assert.Equal(t, 1, fb.probeCalls)
	assert.Equal(t, 0, fb.runCalls)
	// No Cloud Run dispatch on the fallback path.
	assert.Len(t, fj.previewCalls, 0)
	assert.Equal(t, 1, fj.monitored)
	assert.Equal(t, 0, ff.uploadMetadataCalls)
}

// stubCloudRunWorker satisfies gateway.CloudRunWorker; not exercised directly in
// the interactor tests (dispatch is recorded by previewFakeJob) but required to
// set the non-nil cloudRunWorker field that selects the Cloud Run path.
type stubCloudRunWorker struct{}

func (s *stubCloudRunWorker) RunJob(context.Context, gateway.RunJobParam) (gateway.JobStatus, error) {
	return gateway.JobStatusCompleted, nil
}
func (s *stubCloudRunWorker) PreviewSchema(context.Context, gateway.ProbeSchemaParam) (gateway.JobStatus, error) {
	return gateway.JobStatusCompleted, nil
}
func (s *stubCloudRunWorker) CancelJob(context.Context, id.JobID) error { return nil }

func TestProject_PreviewSchema_RequiresWorkflow(t *testing.T) {
	prj := project.New().NewID().Workspace(project.NewWorkspaceID()).MustBuild()
	uc := &Project{
		permissionChecker: NewMockPermissionChecker(nil),
	}

	got, err := uc.PreviewSchema(previewTestContext(), interfaces.PreviewSchemaParam{
		ProjectID: prj.ID(),
		Workflow:  nil,
	})

	// A missing workflow yields a clear error, not a nil job (which would violate
	// the non-null PreviewSchemaPayload.job at the GraphQL layer).
	assert.Nil(t, got)
	assert.ErrorIs(t, err, interfaces.ErrWorkflowFileRequired)
}

func TestParametersToVariables(t *testing.T) {
	mustParam := func(name string, def any) *parameter.Parameter {
		p, err := parameter.New().Name(name).DefaultValue(def).Build()
		assert.NoError(t, err)
		return p
	}

	assert.Nil(t, parametersToVariables(nil))

	vars := parametersToVariables([]*parameter.Parameter{
		mustParam("dataset", "file:///a.geojson"),
		mustParam("noDefault", nil), // must be omitted, never the literal "<nil>"
		nil,                         // nil entry skipped
	})

	assert.Equal(t, "file:///a.geojson", vars["dataset"])
	_, ok := vars["noDefault"]
	assert.False(t, ok, "parameter with nil default must be omitted, not set to \"<nil>\"")
}
