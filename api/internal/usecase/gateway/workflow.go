package gateway

import (
	"context"
	"errors"
	"io"
	"net/url"

	"github.com/reearth/reearth-flow/api/pkg/file"
)

var (
	ErrInvalidWorkflow        error = errors.New("invalid workflow")
	ErrFailedToUploadWorkflow error = errors.New("failed to upload workflow")
	ErrFailedToRemoveWorkflow error = errors.New("failed to remove workflow")
)

type Workflow interface {
	ReadWorkflow(context.Context, string) (io.ReadCloser, error)
	UploadWorkflow(context.Context, *file.File) (*url.URL, error)
	RemoveWorkflow(context.Context, *url.URL) error
}
