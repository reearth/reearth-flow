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

// renderTimeout bounds how long a request will wait for the renderer.
//
// A view is normally seconds of work, so this is a ceiling for a pathologically
// large port rather than an expected wait. Exceeding it is recoverable without
// re-rendering: the worker is not cancelled by the client going away, so it
// finishes and writes its report, and asking again finds it in place.
const renderTimeout = 2 * time.Minute

// IntermediateDataView renders the intermediate data a finished run left on one
// output port into a form a viewer can open.
//
// The render is called from the request path and waited on. Nothing tracks it:
// a view is a seconds-long artifact transform, and wrapping it in a Job would
// mean a row in the workspace job list, a goroutine polling the database every
// five seconds, and a Pub/Sub round trip — to discover something the renderer
// already knew, all to draw one building.
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

// checkPermission guards views with the same resource that guards the
// intermediate data itself, so anyone who can read a port's table can render a
// view of it and nobody else can.
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
		return nil, fmt.Errorf("failed to resolve intermediate data: %w", err)
	}
	if !found {
		return nil, fmt.Errorf("%w: %s on job %s", interfaces.ErrIntermediateDataNotFound, p.FileID, p.JobID)
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

	status, renderErr := i.cloudRunWorker.RenderView(renderCtx, gateway.RenderViewParam{
		InputURI:  inputURI,
		OutputURI: i.file.GetFeatureViewUploadURI(p.JobID.String(), p.FileID),
		ReportURI: i.file.GetFeatureViewReportUploadURI(p.JobID.String(), p.FileID, key),
		Name:      key,
		Shape:     p.Request.Shape,
		Row:       p.Request.Selection.Row,
		Filter:    p.Request.Selection.Filter,
		Options:   p.Request.Options,
	})

	// Read the report before judging the call. A render that drew nothing exits
	// successfully and says why in the report, so the report is more
	// informative than the status whenever it exists.
	report, err := i.readReport(ctx, p.JobID, p.FileID, key)
	if err != nil {
		return nil, err
	}
	if report != nil {
		return i.result(ctx, p.JobID, p.FileID, key, p.Request.Shape, report), nil
	}

	// No report and no view: the render did not get far enough to record an
	// outcome, so there is nothing to show the user but the failure itself.
	if renderErr != nil {
		return nil, fmt.Errorf("failed to render the view: %w", renderErr)
	}
	return nil, fmt.Errorf("the renderer reported %s but wrote no report for %s", status, key)
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
	if key == "" {
		return nil, rerror.ErrNotFound
	}

	report, err := i.readReport(ctx, jobID, fileID, key)
	if err != nil {
		return nil, err
	}
	if report == nil {
		// No report means no render has finished. Whether one is in flight is
		// the render job's business, and the client already holds its id.
		return nil, rerror.ErrNotFound
	}

	return i.result(ctx, jobID, fileID, key, report.Shape, report), nil
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
		message := "the render finished but its report could not be read"
		return &featureview.Report{
			Version: featureview.ReportVersion,
			Status:  featureview.StatusFailed,
			Error:   &message,
		}, nil
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
		return false, fmt.Errorf("failed to check the rendered view: %w", err)
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
	// is the whole reason an empty view is empty.
	selected, rendered := report.SelectedFeatures, report.RenderedFeatures
	out.SelectedFeatures = &selected
	out.RenderedFeatures = &rendered

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
