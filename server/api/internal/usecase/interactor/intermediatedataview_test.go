package interactor

import (
	"context"
	"errors"
	"fmt"
	"io"
	"strings"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// --- fakes ---------------------------------------------------------------

// viewFakeFile serves one report and records what was asked of it. Like real
// storage, the report stays until something replaces or deletes it.
type viewFakeFile struct {
	mockCheckStatusFile
	reportErr      error
	deleteErr      error
	report         string
	lastReportKey  string
	lastExistsName string
	// missingName makes one file of the view absent, when set.
	missingName   string
	reportReads   int
	resolveCalls  int
	existsChecks  int
	reportDeletes int
	inputMissing  bool
	// viewMissing makes the view absent while its report stays in place, which
	// is what a retention rule sweeping the rendered files looks like.
	viewMissing bool
}

func (f *viewFakeFile) CheckFeatureViewFileExists(_ context.Context, _, _, name string) (bool, error) {
	f.existsChecks++
	f.lastExistsName = name
	return !f.viewMissing && name != f.missingName, nil
}

func (f *viewFakeFile) ResolveIntermediateDataURI(_ context.Context, jobID, fileID string) (string, bool, error) {
	f.resolveCalls++
	if f.inputMissing {
		return "", false, nil
	}
	return "gs://bucket/artifacts/" + jobID + "/feature-store/" + fileID + ".jsonl.zst", true, nil
}

func (f *viewFakeFile) GetFeatureViewUploadURI(jobID, fileID string) string {
	return "gs://bucket/artifacts/" + jobID + "/feature-view/" + fileID
}

func (f *viewFakeFile) GetFeatureViewReportUploadURI(jobID, fileID, key string) string {
	return f.GetFeatureViewUploadURI(jobID, fileID) + "/" + featureview.ReportName(key)
}

func (f *viewFakeFile) GetFeatureViewURL(jobID, fileID, name string) string {
	return "https://api.example/artifacts/" + jobID + "/feature-view/" + fileID + "/" + name
}

func (f *viewFakeFile) ReadFeatureViewReport(_ context.Context, _, _, key string) (io.ReadCloser, error) {
	f.reportReads++
	f.lastReportKey = key
	if f.reportErr != nil {
		return nil, f.reportErr
	}
	if f.report == "" {
		return nil, rerror.ErrNotFound
	}
	return io.NopCloser(strings.NewReader(f.report)), nil
}

func (f *viewFakeFile) DeleteFeatureViewReport(context.Context, string, string, string) error {
	f.reportDeletes++
	if f.deleteErr != nil {
		return f.deleteErr
	}
	f.report = ""
	return nil
}

// viewFakeWorker stands in for the engine worker: it records the call and
// writes whatever report the test wants the render to have produced.
type viewFakeWorker struct {
	err error
	// versionErr and version are what EngineVersion answers.
	versionErr error
	file       *viewFakeFile
	// writes is the report the render leaves behind, or "" for a render that
	// recorded nothing — which, as with real storage, leaves whatever report was
	// already there untouched.
	writes       string
	status       gateway.JobStatus
	version      string
	lastParam    gateway.RenderViewParam
	calls        int
	versionCalls int
}

func (w *viewFakeWorker) RenderView(_ context.Context, p gateway.RenderViewParam) (gateway.JobStatus, error) {
	w.calls++
	w.lastParam = p
	if w.writes != "" {
		w.file.report = w.writes
		w.file.viewMissing = false
	}
	if w.status == "" {
		w.status = gateway.JobStatusCompleted
	}
	return w.status, w.err
}

func (w *viewFakeWorker) RunJob(context.Context, gateway.RunJobParam) (gateway.JobStatus, error) {
	panic("unused")
}
func (w *viewFakeWorker) PreviewSchema(context.Context, gateway.ProbeSchemaParam) (gateway.JobStatus, error) {
	panic("unused")
}
func (w *viewFakeWorker) CancelJob(context.Context, id.JobID) error { panic("unused") }
func (w *viewFakeWorker) EngineVersion(context.Context) (string, error) {
	w.versionCalls++
	return w.version, w.versionErr
}

// --- harness -------------------------------------------------------------

const testFileID = "7c9e6679-7425-40de-944b-e07fc1f90ae7.default"

func viewTestContext() context.Context {
	return adapter.AttachAuthInfo(context.Background(), &appx.AuthInfo{Token: "token"})
}

type viewHarness struct {
	uc     *IntermediateDataView
	file   *viewFakeFile
	worker *viewFakeWorker
	source *job.Job
}

func newViewHarness(t *testing.T, status job.Status, withWorker bool) *viewHarness {
	t.Helper()

	jobRepo := memory.NewJob()
	projectID := id.NewProjectID()
	source := job.New().
		NewID().
		Mode(job.ModeRun).
		Workspace(accountsid.NewWorkspaceID()).
		ProjectID(&projectID).
		Status(status).
		StartedAt(time.Now()).
		MustBuild()
	require.NoError(t, jobRepo.Save(context.Background(), source))

	h := &viewHarness{file: &viewFakeFile{}, source: source}
	h.worker = &viewFakeWorker{file: h.file}
	h.uc = &IntermediateDataView{
		jobRepo:           jobRepo,
		file:              h.file,
		permissionChecker: NewMockPermissionChecker(nil),
	}
	if withWorker {
		h.uc.cloudRunWorker = h.worker
	}
	return h
}

func (h *viewHarness) render(ctx context.Context, req featureview.Request) (*interfaces.IntermediateDataViewResult, error) {
	return h.uc.Render(ctx, interfaces.RenderIntermediateDataViewParam{
		JobID:   h.source.ID(),
		FileID:  testFileID,
		Request: req,
	})
}

// key is the key Render derives for req: a tiles key covers where the view is
// served, which only the harness's storage knows, and every key covers the
// engine the harness's worker names.
func (h *viewHarness) key(req featureview.Request) string {
	req.ServedFrom = h.file.GetFeatureViewURL(h.source.ID().String(), testFileID, "")
	if h.worker != nil && h.worker.versionErr == nil {
		req.EngineVersion = h.worker.version
	}
	return req.Key()
}

func gltfReq(r int) featureview.Request {
	return featureview.Request{
		Shape:     featureview.ShapeGLTF,
		Selection: featureview.Selection{Row: &r},
		Options:   featureview.DefaultOptions(),
	}
}

func tilesReq() featureview.Request {
	return featureview.Request{Shape: featureview.ShapeTiles, Options: featureview.DefaultOptions()}
}

// --- tests ---------------------------------------------------------------

func TestIntermediateDataView_Render_ReusesARenderedView(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(42).Key()
	h.file.report = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":42,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":43,"entryPoint":"` + key + `.glb"}`

	got, err := h.render(viewTestContext(), gltfReq(42))
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusReady, got.Status)
	assert.Equal(t, key, got.Key)
	require.Len(t, got.Layers, 1)
	assert.Equal(t, featureview.FormatGLB, got.Layers[0].Format)
	assert.Equal(t,
		"https://api.example/artifacts/"+h.source.ID().String()+"/feature-view/"+testFileID+"/"+key+".glb",
		got.Layers[0].EntryPointURL)
	require.NotNil(t, got.RenderedFeatures)
	assert.Equal(t, 1, *got.RenderedFeatures)

	assert.Zero(t, h.worker.calls, "an existing view is not re-rendered")
	assert.Zero(t, h.file.resolveCalls, "the input is not even resolved on a cache hit")
	assert.Zero(t, h.file.reportDeletes, "a reused report is kept")
	assert.Equal(t, key, h.file.lastReportKey)
}

// A retention rule can sweep the view and leave its report. Reusing that report
// would pin the port to an entry point that 404s, forever.
func TestIntermediateDataView_Render_RerendersWhenTheViewIsGone(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(42).Key()
	ready := `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":42,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":43,"entryPoint":"` + key + `.glb"}`
	h.file.report = ready
	h.file.viewMissing = true
	h.worker.writes = ready

	got, err := h.render(viewTestContext(), gltfReq(42))
	require.NoError(t, err)

	assert.Equal(t, 1, h.worker.calls, "the report outlived its view, so render again")
	assert.Equal(t, key+".glb", h.file.lastExistsName, "the entry point is what gets checked")
	assert.Equal(t, featureview.StatusReady, got.Status)
}

func TestIntermediateDataView_Render_ChecksTheViewBeforeReusingIt(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := h.key(tilesReq())
	h.file.report = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
	  "selectedFeatures":3,"renderedFeatures":3,"scanned":3,"entryPoint":"` + key + `/tilejson.json"}`

	_, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, 1, h.file.existsChecks)
	assert.Equal(t, key+"/tilejson.json", h.file.lastExistsName)
	assert.Zero(t, h.worker.calls)
}

// An empty view wrote no files, so its report stands on its own.
func TestIntermediateDataView_Render_DoesNotLookForAnEmptyViewsFiles(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.report = `{"version":1,"status":"empty","shape":"tiles",
	  "selectedFeatures":0,"renderedFeatures":0,"scanned":900}`
	h.file.viewMissing = true

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusEmpty, got.Status)
	assert.Zero(t, h.file.existsChecks, "an empty view has no entry point to check")
	assert.Zero(t, h.worker.calls, "and is not re-rendered for the lack of one")
}

// The report's entryPoint would reach a path join, where ".." escapes the view's
// directory, so it is cross-checked and not followed.
func TestIntermediateDataView_Render_DerivesTheEntryPointRatherThanTrustingTheReport(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(7).Key()
	h.file.report = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":7,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":8,
	  "entryPoint":"../../../../metadata/job-secrets.json"}`

	got, err := h.render(viewTestContext(), gltfReq(7))
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusReady, got.Status)
	require.Len(t, got.Layers, 1)
	assert.Equal(t,
		"https://api.example/artifacts/"+h.source.ID().String()+"/feature-view/"+testFileID+"/"+key+".glb",
		got.Layers[0].EntryPointURL,
		"the served URL follows the key and format, not the report")
	assert.NotContains(t, got.Layers[0].EntryPointURL, "..")
}

func TestIntermediateDataView_Get_FailsAReadyReportWithNoFormat(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := tilesReq().Key()
	// Bypasses ParseReport's own guard by going through Get with a report the
	// server built rather than parsed.
	got := h.uc.result(viewTestContext(), h.source.ID(), testFileID, key, featureview.ShapeTiles,
		&featureview.Report{Version: featureview.ReportVersion, Status: featureview.StatusReady})

	assert.Equal(t, featureview.StatusFailed, got.Status)
	assert.Empty(t, got.Layers)
	require.NotNil(t, got.Error)
}

// Geometry with no CRS, or that the writer cannot draw, is dropped on purpose,
// so a partial render is READY. Both counts come back so a client can say so.
func TestIntermediateDataView_Render_ReportsAPartialDropAsReady(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := h.key(tilesReq())
	h.file.report = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
	  "selectedFeatures":900,"renderedFeatures":812,"scanned":900,
	  "entryPoint":"` + key + `/tilejson.json"}`

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusReady, got.Status)
	require.NotNil(t, got.SelectedFeatures)
	require.NotNil(t, got.RenderedFeatures)
	assert.Equal(t, 900, *got.SelectedFeatures)
	assert.Equal(t, 812, *got.RenderedFeatures)
	assert.NotEmpty(t, got.Layers)
}

func TestIntermediateDataView_Render_CarriesTheSizeLimitedCountsOfVectorTiles(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := tilesReq().Key()
	h.file.report = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
	  "selectedFeatures":900,"renderedFeatures":812,"scanned":900,
	  "sizeLimitedFeatures":88,"sizeLimitedTiles":3,
	  "entryPoint":"` + key + `/tilejson.json"}`

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	require.NotNil(t, got.SizeLimitedFeatures)
	require.NotNil(t, got.SizeLimitedTiles)
	assert.Equal(t, 88, *got.SizeLimitedFeatures)
	assert.Equal(t, 3, *got.SizeLimitedTiles)
}

// Other formats get null rather than a zero that would claim nothing was left
// out, even from a report that carries counts.
func TestIntermediateDataView_HasNoSizeLimitedCountsOutsideVectorTiles(t *testing.T) {
	for name, tc := range map[string]struct {
		report func(key string) string
		req    featureview.Request
	}{
		"glb": {req: gltfReq(1), report: func(key string) string {
			return `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":1,
			  "selectedFeatures":1,"renderedFeatures":1,"scanned":2,
			  "sizeLimitedFeatures":5,"sizeLimitedTiles":5,"entryPoint":"` + key + `.glb"}`
		}},
		"3D Tiles": {req: tilesReq(), report: func(key string) string {
			return `{"version":1,"status":"ready","shape":"tiles","format":"cesium_3d_tiles",
			  "selectedFeatures":2,"renderedFeatures":2,"scanned":2,
			  "sizeLimitedFeatures":5,"sizeLimitedTiles":5,"entryPoint":"` + key + `/tileset.json"}`
		}},
		"unreadable report": {req: tilesReq(), report: func(string) string { return `{"version":2,"status":"ready"}` }},
	} {
		t.Run(name, func(t *testing.T) {
			h := newViewHarness(t, job.StatusCompleted, true)
			key := tc.req.Key()
			h.file.report = tc.report(key)

			got, err := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, key)
			require.NoError(t, err)
			require.NotNil(t, got)

			assert.Nil(t, got.SizeLimitedFeatures)
			assert.Nil(t, got.SizeLimitedTiles)
		})
	}
}

func TestIntermediateDataView_Render_KeepsTheCountsOnAnEmptyView(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.report = `{"version":1,"status":"empty","shape":"tiles",
	  "selectedFeatures":900,"renderedFeatures":0,"scanned":900,
	  "error":"none of the 900 selected features carried geometry the view draws"}`

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusEmpty, got.Status)
	assert.Empty(t, got.Layers, "a view that wrote nothing has no layers")
	require.NotNil(t, got.SelectedFeatures)
	require.NotNil(t, got.RenderedFeatures)
	assert.Equal(t, 900, *got.SelectedFeatures)
	assert.Equal(t, 0, *got.RenderedFeatures)
}

func TestIntermediateDataView_Render_RendersAndReturnsTheFinishedView(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(42).Key()
	h.worker.writes = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":42,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":43,"entryPoint":"` + key + `.glb"}`

	got, err := h.render(viewTestContext(), gltfReq(42))
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusReady, got.Status)
	require.Len(t, got.Layers, 1)
	assert.Equal(t, featureview.FormatGLB, got.Layers[0].Format)
	assert.Contains(t, got.Layers[0].EntryPointURL, key+".glb")

	require.Equal(t, 1, h.worker.calls)
	assert.Zero(t, h.file.reportDeletes, "a first render has no report to clear")
	p := h.worker.lastParam
	assert.Equal(t, key, p.Name)
	assert.Equal(t, featureview.ShapeGLTF, p.Shape)
	require.NotNil(t, p.Row)
	assert.Equal(t, 42, *p.Row)
	assert.Contains(t, p.InputURI, "/feature-store/"+testFileID)
	assert.Contains(t, p.OutputURI, "/feature-view/"+testFileID)
	assert.Contains(t, p.ReportURI, featureview.ReportName(key))
}

// A tiles view is also keyed by where it is served. A glb view is told no URL.
func TestIntermediateDataView_Render_TellsATilesViewWhereItIsServed(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := h.key(tilesReq())
	h.worker.writes = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":1,"entryPoint":"` + key + `/tilejson.json"}`

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)
	assert.Equal(t, key, got.Key, "the key covers where the view is served")

	p := h.worker.lastParam
	require.NotNil(t, p.TilesURL)
	assert.Equal(t,
		"https://api.example/artifacts/"+h.source.ID().String()+"/feature-view/"+testFileID+"/"+key+"/{z}/{x}/{y}.mvt",
		*p.TilesURL)
	require.Len(t, got.Layers, 1)
	assert.Equal(t, strings.TrimSuffix(got.Layers[0].EntryPointURL, "tilejson.json")+"{z}/{x}/{y}.mvt", *p.TilesURL,
		"the tiles sit beside the tilejson the caller is handed")

	g := newViewHarness(t, job.StatusCompleted, true)
	g.worker.writes = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":1,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":2,"entryPoint":"` + gltfReq(1).Key() + `.glb"}`
	_, err = g.render(viewTestContext(), gltfReq(1))
	require.NoError(t, err)
	require.Equal(t, 1, g.worker.calls)
	assert.Nil(t, g.worker.lastParam.TilesURL)
}

func TestIntermediateDataView_Render_KeysAViewByTheWorkersEngine(t *testing.T) {
	for name, req := range map[string]featureview.Request{"gltf": gltfReq(1), "tiles": tilesReq()} {
		h := newViewHarness(t, job.StatusCompleted, true)
		unversioned := h.key(req)
		h.worker.version = "0.0.583"

		_, _ = h.render(viewTestContext(), req)

		require.Equal(t, 1, h.worker.calls, name)
		assert.Equal(t, h.key(req), h.worker.lastParam.Name, name)
		assert.NotEqual(t, unversioned, h.worker.lastParam.Name, name)
	}
}

// A worker that cannot report its engine version still renders, under the
// unversioned key.
func TestIntermediateDataView_Render_RendersWhenTheEngineIsUnknown(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.worker.versionErr = errors.New("cloudrunworker: version: http 404")
	key := tilesReq()
	key.ServedFrom = h.file.GetFeatureViewURL(h.source.ID().String(), testFileID, "")
	unversioned := key.Key()
	h.worker.writes = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":1,"entryPoint":"` + unversioned + `/tilejson.json"}`

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, 1, h.worker.calls)
	assert.Equal(t, unversioned, got.Key)
}

// The report is read before the call's status is judged, because a render that
// drew nothing exits successfully and explains itself there.
func TestIntermediateDataView_Render_PrefersTheReportOverTheCallStatus(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.worker.writes = `{"version":1,"status":"unsupported_geometry","shape":"gltf","row":7,
	  "selectedFeatures":1,"renderedFeatures":0,"scanned":8,
	  "error":"A glTF view needs 3D geometry, and this row holds 2D geometry"}`

	got, err := h.render(viewTestContext(), gltfReq(7))
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusUnsupportedGeometry, got.Status)
	require.NotNil(t, got.Error)
	assert.Contains(t, *got.Error, "needs 3D geometry")
}

// A render the worker stopped at its own time limit ended inside the request's
// wait, so it is answered from its failed report rather than as a timeout.
func TestIntermediateDataView_Render_AnswersAStoppedRenderFromItsReport(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.worker.status = gateway.JobStatusFailed
	h.worker.err = errors.New("cloudrunworker: render-view failed (http 500): " +
		"the render did not finish within 100 seconds and was stopped")
	h.worker.writes = `{"version":1,"status":"failed","shape":"tiles",
	  "selectedFeatures":0,"renderedFeatures":0,"scanned":0,
	  "error":"the render did not finish within 100 seconds and was stopped"}`

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusFailed, got.Status)
	require.NotNil(t, got.Error)
	assert.Contains(t, *got.Error, "did not finish within 100 seconds")
}

// The failure detail is infrastructure's (worker stderr, storage paths), so it
// is logged and not returned to the caller.
func TestIntermediateDataView_Render_FailsWhenNoReportIsWritten(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.worker.status = gateway.JobStatusFailed
	h.worker.err = errors.New("worker exit: signal 9 writing gs://bucket/artifacts/x")

	_, err := h.render(viewTestContext(), gltfReq(0))
	require.Error(t, err)
	assert.ErrorIs(t, err, h.worker.err)
	assert.Equal(t, "failed to render the view", err.Error())
}

// A worker that claims success but leaves no report is a fault, not an empty
// view.
func TestIntermediateDataView_Render_FailsOnASilentSuccess(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)

	_, err := h.render(viewTestContext(), gltfReq(0))
	require.Error(t, err)
	assert.Equal(t, "failed to render the view", err.Error())
}

func TestIntermediateDataView_Render_UnavailableWithoutAWorker(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, false)

	_, err := h.render(viewTestContext(), gltfReq(0))
	assert.ErrorIs(t, err, interfaces.ErrViewsUnavailable)
	assert.Zero(t, h.worker.calls)
}

// Empty and unsupported-geometry outcomes come from the data, so they are served
// from the report. A fault might be transient, so it is re-rendered.
func TestIntermediateDataView_Render_CachesOutcomesButNotFaults(t *testing.T) {
	tests := map[string]struct {
		status     featureview.Status
		rerendered bool
	}{
		"empty":                {featureview.StatusEmpty, false},
		"unsupported geometry": {featureview.StatusUnsupportedGeometry, false},
		"failed":               {featureview.StatusFailed, true},
	}

	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			h := newViewHarness(t, job.StatusCompleted, true)
			h.file.report = `{"version":1,"status":"` + string(tt.status) + `","shape":"gltf","row":9,
			  "selectedFeatures":1,"renderedFeatures":0,"scanned":10,"error":"nothing to draw"}`
			h.worker.writes = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":9,
			  "selectedFeatures":1,"renderedFeatures":1,"scanned":10,
			  "entryPoint":"` + gltfReq(9).Key() + `.glb"}`

			got, err := h.render(viewTestContext(), gltfReq(9))
			require.NoError(t, err)

			if !tt.rerendered {
				assert.Equal(t, tt.status, got.Status)
				require.NotNil(t, got.Error)
				assert.Equal(t, "nothing to draw", *got.Error)
				assert.Zero(t, h.worker.calls)
				return
			}

			assert.Equal(t, 1, h.worker.calls, "a fault is worth retrying")
		})
	}
}

func TestIntermediateDataView_Render_TreatsAnUnreadableReportAsAFault(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.report = `{"version":1,"status":"ready","shape":"gltf"}` // ready with no entry point

	key := gltfReq(1).Key()
	h.worker.writes = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":1,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":2,"entryPoint":"` + key + `.glb"}`

	got, err := h.render(viewTestContext(), gltfReq(1))
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusReady, got.Status)
	assert.Equal(t, 1, h.worker.calls,
		"an unreadable report is a fault, and faults are re-rendered")
}

// The feature writer appends as the run proceeds, so a file belonging to an
// unfinished job may be truncated mid-feature.
func TestIntermediateDataView_Render_RefusesAnUnfinishedJob(t *testing.T) {
	for _, status := range []job.Status{job.StatusPending, job.StatusRunning} {
		t.Run(string(status), func(t *testing.T) {
			h := newViewHarness(t, status, true)

			_, err := h.render(viewTestContext(), gltfReq(0))
			assert.ErrorIs(t, err, interfaces.ErrJobNotFinished)
			assert.Zero(t, h.worker.calls)
		})
	}

	// A failed or cancelled run still leaves data for the ports that did run.
	for _, status := range []job.Status{job.StatusCompleted, job.StatusFailed, job.StatusCancelled} {
		t.Run(string(status), func(t *testing.T) {
			h := newViewHarness(t, status, true)

			_, err := h.render(viewTestContext(), gltfReq(0))
			assert.NotErrorIs(t, err, interfaces.ErrJobNotFinished)
			assert.Equal(t, 1, h.worker.calls, "the render was attempted")
		})
	}
}

func TestIntermediateDataView_Render_RefusesMissingInput(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.inputMissing = true

	_, err := h.render(viewTestContext(), gltfReq(0))
	assert.ErrorIs(t, err, interfaces.ErrIntermediateDataNotFound)
	assert.Zero(t, h.worker.calls, "nothing is rendered for input that is not there")
}

// The file id is client-supplied and is used to build a WRITE path.
func TestIntermediateDataView_Render_ValidatesTheFileID(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)

	_, err := h.uc.Render(viewTestContext(), interfaces.RenderIntermediateDataViewParam{
		JobID:   h.source.ID(),
		FileID:  "../../etc/passwd",
		Request: gltfReq(0),
	})
	assert.ErrorIs(t, err, featureview.ErrInvalidFileID)
	assert.Zero(t, h.file.reportReads, "a bad id never reaches storage")
}

func TestIntermediateDataView_Render_ValidatesTheRequest(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)

	// A glb with no row: the engine CLI rejects this at parse time.
	_, err := h.render(viewTestContext(), featureview.Request{
		Shape:   featureview.ShapeGLTF,
		Options: featureview.DefaultOptions(),
	})
	assert.ErrorIs(t, err, featureview.ErrInvalidRequest)
	assert.Zero(t, h.worker.calls)
}

// The checker mirrors the generated edge policy, which denies an action it has
// no rule for even to an owner. The allow-all mock could not catch that.
func TestIntermediateDataView_IsAuthorizedByTheEdgePolicy(t *testing.T) {
	for _, tc := range []struct {
		role    string
		allowed bool
	}{
		{role: "owner", allowed: true},
		{role: "maintainer", allowed: true},
		{role: "writer", allowed: true},
		{role: "reader", allowed: false},
	} {
		t.Run(tc.role, func(t *testing.T) {
			h := newViewHarness(t, job.StatusCompleted, true)
			h.uc.permissionChecker = policyChecker(tc.role)
			key := gltfReq(0).Key()
			h.worker.writes = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":0,
			  "selectedFeatures":1,"renderedFeatures":1,"scanned":1,"entryPoint":"` + key + `.glb"}`

			_, renderErr := h.render(viewTestContext(), gltfReq(0))
			_, getErr := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, key)

			if tc.allowed {
				assert.NoError(t, renderErr)
				assert.NoError(t, getErr)
				return
			}
			assert.ErrorIs(t, renderErr, interfaces.ErrOperationDenied)
			assert.ErrorIs(t, getErr, interfaces.ErrOperationDenied)
			assert.Zero(t, h.worker.calls)
		})
	}
}

func TestIntermediateDataView_Get(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := tilesReq().Key()
	h.file.report = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
	  "renderedFeatures":812,"scanned":900,"entryPoint":"` + key + `/tilejson.json"}`

	got, err := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, key)
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusReady, got.Status)
	assert.Equal(t, featureview.ShapeTiles, got.Shape, "the shape comes back from the report")
	require.Len(t, got.Layers, 1)
	assert.Equal(t, featureview.FormatVectorTiles, got.Layers[0].Format, "an all-2D edge renders vector tiles")
	assert.Contains(t, got.Layers[0].EntryPointURL, key+"/tilejson.json")
	assert.Zero(t, h.worker.calls, "Get reads an existing view and never renders")
}

func TestIntermediateDataView_Get_NoViewWhenNoneExists(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)

	got, err := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, "tiles-all-deadbeef")
	require.NoError(t, err)
	assert.Nil(t, got)
}

func TestIntermediateDataView_Get_RejectsAnUnknownJob(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)

	_, err := h.uc.Get(viewTestContext(), id.NewJobID(), testFileID, "tiles-all-deadbeef")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

// A storage fault must not read as "no view yet", which would re-dispatch.
func TestIntermediateDataView_Render_PropagatesAStorageFault(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.reportErr = errors.New("gcs unavailable")

	_, err := h.render(viewTestContext(), gltfReq(0))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "gcs unavailable")
	assert.Zero(t, h.worker.calls)
}

// The key comes straight from the query and reaches storage as a path segment.
// A key that could climb out of the view's directory is refused before any read.
func TestIntermediateDataView_Get_RefusesATraversingKey(t *testing.T) {
	for _, key := range []string{"../../secret", "..", "a/b", "tiles-all-dead.beef"} {
		h := newViewHarness(t, job.StatusCompleted, true)
		h.file.report = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
		  "renderedFeatures":1,"scanned":1,"entryPoint":"x/tilejson.json"}`

		got, err := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, key)

		assert.ErrorIs(t, err, featureview.ErrInvalidViewKey, "key %q must be refused", key)
		assert.Nil(t, got)
		assert.Zero(t, h.file.reportReads, "key %q must be refused before any storage read", key)
	}
}

// Retention can leave a ready report whose view is gone. Get cannot re-render,
// so it must not answer READY with a URL that 404s.
func TestIntermediateDataView_Get_NoViewWhenTheViewWasSwept(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := tilesReq().Key()
	h.file.report = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
	  "renderedFeatures":812,"scanned":900,"entryPoint":"` + key + `/tilejson.json"}`
	h.file.viewMissing = true

	got, err := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, key)

	require.NoError(t, err)
	assert.Nil(t, got, "a swept view is no view, not a READY one")
	assert.Zero(t, h.worker.calls, "Get never renders, even when the view is gone")
}

// A render that dies first writes nothing, so the stale report must be cleared
// beforehand or the read-back would answer with it.
func TestIntermediateDataView_Render_NeverAnswersWithTheReportItReplaces(t *testing.T) {
	key := gltfReq(42).Key()
	tests := map[string]struct {
		cached      string
		viewMissing bool
	}{
		"a ready report whose view was swept": {
			cached: `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":42,
			  "selectedFeatures":1,"renderedFeatures":1,"scanned":43,"entryPoint":"` + key + `.glb"}`,
			viewMissing: true,
		},
		"an earlier failure": {
			cached: `{"version":1,"status":"failed","shape":"gltf","row":42,
			  "selectedFeatures":0,"renderedFeatures":0,"scanned":0,"error":"an earlier fault"}`,
		},
	}

	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			h := newViewHarness(t, job.StatusCompleted, true)
			h.file.report = tt.cached
			h.file.viewMissing = tt.viewMissing
			h.worker.status = gateway.JobStatusFailed
			h.worker.err = fmt.Errorf("Post /render-view: %w", context.DeadlineExceeded)

			got, err := h.render(viewTestContext(), gltfReq(42))

			require.Error(t, err, "the old report must not stand in for this render's outcome")
			assert.Nil(t, got)
			assert.ErrorIs(t, err, interfaces.ErrRenderTimedOut, "the real cause reaches the caller")
			assert.Equal(t, 1, h.file.reportDeletes)
			assert.Equal(t, 1, h.worker.calls)
		})
	}
}

func TestIntermediateDataView_Render_DoesNotRenderOverAReportItCannotClear(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.report = `{"version":1,"status":"failed","shape":"gltf","row":3,
	  "selectedFeatures":0,"renderedFeatures":0,"scanned":0,"error":"an earlier fault"}`
	h.file.deleteErr = errors.New("gcs unavailable")

	_, err := h.render(viewTestContext(), gltfReq(3))

	require.Error(t, err)
	assert.ErrorIs(t, err, h.file.deleteErr)
	assert.NotContains(t, err.Error(), "gcs unavailable", "storage detail is logged, not returned")
	assert.Zero(t, h.worker.calls)
}

// An unparseable report is answered as a failed view of the kind the key names,
// with no counts. Describing a glb view as tiles would open the wrong viewer.
func TestIntermediateDataView_Get_DescribesAnUnreadableReportByItsKey(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(3).Key()
	h.file.report = `{"version":2,"status":"ready"}`

	got, err := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, key)
	require.NoError(t, err)
	require.NotNil(t, got)

	assert.Equal(t, featureview.StatusFailed, got.Status)
	assert.Equal(t, featureview.ShapeGLTF, got.Shape)
	assert.Nil(t, got.SelectedFeatures)
	assert.Nil(t, got.RenderedFeatures)
}

func TestIntermediateDataView_Render_ReportsATimeoutAsATimeout(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.worker.status = gateway.JobStatusFailed
	h.worker.err = fmt.Errorf("Post /render-view: %w", context.DeadlineExceeded)

	_, err := h.render(viewTestContext(), tilesReq())

	assert.ErrorIs(t, err, interfaces.ErrRenderTimedOut)
	assert.Equal(t, "the view did not finish rendering in time", err.Error())
}

func TestIntermediateDataView_Render_ReportsAFullWorkerPoolAsBusy(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.worker.status = gateway.JobStatusFailed
	h.worker.err = fmt.Errorf("cloudrunworker: render-view refused (http 429): %w", gateway.ErrWorkerBusy)

	_, err := h.render(viewTestContext(), tilesReq())

	assert.ErrorIs(t, err, interfaces.ErrViewWorkersBusy)
	assert.Equal(t, "no worker is free to render the view", err.Error())
}

// Storage that cannot hold views, such as the local filesystem, gets the same
// answer as having no worker.
func TestIntermediateDataView_ViewsUnavailableOnStorageWithoutViews(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.reportErr = gateway.ErrUnsupportedOperation

	_, err := h.render(viewTestContext(), tilesReq())
	assert.ErrorIs(t, err, interfaces.ErrViewsUnavailable)
	assert.Zero(t, h.worker.calls)

	_, err = h.uc.Get(viewTestContext(), h.source.ID(), testFileID, tilesReq().Key())
	assert.ErrorIs(t, err, interfaces.ErrViewsUnavailable)
}

// twoLayerReport is a ready tiles report with 3D Tiles for 3D features and
// vector tiles, at vectorEntryPoint, for 2D ones.
func twoLayerReport(key, vectorEntryPoint string) string {
	return `{"version":1,"status":"ready","shape":"tiles","format":"cesium_3d_tiles",
	  "selectedFeatures":16,"renderedFeatures":15,"scanned":16,
	  "entryPoint":"` + key + `/tileset.json",
	  "layers":[
	    {"format":"cesium_3d_tiles","entryPoint":"` + key + `/tileset.json",
	     "selectedFeatures":8,"renderedFeatures":8},
	    {"format":"vector_tiles","entryPoint":"` + vectorEntryPoint + `",
	     "selectedFeatures":9,"renderedFeatures":7,"sizeLimitedFeatures":2,"sizeLimitedTiles":1}
	  ]}`
}

// Each layer carries its own counts, and the view's size-limited counts are its
// vector tile layer's.
func TestIntermediateDataView_Render_ServesEveryLayerOfAView(t *testing.T) {
	t.Parallel()

	h := newViewHarness(t, job.StatusCompleted, true)
	key := h.key(tilesReq())
	h.file.report = twoLayerReport(key, key+"/tilejson.json")

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	base := "https://api.example/artifacts/" + h.source.ID().String() + "/feature-view/" + testFileID + "/" + key
	require.Len(t, got.Layers, 2)
	assert.Equal(t, featureview.FormatCesium3DTiles, got.Layers[0].Format)
	assert.Equal(t, base+"/tileset.json", got.Layers[0].EntryPointURL)
	assert.Equal(t, 8, got.Layers[0].SelectedFeatures)
	assert.Equal(t, 8, got.Layers[0].RenderedFeatures)
	assert.Nil(t, got.Layers[0].SizeLimitedFeatures, "3D Tiles have no size limit")
	assert.Equal(t, featureview.FormatVectorTiles, got.Layers[1].Format)
	assert.Equal(t, base+"/tilejson.json", got.Layers[1].EntryPointURL)
	assert.Equal(t, 9, got.Layers[1].SelectedFeatures)
	assert.Equal(t, 7, got.Layers[1].RenderedFeatures)
	require.NotNil(t, got.Layers[1].SizeLimitedFeatures)
	require.NotNil(t, got.Layers[1].SizeLimitedTiles)
	assert.Equal(t, 2, *got.Layers[1].SizeLimitedFeatures)
	assert.Equal(t, 1, *got.Layers[1].SizeLimitedTiles)

	require.NotNil(t, got.RenderedFeatures)
	assert.Equal(t, 15, *got.RenderedFeatures)
	require.NotNil(t, got.SizeLimitedFeatures)
	assert.Equal(t, 2, *got.SizeLimitedFeatures)

	assert.Equal(t, 2, h.file.existsChecks, "both entry points are checked before reuse")
	assert.Zero(t, h.worker.calls)
}

func TestIntermediateDataView_Render_RerendersWhenOneLayerIsGone(t *testing.T) {
	t.Parallel()

	h := newViewHarness(t, job.StatusCompleted, true)
	key := h.key(tilesReq())
	h.file.report = twoLayerReport(key, key+"/tilejson.json")
	h.file.missingName = key + "/tilejson.json"
	h.worker.writes = h.file.report

	_, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, 1, h.worker.calls, "the vector tiles are gone, so render again")
}

// A layer's reported entry point would reach a path join where ".." escapes the
// view's directory.
func TestIntermediateDataView_Render_DerivesEveryLayersEntryPoint(t *testing.T) {
	t.Parallel()

	h := newViewHarness(t, job.StatusCompleted, true)
	key := h.key(tilesReq())
	h.file.report = twoLayerReport(key, "../../../../metadata/job-secrets.json")

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	require.Len(t, got.Layers, 2)
	assert.Equal(t,
		"https://api.example/artifacts/"+h.source.ID().String()+"/feature-view/"+testFileID+"/"+key+"/tilejson.json",
		got.Layers[1].EntryPointURL)
	assert.NotContains(t, got.Layers[1].EntryPointURL, "..")
}

func TestIntermediateDataView_Get_ServesASingleEntryPointAsOneLayer(t *testing.T) {
	t.Parallel()

	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(42).Key()
	h.file.report = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":42,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":43,"entryPoint":"` + key + `.glb"}`

	got, err := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, key)
	require.NoError(t, err)
	require.NotNil(t, got)

	require.Len(t, got.Layers, 1)
	assert.Equal(t, featureview.FormatGLB, got.Layers[0].Format)
	assert.True(t, strings.HasSuffix(got.Layers[0].EntryPointURL, "/"+key+".glb"))
	assert.Equal(t, 1, got.Layers[0].RenderedFeatures)
}

func TestIntermediateDataView_Render_RefusedWhenThePlansRendersAreAllInProgress(t *testing.T) {
	t.Parallel()

	h := newViewHarness(t, job.StatusCompleted, true)
	usage := &usageCounter{counts: map[string]int64{renderSlotKeyPrefix + h.source.Workspace().String(): 2}}
	h.uc.planLimits = planLimits{policyChecker: &planChecker{}, usage: usage, now: time.Now}

	_, err := h.render(viewTestContext(), gltfReq(42))

	assert.ErrorIs(t, err, interfaces.ErrConcurrentRenderLimit)
	assert.Zero(t, h.worker.calls)
	assert.Equal(t, int64(2), usage.counts[renderSlotKeyPrefix+h.source.Workspace().String()], "the refused render takes no slot")
}

func TestIntermediateDataView_Render_GivesItsSlotBack(t *testing.T) {
	t.Parallel()

	h := newViewHarness(t, job.StatusCompleted, true)
	h.worker.writes = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":42,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":43,"entryPoint":"` + gltfReq(42).Key() + `.glb"}`
	usage := &usageCounter{}
	h.uc.planLimits = planLimits{policyChecker: &planChecker{}, usage: usage, now: time.Now}

	_, err := h.render(viewTestContext(), gltfReq(42))

	require.NoError(t, err)
	assert.Equal(t, 1, h.worker.calls)
	assert.Zero(t, usage.counts[renderSlotKeyPrefix+h.source.Workspace().String()])
}

func TestIntermediateDataView_Render_RefusedRetryKeepsTheFailedReport(t *testing.T) {
	t.Parallel()

	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.report = `{"version":1,"status":"failed","shape":"gltf","row":9,
	  "selectedFeatures":1,"renderedFeatures":0,"scanned":10,"error":"storage blip"}`
	key := renderSlotKeyPrefix + h.source.Workspace().String()
	usage := &usageCounter{counts: map[string]int64{key: 2}}
	h.uc.planLimits = planLimits{policyChecker: &planChecker{}, usage: usage, now: time.Now}

	_, err := h.render(viewTestContext(), gltfReq(9))

	assert.ErrorIs(t, err, interfaces.ErrConcurrentRenderLimit)
	assert.Zero(t, h.file.reportDeletes, "a refused retry leaves the failed report in place")
	assert.NotEmpty(t, h.file.report)
}

func TestIntermediateDataView_Render_GivesItsSlotBackWhenClearingFails(t *testing.T) {
	t.Parallel()

	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.report = `{"version":1,"status":"failed","shape":"gltf","row":9,
	  "selectedFeatures":1,"renderedFeatures":0,"scanned":10,"error":"storage blip"}`
	h.file.deleteErr = errors.New("gcs: down")
	usage := &usageCounter{}
	h.uc.planLimits = planLimits{policyChecker: &planChecker{}, usage: usage, now: time.Now}

	_, err := h.render(viewTestContext(), gltfReq(9))

	assert.Error(t, err)
	assert.Zero(t, h.worker.calls)
	assert.Zero(t, usage.counts[renderSlotKeyPrefix+h.source.Workspace().String()])
}
