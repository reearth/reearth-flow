package gateway

import (
	"context"
	"errors"
	"io"
	"mime"
	"net/url"
	"path"
	"time"

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
)

type IssueUploadAssetParam struct {
	UUID            string
	Filename        string
	ContentLength   int64
	ContentType     string
	ContentEncoding string
	ExpiresAt       time.Time
	Cursor          string
	Workspace       string
}

type UploadAssetLink struct {
	URL             string
	ContentType     string
	ContentLength   int64
	ContentEncoding string
	Next            string
}

func (p IssueUploadAssetParam) GetOrGuessContentType() string {
	if p.ContentType != "" {
		return p.ContentType
	}
	return mime.TypeByExtension(path.Ext(p.Filename))
}

type File interface {
	ReadAsset(context.Context, string) (io.ReadCloser, error)
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
	GetIntermediateDataURL(context.Context, string, string) string
	CheckIntermediateDataExists(context.Context, string, string) (bool, error)
	IssueUploadAssetLink(context.Context, IssueUploadAssetParam) (*UploadAssetLink, error)
}
