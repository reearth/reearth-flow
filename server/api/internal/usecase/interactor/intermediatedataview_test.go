package interactor

import (
	"context"
	"errors"
	"io"
	"strings"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/rbac"
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

// viewFakeFile serves one report and records what was asked of it.
type viewFakeFile struct {
	mockCheckStatusFile

	report       string
	reportErr    error
	inputMissing bool
	// viewMissing makes the view absent while its report stays in place, which
	// is what a retention rule sweeping the rendered files looks like.
	viewMissing bool

	reportReads    int
	resolveCalls   int
	lastReportKey  string
	existsChecks   int
	lastExistsName string
}

func (f *viewFakeFile) CheckFeatureViewFileExists(_ context.Context, _, _, name string) (bool, error) {
	f.existsChecks++
	f.lastExistsName = name
	return !f.viewMissing, nil
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

// viewFakeWorker stands in for the engine worker: it records the call and
// writes whatever report the test wants the render to have produced.
type viewFakeWorker struct {
	calls     int
	lastParam gateway.RenderViewParam

	// writes is the report the render leaves behind, or "" for a render that
	// recorded nothing.
	writes string
	status gateway.JobStatus
	err    error

	file *viewFakeFile
}

func (w *viewFakeWorker) RenderView(_ context.Context, p gateway.RenderViewParam) (gateway.JobStatus, error) {
	w.calls++
	w.lastParam = p
	w.file.report = w.writes
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

// A rendered view is reused rather than rebuilt: it is a pure function of the
// request, so the second look costs one report read and no job.
func TestIntermediateDataView_Render_ReusesARenderedView(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(42).Key()
	h.file.report = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":42,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":43,"entryPoint":"` + key + `.glb"}`

	got, err := h.render(viewTestContext(), gltfReq(42))
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusReady, got.Status)
	assert.Equal(t, key, got.Key)
	require.NotNil(t, got.Format)
	assert.Equal(t, featureview.FormatGLB, *got.Format)
	assert.Equal(t,
		"https://api.example/artifacts/"+h.source.ID().String()+"/feature-view/"+testFileID+"/"+key+".glb",
		got.EntryPointURL)
	require.NotNil(t, got.RenderedFeatures)
	assert.Equal(t, 1, *got.RenderedFeatures)

	assert.Zero(t, h.worker.calls, "an existing view is not re-rendered")
	assert.Zero(t, h.file.resolveCalls, "the input is not even resolved on a cache hit")
	assert.Equal(t, key, h.file.lastReportKey)
}

// A report and the view it describes have separate lifetimes. When a retention
// rule takes the view and leaves the report, the report is not a cache hit:
// reusing it would pin the port to an entry point that 404s, and because a
// non-failed report is otherwise never re-rendered, it would do so forever.
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

// The counterpart: the view is there, so the report is served without a render.
func TestIntermediateDataView_Render_ChecksTheViewBeforeReusingIt(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := tilesReq().Key()
	h.file.report = `{"version":1,"status":"ready","shape":"tiles","format":"vector_tiles",
	  "selectedFeatures":3,"renderedFeatures":3,"scanned":3,"entryPoint":"` + key + `/tilejson.json"}`

	_, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, 1, h.file.existsChecks)
	assert.Equal(t, key+"/tilejson.json", h.file.lastExistsName)
	assert.Zero(t, h.worker.calls)
}

// A view that drew nothing wrote no files, so there is nothing to look for and
// the report stands on its own.
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

// The entry point served is the one the server's own layout implies, derived
// from the key and the reported format. The report's entryPoint is the
// renderer's word for the same file and reaches a path join, where ".." would
// escape the view's directory, so it is cross-checked and not followed.
func TestIntermediateDataView_Render_DerivesTheEntryPointRatherThanTrustingTheReport(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(7).Key()
	h.file.report = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":7,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":8,
	  "entryPoint":"../../../../metadata/job-secrets.json"}`

	got, err := h.render(viewTestContext(), gltfReq(7))
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusReady, got.Status)
	assert.Equal(t,
		"https://api.example/artifacts/"+h.source.ID().String()+"/feature-view/"+testFileID+"/"+key+".glb",
		got.EntryPointURL,
		"the served URL follows the key and format, not the report")
	assert.NotContains(t, got.EntryPointURL, "..")
}

// A ready report that names no format cannot be resolved to a file at all, so
// it is reported as failed rather than served as a view with no entry point.
func TestIntermediateDataView_Get_FailsAReadyReportWithNoFormat(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := tilesReq().Key()
	// Bypasses ParseReport's own guard by going through Get with a report the
	// server built rather than parsed.
	got := h.uc.result(viewTestContext(), h.source.ID(), testFileID, key, featureview.ShapeTiles,
		&featureview.Report{Version: featureview.ReportVersion, Status: featureview.StatusReady})

	assert.Equal(t, featureview.StatusFailed, got.Status)
	assert.Empty(t, got.EntryPointURL)
	assert.Nil(t, got.Format)
	require.NotNil(t, got.Error)
}

// Geometry naming no CRS, and geometry the writer cannot draw, are dropped on
// purpose: the feature keeps its row in the table and only its geometry is
// missing from the view. So a view that rendered fewer features than it
// selected is READY, not degraded — and both counts come back so a client can
// say so.
func TestIntermediateDataView_Render_ReportsAPartialDropAsReady(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := tilesReq().Key()
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
	assert.NotEmpty(t, got.EntryPointURL)
}

// "0 of 900 selected" is the explanation for an empty view, so the counts must
// survive exactly when there is nothing to show.
func TestIntermediateDataView_Render_KeepsTheCountsOnAnEmptyView(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.file.report = `{"version":1,"status":"empty","shape":"tiles",
	  "selectedFeatures":900,"renderedFeatures":0,"scanned":900,
	  "error":"none of the 900 selected features carried geometry the view draws"}`

	got, err := h.render(viewTestContext(), tilesReq())
	require.NoError(t, err)

	assert.Equal(t, featureview.StatusEmpty, got.Status)
	require.NotNil(t, got.SelectedFeatures)
	require.NotNil(t, got.RenderedFeatures)
	assert.Equal(t, 900, *got.SelectedFeatures)
	assert.Equal(t, 0, *got.RenderedFeatures)
	assert.Empty(t, got.EntryPointURL)
}

func TestIntermediateDataView_Render_RendersAndReturnsTheFinishedView(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	key := gltfReq(42).Key()
	h.worker.writes = `{"version":1,"status":"ready","shape":"gltf","format":"glb","row":42,
	  "selectedFeatures":1,"renderedFeatures":1,"scanned":43,"entryPoint":"` + key + `.glb"}`

	got, err := h.render(viewTestContext(), gltfReq(42))
	require.NoError(t, err)

	// One round trip: the caller gets the loadable view, not a handle to poll.
	assert.Equal(t, featureview.StatusReady, got.Status)
	require.NotNil(t, got.Format)
	assert.Equal(t, featureview.FormatGLB, *got.Format)
	assert.Contains(t, got.EntryPointURL, key+".glb")

	require.Equal(t, 1, h.worker.calls)
	p := h.worker.lastParam
	assert.Equal(t, key, p.Name)
	assert.Equal(t, featureview.ShapeGLTF, p.Shape)
	require.NotNil(t, p.Row)
	assert.Equal(t, 42, *p.Row)
	assert.Contains(t, p.InputURI, "/feature-store/"+testFileID)
	assert.Contains(t, p.OutputURI, "/feature-view/"+testFileID)
	assert.Contains(t, p.ReportURI, featureview.ReportName(key))
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

// A render that recorded nothing has nothing to show the user but the failure.
func TestIntermediateDataView_Render_FailsWhenNoReportIsWritten(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)
	h.worker.status = gateway.JobStatusFailed
	h.worker.err = errors.New("worker exit: signal 9")

	_, err := h.render(viewTestContext(), gltfReq(0))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "signal 9")
}

// A worker that claims success but leaves no report is a fault, not an empty
// view — saying "nothing to show" would be inventing an outcome.
func TestIntermediateDataView_Render_FailsOnASilentSuccess(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)

	_, err := h.render(viewTestContext(), gltfReq(0))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "wrote no report")
}

// Views have no second pipeline, so where no worker service is configured they
// are simply not offered.
func TestIntermediateDataView_Render_UnavailableWithoutAWorker(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, false)

	_, err := h.render(viewTestContext(), gltfReq(0))
	assert.ErrorIs(t, err, interfaces.ErrViewsUnavailable)
	assert.Zero(t, h.worker.calls)
}

// Empty and unsupported-geometry are properties of the data and will not change
// on a retry, so they are served from the report. A fault might have been
// transient, so asking again re-renders.
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

// A render did happen; the server simply cannot read what it wrote. Reporting
// "not rendered" would send the caller into a render loop.
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

	// A failed or cancelled run still leaves data for the ports that did run,
	// so those are not refused. Whether the render then succeeds is a separate
	// question, covered elsewhere.
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

// Views are guarded by the same resource that guards the intermediate data
// itself, so anyone who can read a port's table can render a view of it.
func TestIntermediateDataView_Render_ChecksThePortResource(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)

	var resource, action string
	h.uc.permissionChecker = NewMockPermissionChecker(
		func(_ context.Context, r, a string) (bool, error) {
			resource, action = r, a
			return false, nil
		})

	_, err := h.render(viewTestContext(), gltfReq(0))
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
	assert.Equal(t, rbac.ResourceEdge, resource)
	assert.Equal(t, rbac.ActionRead, action)
	assert.Zero(t, h.worker.calls)
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
	require.NotNil(t, got.Format)
	assert.Equal(t, featureview.FormatVectorTiles, *got.Format, "an all-2D edge renders vector tiles")
	assert.Contains(t, got.EntryPointURL, key+"/tilejson.json")
	assert.Zero(t, h.worker.calls, "Get reads an existing view and never renders")
}

func TestIntermediateDataView_Get_NotFoundWhenNoViewExists(t *testing.T) {
	h := newViewHarness(t, job.StatusCompleted, true)

	_, err := h.uc.Get(viewTestContext(), h.source.ID(), testFileID, "tiles-all-deadbeef")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
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
