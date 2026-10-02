package interactor

import (
	"context"
	"errors"
	"fmt"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
)

// renderTimeout bounds how long a request waits for the renderer. A view is
// normally seconds of work, so this is a ceiling, not an expected wait.
const renderTimeout = 2 * time.Minute

// IntermediateDataView renders the intermediate data a finished run left on one
// output port into a form a viewer can open. The render is awaited in the
// request and is not tracked as a Job.
type IntermediateDataView struct {
	jobRepo           repo.Job
	file              gateway.File
	cloudRunWorker    gateway.CloudRunWorker
	permissionChecker gateway.PermissionChecker
}

func NewIntermediateDataView(
	r *repo.Container,
	gr *gateway.Container,
	permissionChecker gateway.PermissionChecker,
) interfaces.IntermediateDataView {
	return &IntermediateDataView{
		jobRepo:           r.Job,
		file:              gr.File,
		cloudRunWorker:    gr.CloudRunWorker,
		permissionChecker: permissionChecker,
	}
}

// checkPermission guards views with the edge resource, which nothing else
// checks yet. Its rule matches the job's — any action for a workspace writer,
// maintainer or owner — so whoever can see a run's jobs can render and read
// its views, and a reader can do neither. The intermediate data itself is not
// behind this check: the table is fetched through the /artifacts route.
func (i *IntermediateDataView) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceEdge, action, workspaceID...)
}

func (i *IntermediateDataView) Render(
	ctx context.Context,
	p interfaces.RenderIntermediateDataViewParam,
) (*interfaces.IntermediateDataViewResult, error) {
	if _, err := i.authorizedSourceJob(ctx, p.JobID); err != nil {
		return nil, err
	}

	if err := featureview.ValidateFileID(p.FileID); err != nil {
		return nil, err
	}
	if err := p.Request.Validate(); err != nil {
		return nil, err
	}
	p.Request.ServedFrom = i.file.GetFeatureViewURL(p.JobID.String(), p.FileID, "")
	key := p.Request.Key()

	// A rendered view is reused rather than rebuilt: it is a pure function of
	// the request, and the key already encodes every input. This is also what
	// makes a timed-out render recoverable, and what keeps clicking around a
	// table from re-rendering anything.
	existing, err := i.readReport(ctx, p.JobID, p.FileID, key)
	if err != nil {
		return nil, err
	}
	if existing != nil && existing.Status != featureview.StatusFailed {
		reusable, err := i.viewStillPresent(ctx, p.JobID, p.FileID, key, existing)
		if err != nil {
			return nil, err
		}
		if reusable {
			return i.result(ctx, p.JobID, p.FileID, key, p.Request.Shape, existing), nil
		}
		// The report outlived its view. Fall through and render it again.
	}
	// A cached FAILURE is not reused. Empty and unsupported-geometry are
	// properties of the data and will not change; a fault might have been a
	// storage blip, so the caller gets to retry by asking again.

	if i.cloudRunWorker == nil {
		return nil, interfaces.ErrViewsUnavailable
	}

	inputURI, found, err := i.file.ResolveIntermediateDataURI(ctx, p.JobID.String(), p.FileID)
	if err != nil {
		return nil, rerror.ErrInternalByWithContextAndLabel(ctx, "failed to resolve intermediate data", err)
	}
	if !found {
		return nil, fmt.Errorf("%w: %s on job %s", interfaces.ErrIntermediateDataNotFound, p.FileID, p.JobID)
	}

	// The outcome is read back from the report after the call, so a report left
	// by an earlier render has to go first. A render that dies before writing
	// one — a timeout, a reclaimed instance — would otherwise be answered with
	// the old outcome: a READY whose view is gone, or a stale failure in place
	// of the real one.
	if existing != nil {
		if err := i.file.DeleteFeatureViewReport(ctx, p.JobID.String(), p.FileID, key); err != nil {
			return nil, rerror.ErrInternalByWithContextAndLabel(ctx, "failed to clear the previous view report", err)
		}
	}

	return i.render(ctx, p, key, inputURI)
}

func (i *IntermediateDataView) render(
	ctx context.Context,
	p interfaces.RenderIntermediateDataViewParam,
	key, inputURI string,
) (*interfaces.IntermediateDataViewResult, error) {
	renderCtx, cancel := context.WithTimeout(ctx, renderTimeout)
	defer cancel()

	// Sent for every tiles view, since only the engine knows whether it will
	// be vector tiles; a 3D tileset ignores it.
	var tilesURL *string
	if p.Request.Shape == featureview.ShapeTiles && p.Request.ServedFrom != "" {
		u := featureview.VectorTilesURL(i.file.GetFeatureViewURL(p.JobID.String(), p.FileID, key))
		tilesURL = &u
	}

	status, renderErr := i.cloudRunWorker.RenderView(renderCtx, gateway.RenderViewParam{
		InputURI:  inputURI,
		OutputURI: i.file.GetFeatureViewUploadURI(p.JobID.String(), p.FileID),
		ReportURI: i.file.GetFeatureViewReportUploadURI(p.JobID.String(), p.FileID, key),
		Name:      key,
		Shape:     p.Request.Shape,
		Row:       p.Request.Selection.Row,
		Filter:    p.Request.Selection.Filter,
		TilesURL:  tilesURL,
		Options:   p.Request.Options,
	})

	// Read the report before judging the call. A render that drew nothing exits
	// successfully and says why in the report, so the report is more
	// informative than the status whenever it exists. Any report found here is
	// this render's: Render cleared the previous one before dispatching.
	report, err := i.readReport(ctx, p.JobID, p.FileID, key)
	if err != nil {
		return nil, err
	}
	if report != nil {
		// The report is what the caller sees, but a call that also failed is
		// still worth a trace in the log.
		if renderErr != nil {
			log.Warnfc(ctx, "intermediateDataView: render of %s/%s/%s reported %s and failed: %v",
				p.JobID, p.FileID, key, report.Status, renderErr)
		}
		return i.result(ctx, p.JobID, p.FileID, key, p.Request.Shape, report), nil
	}

	// No report and no view: the render did not get far enough to record an
	// outcome, so there is nothing to show the user but the failure itself.
	//
	// A timeout is said plainly; any other failure is the infrastructure's, and
	// its detail — worker stderr, storage paths — is logged, not returned.
	if errors.Is(renderErr, context.DeadlineExceeded) {
		log.Warnfc(ctx, "intermediateDataView: render of %s/%s/%s timed out: %v", p.JobID, p.FileID, key, renderErr)
		return nil, interfaces.ErrRenderTimedOut
	}
	if renderErr != nil {
		return nil, rerror.ErrInternalByWithContextAndLabel(ctx, "failed to render the view", renderErr)
	}
	return nil, rerror.ErrInternalByWithContextAndLabel(ctx, "failed to render the view",
		fmt.Errorf("the renderer reported %s but wrote no report for %s", status, key))
}

func (i *IntermediateDataView) Get(
	ctx context.Context,
	jobID id.JobID,
	fileID, key string,
) (*interfaces.IntermediateDataViewResult, error) {
	if _, err := i.authorizedSourceJob(ctx, jobID); err != nil {
		return nil, err
	}
	if err := featureview.ValidateFileID(fileID); err != nil {
		return nil, err
	}
	// The key is client-supplied here and becomes a storage path segment, so it
	// is checked before any read rather than only for emptiness. A malformed
	// key is an error rather than "no view", the same as a malformed job id.
	if err := featureview.ValidateViewKey(key); err != nil {
		return nil, err
	}

	report, err := i.readReport(ctx, jobID, fileID, key)
	if err != nil {
		return nil, err
	}
	if report == nil {
		// No report means no render has finished: an expected state, not a
		// failure, so it is answered with no view rather than an error.
		return nil, nil
	}

	// A report outlives the view it describes, so a ready one whose entry point
	// has been swept would otherwise be answered READY with a URL that 404s.
	// Get does not render, so the honest answer is that there is no view.
	present, err := i.viewStillPresent(ctx, jobID, fileID, key, report)
	if err != nil {
		return nil, err
	}
	if !present {
		return nil, nil
	}

	// Get has no request to fall back on, so a report that recorded no shape —
	// an unreadable one — is described by the key, which names the shape the
	// view was rendered for.
	keyShape, _ := featureview.ShapeFromKey(key)
	return i.result(ctx, jobID, fileID, key, keyShape, report), nil
}

func (i *IntermediateDataView) authorizedSourceJob(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	source, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}
	if source == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionRead, source.Workspace()); err != nil {
		return nil, err
	}

	// The feature writer appends as the run proceeds, so a file belonging to an
	// unfinished job may be truncated mid-feature. Rendering it would produce a
	// view that disagrees with the table for no visible reason.
	switch source.Status() {
	case job.StatusCompleted, job.StatusFailed, job.StatusCancelled:
	default:
		return nil, fmt.Errorf("%w: %s is %s", interfaces.ErrJobNotFinished, jobID, source.Status())
	}

	return source, nil
}

// readReport returns a view's recorded outcome, or nil when none exists.
//
// A report that cannot be parsed is reported as a failed view rather than an
// error: the render did happen, the server simply cannot read what it wrote,
// and answering "not rendered" would send the caller into a render loop.
func (i *IntermediateDataView) readReport(
	ctx context.Context,
	jobID id.JobID,
	fileID, key string,
) (*featureview.Report, error) {
	body, err := i.file.ReadFeatureViewReport(ctx, jobID.String(), fileID, key)
	if errors.Is(err, rerror.ErrNotFound) {
		return nil, nil
	}
	// A storage backend with no feature-view support (the local filesystem)
	// cannot hold a view at all, which is the same answer as having no worker.
	if errors.Is(err, gateway.ErrUnsupportedOperation) {
		return nil, interfaces.ErrViewsUnavailable
	}
	if err != nil {
		return nil, fmt.Errorf("failed to read view report: %w", err)
	}
	defer func() {
		if err := body.Close(); err != nil {
			log.Warnfc(ctx, "intermediateDataView: closing report for %s/%s: %v", jobID, key, err)
		}
	}()

	report, err := featureview.ParseReport(body)
	if err != nil {
		log.Errorfc(ctx, "intermediateDataView: unreadable report for %s/%s/%s: %v", jobID, fileID, key, err)
		return featureview.UnreadableReport("the render finished but its report could not be read"), nil
	}

	return report, nil
}

// viewStillPresent reports whether a non-failed cached report can be served as
// it stands.
//
// A report and the view it describes are separate objects with separate
// lifetimes: a retention rule can remove the view and leave the report behind.
// Because a non-failed report is otherwise reused forever, that would pin the
// port to an entry point that 404s with no way to rebuild it. Only a ready
// report has anything to check — empty and unsupported-geometry describe a view
// that was never written.
func (i *IntermediateDataView) viewStillPresent(
	ctx context.Context,
	jobID id.JobID,
	fileID, key string,
	report *featureview.Report,
) (bool, error) {
	if report.Status != featureview.StatusReady {
		return true, nil
	}

	name, err := entryPointName(key, report)
	if err != nil {
		// A ready report the server cannot resolve to a file is not a cache
		// hit; rendering again is the only way it improves.
		log.Warnfc(ctx, "intermediateDataView: unusable cached report for %s/%s/%s: %v", jobID, fileID, key, err)
		return false, nil
	}

	exists, err := i.file.CheckFeatureViewFileExists(ctx, jobID.String(), fileID, name)
	if err != nil {
		return false, rerror.ErrInternalByWithContextAndLabel(ctx, "failed to check the rendered view", err)
	}
	if !exists {
		log.Infofc(ctx, "intermediateDataView: report for %s/%s/%s outlived its view; re-rendering", jobID, fileID, key)
	}
	return exists, nil
}

// entryPointName is the view's entry point as the server lays it out, derived
// from the key and the reported format rather than read out of the report.
//
// The server chose the layout — it passed the key to the renderer as --name —
// so it can name the file itself. The report's own entryPoint is the renderer's
// word for the same thing, and it would otherwise reach GetFeatureViewURL as a
// path segment, where path.Join resolves ".." and a wrong or hostile report
// could address a file outside the view's directory.
func entryPointName(key string, report *featureview.Report) (string, error) {
	if report.Format == nil {
		return "", errors.New("a ready report named no format")
	}
	return featureview.EntryPointName(key, *report.Format)
}

// result maps a report onto the usecase result, resolving the view's entry
// point to a URL a browser can open.
//
// fallbackShape covers a report that recorded no shape; it is what the caller
// asked for, which is the same thing the renderer was given.
func (i *IntermediateDataView) result(
	ctx context.Context,
	jobID id.JobID,
	fileID, key string,
	fallbackShape featureview.Shape,
	report *featureview.Report,
) *interfaces.IntermediateDataViewResult {
	shape := report.Shape
	if shape == "" {
		shape = fallbackShape
	}

	out := &interfaces.IntermediateDataViewResult{
		Key:    key,
		JobID:  jobID,
		FileID: fileID,
		Shape:  shape,
		Status: report.Status,
		Format: report.Format,
		Error:  report.Error,
	}

	// The counts explain a view whether or not it drew anything: a partial drop
	// is intended behaviour, so "812 of 900" is not a warning, and "0 of 900"
	// is the whole reason an empty view is empty. A report the server could not
	// read measured nothing, so it answers null rather than a false "0 of 0".
	if report.CountsKnown() {
		selected, rendered := report.SelectedFeatures, report.RenderedFeatures
		out.SelectedFeatures = &selected
		out.RenderedFeatures = &rendered
	}

	if report.Status == featureview.StatusReady {
		name, err := entryPointName(key, report)
		if err != nil {
			log.Errorfc(ctx, "intermediateDataView: unusable report for %s/%s/%s: %v", jobID, fileID, key, err)
			message := "the render finished but its report could not be used"
			out.Status = featureview.StatusFailed
			out.Format = nil
			out.Error = &message
			return out
		}
		// Drift between the two is worth seeing, but the server's layout wins.
		if report.EntryPoint != "" && report.EntryPoint != name {
			log.Warnfc(ctx, "intermediateDataView: report for %s/%s/%s names entry point %q, serving %q",
				jobID, fileID, key, report.EntryPoint, name)
		}
		out.EntryPointURL = i.file.GetFeatureViewURL(jobID.String(), fileID, name)
	}

	return out
}
