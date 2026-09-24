package gateway

import (
	"context"
	"errors"
	"io"
	"mime"
	"net/url"
	"path"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/file"
)

var (
	ErrInvalidFile                error = errors.New("invalid file")
	ErrFailedToUploadFile         error = errors.New("failed to upload file")
	ErrFileTooLarge               error = errors.New("file too large")
	ErrFailedToRemoveFile         error = errors.New("failed to remove file")
	ErrInvalidWorkflow            error = errors.New("invalid workflow")
	ErrFailedToUploadWorkflow     error = errors.New("failed to upload workflow")
	ErrFailedToRemoveWorkflow     error = errors.New("failed to remove workflow")
	ErrUnsupportedContentEncoding error = errors.New("unsupported content encoding")
	ErrUnsupportedOperation       error = errors.New("unsupported operation")
	// ErrAssetUploadNotConfigured is returned when the deployment has no
	// asset storage backend capable of issuing upload links (local-fs fallback).
	ErrAssetUploadNotConfigured error = errors.New("asset upload is not configured for this deployment")
	// ErrSignedURLFailed is returned when signing an asset upload URL fails.
	// Wrap the underlying cause with %w so it can still be inspected/logged.
	ErrSignedURLFailed error = errors.New("failed to sign asset upload URL")
)

type IssueUploadAssetParam struct {
	ExpiresAt       time.Time
	UUID            string
	Filename        string
	ContentType     string
	ContentEncoding string
	Cursor          string
	Workspace       string
	ContentLength   int64
}

type UploadAssetLink struct {
	URL             string
	ContentType     string
	ContentEncoding string
	Next            string
	ContentLength   int64
}

func (p IssueUploadAssetParam) GetOrGuessContentType() string {
	if p.ContentType != "" {
		return p.ContentType
	}
	return mime.TypeByExtension(path.Ext(p.Filename))
}

type File interface {
	ReadAsset(context.Context, string) (io.ReadCloser, error)
	ReadActions(context.Context, string) (io.ReadCloser, error)
	UploadAsset(context.Context, *file.File) (*url.URL, int64, error)
	DeleteAsset(context.Context, *url.URL) error
	ReadWorkflow(context.Context, string) (io.ReadCloser, error)
	UploadWorkflow(context.Context, *file.File) (*url.URL, error)
	RemoveWorkflow(context.Context, *url.URL) error
	ReadMetadata(context.Context, string) (io.ReadCloser, error)
	UploadMetadata(context.Context, string, []string) (*url.URL, error)
	RemoveMetadata(context.Context, *url.URL) error
	ReadArtifact(context.Context, string) (io.ReadCloser, error)
	ListJobArtifacts(context.Context, string) ([]string, error)
	GetJobLogURL(string) string
	CheckJobLogExists(context.Context, string) (bool, error)
	GetJobWorkerLogURL(string) string
	CheckJobWorkerLogExists(context.Context, string) (bool, error)
	GetJobUserFacingLogURL(string) string
	CheckJobUserFacingLogExists(context.Context, string) (bool, error)
	// GetJobPreviewSchemaURL returns the GCS URL where the preview-schema probe
	// writes its SchemaReport JSON for jobID.
	GetJobPreviewSchemaURL(string) string
	// GetJobPreviewSchemaUploadURI returns the gs://-style write URI the worker
	// writes the SchemaReport to (distinct from the https read URL above).
	GetJobPreviewSchemaUploadURI(string) string
	// CheckJobPreviewSchemaExists reports whether the SchemaReport for jobID exists.
	CheckJobPreviewSchemaExists(context.Context, string) (bool, error)
	GetIntermediateDataURL(context.Context, string, string) string
	CheckIntermediateDataExists(context.Context, string, string) (bool, error)
	// ResolveIntermediateDataURI returns the gs://-style read URI of the
	// intermediate-data file fileID under jobID, and whether it exists. The
	// renderer is handed gs:// rather than the public https URL because the
	// engine's http storage backend is read-only and unauthenticated.
	//
	// It reports which extension the file actually carries: a run writes
	// `.jsonl` or `.jsonl.zst` depending on WORKER_COMPRESS_INTERMEDIATE_DATA,
	// and resolving it here means a missing input fails before a render job is
	// dispatched rather than inside one.
	ResolveIntermediateDataURI(ctx context.Context, jobID, fileID string) (string, bool, error)
	// GetFeatureViewUploadURI returns the gs:// root a view's files are written
	// under, which is the renderer's --output.
	GetFeatureViewUploadURI(jobID, fileID string) string
	// GetFeatureViewReportUploadURI returns the gs:// URI the renderer writes a
	// view's report to.
	GetFeatureViewReportUploadURI(jobID, fileID, key string) string
	// GetFeatureViewURL returns the public https URL of one file inside a view.
	// name is relative to the view's own directory.
	GetFeatureViewURL(jobID, fileID, name string) string
	// ReadFeatureViewReport opens a view's report, and is also how an existing
	// view is detected: rerror.ErrNotFound means it has not been rendered.
	ReadFeatureViewReport(ctx context.Context, jobID, fileID, key string) (io.ReadCloser, error)
	// CheckFeatureViewFileExists reports whether one file inside a view is
	// still present. name is relative to the view's own directory, as
	// EntryPointName builds it.
	//
	// A report and the view it describes are separate objects with separate
	// lifetimes, so a report alone is not proof the view is still loadable.
	CheckFeatureViewFileExists(ctx context.Context, jobID, fileID, name string) (bool, error)
	IssueUploadAssetLink(context.Context, IssueUploadAssetParam) (*UploadAssetLink, error)
	GetPublicAssetURL(string, string) (*url.URL, error)
	UploadedAsset(context.Context, *asset.Upload) (*file.File, error)
	// WriteCancelFlag writes the debug-run cancel marker for jobID.
	WriteCancelFlag(ctx context.Context, jobID string) error
	// CancelFlagURI returns the URI of the cancel marker for jobID (the worker polls it).
	CancelFlagURI(jobID string) string
}
