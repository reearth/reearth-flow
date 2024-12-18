package gateway

import (
	"context"
	"errors"
	"io"
	"net/url"

	"github.com/reearth/reearth-flow/api/pkg/file"
)

var (
	ErrInvalidFile            error = errors.New("invalid file")
	ErrFailedToUploadFile     error = errors.New("failed to upload file")
	ErrFileTooLarge           error = errors.New("file too large")
	ErrFailedToRemoveFile     error = errors.New("failed to remove file")
	ErrInvalidWorkflow        error = errors.New("invalid workflow")
	ErrFailedToUploadWorkflow error = errors.New("failed to upload workflow")
	ErrFailedToRemoveWorkflow error = errors.New("failed to remove workflow")
)

type File interface {
	ReadAsset(context.Context, string) (io.ReadCloser, error)
	UploadAsset(context.Context, *file.File) (*url.URL, int64, error)
	RemoveAsset(context.Context, *url.URL) error
	ReadWorkflow(context.Context, string) (io.ReadCloser, error)
	UploadWorkflow(context.Context, *file.File) (*url.URL, error)
	RemoveWorkflow(context.Context, *url.URL) error
	UploadMetadata(context.Context, string, []string) (*url.URL, error)
	RemoveMetadata(context.Context, *url.URL) error
}
