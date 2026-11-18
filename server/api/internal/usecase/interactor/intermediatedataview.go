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

// renderTimeout bounds how long a request waits for the renderer. It sits above
// the worker's own 100-second limit, so it only runs out when the worker never answers.
const renderTimeout = 2 * time.Minute

// IntermediateDataView renders the intermediate data a finished run left on one
// output port into a form a viewer can open. The render is awaited in the
// request and is not tracked as a Job.
type IntermediateDataView struct {
	jobRepo           repo.Job
	file              gateway.File
	cloudRunWorker    gateway.CloudRunWorker
	permissionChecker gateway.PermissionChecker
	planLimits        planLimits
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
		planLimits:        newPlanLimits(r, gr, nil),
	}
}

// checkPermission is called with rbac.ActionAny because "any" is a literal action
// name in the edge policy, not a wildcard, and the policy defines no other action.
// The intermediate data itself is fetched through the /artifacts route and is not
// behind this check.
func (i *IntermediateDataView) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceEdge, action, workspaceID...)
}

func (i *IntermediateDataView) Render(
	ctx context.Context,
	p interfaces.RenderIntermediateDataViewParam,
) (*interfaces.IntermediateDataViewResult, error) {
	source, err := i.authorizedSourceJob(ctx, p.JobID)
	if err != nil {
		return nil, err
	}

	if err := featureview.ValidateFileID(p.FileID); err != nil {
		return nil, err
	}
	if err := p.Request.Validate(); err != nil {
		return nil, err
	}
	p.Request.ServedFrom = i.file.GetFeatureViewURL(p.JobID.String(), p.FileID, "")
	p.Request.EngineVersion = i.engineVersion(ctx)
	key := p.Request.Key()

	// A view is a pure function of its key, so a rendered one is reused rather than rebuilt.
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
	// A cached failure is not reused, since it may have been transient.

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

	// The slot is taken before the previous report is cleared, so a refused retry
	// leaves that report in place.
	done, err := i.planLimits.startRender(ctx, source.Workspace())
	if err != nil {
		return nil, err
	}
	defer done()
	// The slot is held only so long, so the work it covers ends before then.
	ctx, cancel := context.WithTimeout(ctx, renderWorkTimeout)
	defer cancel()

	// The outcome is read back from the report after the call, so an earlier
	// report must go first. Otherwise a render that dies before writing one would
	// be answered with the stale outcome.
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

	// The report is more informative than the call's status, since a render that
	// drew nothing still exits successfully. Render cleared the previous report,
	// so any found here is this render's.
	report, err := i.readReport(ctx, p.JobID, p.FileID, key)
	if err != nil {
		return nil, err
	}
	if report != nil {
		// A worker redeployed inside the version cache's window renders as the
		// new engine under a key built for the old one. The view is still
		// sound, and the key catches up when the cache does.
		if report.EngineVersion != "" && report.EngineVersion != p.Request.EngineVersion {
			log.Warnfc(ctx, "intermediateDataView: %s/%s/%s was keyed for engine %q but rendered by %q",
				p.JobID, p.FileID, key, p.Request.EngineVersion, report.EngineVersion)
		}
		if renderErr != nil {
			log.Warnfc(ctx, "intermediateDataView: render of %s/%s/%s reported %s and failed: %v",
				p.JobID, p.FileID, key, report.Status, renderErr)
		}
		return i.result(ctx, p.JobID, p.FileID, key, p.Request.Shape, report), nil
	}

	// No report means the render never recorded an outcome. Infrastructure
	// failure detail, such as worker stderr and storage paths, is logged rather than returned.
	if errors.Is(renderErr, gateway.ErrWorkerBusy) {
		log.Warnfc(ctx, "intermediateDataView: render of %s/%s/%s turned away: %v", p.JobID, p.FileID, key, renderErr)
		return nil, interfaces.ErrViewWorkersBusy
	}
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

// engineVersion is the engine the worker runs, for the view key, or "" when it
// cannot be learned. An unknown version leaves the version out of the key instead
// of failing the request.
func (i *IntermediateDataView) engineVersion(ctx context.Context) string {
	if i.cloudRunWorker == nil {
		return ""
	}
	v, err := i.cloudRunWorker.EngineVersion(ctx)
	if err != nil {
		log.Warnfc(ctx, "intermediateDataView: keying views without an engine version: %v", err)
		return ""
	}
	return v
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
	// The key is client-supplied and becomes a storage path segment, so it is
	// validated before any read.
	if err := featureview.ValidateViewKey(key); err != nil {
		return nil, err
	}

	report, err := i.readReport(ctx, jobID, fileID, key)
	if err != nil {
		return nil, err
	}
	if report == nil {
		// No report means no render has finished, which is not an error.
		return nil, nil
	}

	// A report can outlive its swept view, and Get does not render, so that case
	// answers no view.
	present, err := i.viewStillPresent(ctx, jobID, fileID, key, report)
	if err != nil {
		return nil, err
	}
	if !present {
		return nil, nil
	}

	// A report that recorded no shape falls back to the shape named in the key.
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
	if err := i.checkPermission(ctx, rbac.ActionAny, source.Workspace()); err != nil {
		return nil, err
	}

	// The feature writer appends as the run proceeds, so an unfinished job's file
	// may be truncated mid-feature.
	switch source.Status() {
	case job.StatusCompleted, job.StatusFailed, job.StatusCancelled:
	default:
		return nil, fmt.Errorf("%w: %s is %s", interfaces.ErrJobNotFinished, jobID, source.Status())
	}

	return source, nil
}

// readReport returns a view's recorded outcome, or nil when none exists. An
// unparseable report is returned as a failed view, because answering "not
// rendered" would send the caller into a render loop.
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

// viewStillPresent reports whether a non-failed cached report can be served,
// since a retention rule can remove a view and leave its report behind. Only a
// ready report has files to check, and every layer's entry point must exist.
func (i *IntermediateDataView) viewStillPresent(
	ctx context.Context,
	jobID id.JobID,
	fileID, key string,
	report *featureview.Report,
) (bool, error) {
	if report.Status != featureview.StatusReady {
		return true, nil
	}

	names, err := entryPointNames(key, report)
	if err != nil {
		// A ready report that cannot be resolved to files is treated as a cache miss.
		log.Warnfc(ctx, "intermediateDataView: unusable cached report for %s/%s/%s: %v", jobID, fileID, key, err)
		return false, nil
	}

	for _, name := range names {
		exists, err := i.file.CheckFeatureViewFileExists(ctx, jobID.String(), fileID, name)
		if err != nil {
			return false, rerror.ErrInternalByWithContextAndLabel(ctx, "failed to check the rendered view", err)
		}
		if !exists {
			log.Infofc(ctx, "intermediateDataView: report for %s/%s/%s outlived its view; re-rendering", jobID, fileID, key)
			return false, nil
		}
	}
	return true, nil
}

// entryPointNames derives a ready view's entry points from the key and each
// layer's format rather than trusting the report. The report's own names would
// reach GetFeatureViewURL as path segments, where ".." could escape the view's directory.
func entryPointNames(key string, report *featureview.Report) ([]string, error) {
	layers := report.ViewLayers()
	if len(layers) == 0 {
		return nil, errors.New("a ready report named no format")
	}
	names := make([]string, len(layers))
	for n, layer := range layers {
		name, err := featureview.EntryPointName(key, layer.Format)
		if err != nil {
			return nil, err
		}
		names[n] = name
	}
	return names, nil
}

// result maps a report onto the usecase result, resolving each of the view's entry
// points to a URL a browser can open. fallbackShape is used when the report
// recorded no shape.
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
		Error:  report.Error,
	}

	// An unreadable report measured nothing, so its counts stay null rather than "0 of 0".
	if report.CountsKnown() {
		selected, rendered := report.SelectedFeatures, report.RenderedFeatures
		out.SelectedFeatures = &selected
		out.RenderedFeatures = &rendered
	}

	if report.Status == featureview.StatusReady {
		names, err := entryPointNames(key, report)
		if err != nil {
			log.Errorfc(ctx, "intermediateDataView: unusable report for %s/%s/%s: %v", jobID, fileID, key, err)
			message := "the render finished but its report could not be used"
			out.Status = featureview.StatusFailed
			out.Error = &message
			return out
		}
		out.Layers = i.layers(ctx, jobID, fileID, key, report.ViewLayers(), names)
		// Only vector tiles have a size limit, so other views leave these null rather than zero.
		for _, layer := range out.Layers {
			if layer.Format == featureview.FormatVectorTiles {
				out.SizeLimitedFeatures = layer.SizeLimitedFeatures
				out.SizeLimitedTiles = layer.SizeLimitedTiles
			}
		}
	}

	return out
}

// layers resolves a ready view's layers to URLs a browser can open, served
// from names, the entry points the server's own layout gives them.
func (i *IntermediateDataView) layers(
	ctx context.Context,
	jobID id.JobID,
	fileID, key string,
	layers []featureview.ReportLayer,
	names []string,
) []interfaces.IntermediateDataViewLayer {
	out := make([]interfaces.IntermediateDataViewLayer, len(layers))
	for n, layer := range layers {
		// Drift between the two is worth seeing, but the server's layout wins.
		if layer.EntryPoint != "" && layer.EntryPoint != names[n] {
			log.Warnfc(ctx, "intermediateDataView: report for %s/%s/%s names entry point %q, serving %q",
				jobID, fileID, key, layer.EntryPoint, names[n])
		}
		out[n] = interfaces.IntermediateDataViewLayer{
			Format:           layer.Format,
			EntryPointURL:    i.file.GetFeatureViewURL(jobID.String(), fileID, names[n]),
			SelectedFeatures: layer.SelectedFeatures,
			RenderedFeatures: layer.RenderedFeatures,
		}
		if layer.Format == featureview.FormatVectorTiles {
			out[n].SizeLimitedFeatures = layer.SizeLimitedFeatures
			out[n].SizeLimitedTiles = layer.SizeLimitedTiles
		}
	}
	return out
}
