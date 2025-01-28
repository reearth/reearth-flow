package fs

import (
	"context"
	"errors"
	"io"
	"net/url"
	"os"
	"path"
	"path/filepath"

	"github.com/kennygrant/sanitize"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
	"github.com/spf13/afero"
)

type fileRepo struct {
	fs              afero.Fs
	assetUrlBase    *url.URL
	workflowUrlBase *url.URL
}

func (f *fileRepo) CheckJobLogExists(context.Context, string) (bool, error) {
	panic("unimplemented")
}

func (f *fileRepo) GetJobLogURL(string) string {
	panic("unimplemented")
}

func (f *fileRepo) ListJobArtifacts(context.Context, string) ([]string, error) {
	panic("unimplemented")
}

func (f *fileRepo) ReadArtifact(context.Context, string) (io.ReadCloser, error) {
	panic("unimplemented")
}

func (f *fileRepo) ReadMetadata(context.Context, string) (io.ReadCloser, error) {
	panic("unimplemented")
}

func (f *fileRepo) RemoveMetadata(context.Context, *url.URL) error {
	panic("unimplemented")
}

func (f *fileRepo) UploadMetadata(context.Context, string, []string) (*url.URL, error) {
	panic("unimplemented")
}

func NewFile(fs afero.Fs, assetUrlBase string, workflowUrlBase string) (gateway.File, error) {
	var err error
	aurlb, err := url.Parse(assetUrlBase)
	if err != nil {
		return nil, errors.New("invalid base URL")
	}
	wurlb, err := url.Parse(workflowUrlBase)
	if err != nil {
		return nil, errors.New("invalid base URL")
	}

	return &fileRepo{
		fs:              fs,
		assetUrlBase:    aurlb,
		workflowUrlBase: wurlb,
	}, nil
}

func (f *fileRepo) ReadAsset(ctx context.Context, filename string) (io.ReadCloser, error) {
	return f.read(ctx, filepath.Join(assetDir, sanitize.Path(filename)))
}

func (f *fileRepo) UploadAsset(ctx context.Context, file *file.File) (*url.URL, int64, error) {
	filename := sanitize.Path(newAssetID() + filepath.Ext(file.Path))
	size, err := f.upload(ctx, filepath.Join(assetDir, filename), file.Content)
	if err != nil {
		return nil, 0, err
	}
	return getFileURL(f.assetUrlBase, filename), size, nil
}

func (f *fileRepo) RemoveAsset(ctx context.Context, u *url.URL) error {
	if u == nil {
		return nil
	}
	p := sanitize.Path(u.Path)
	if p == "" || !f.validateURL(u, f.assetUrlBase) {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, filepath.Join(assetDir, filepath.Base(p)))
}

func (f *fileRepo) ReadWorkflow(ctx context.Context, filename string) (io.ReadCloser, error) {
	return f.read(ctx, filepath.Join(workflowsDir, sanitize.Path(filename)))
}

func (f *fileRepo) UploadWorkflow(ctx context.Context, file *file.File) (*url.URL, error) {
	filename := sanitize.Path(newWorkflowID() + filepath.Ext(file.Path))
	_, err := f.upload(ctx, filepath.Join(workflowsDir, filename), file.Content)
	if err != nil {
		return nil, err
	}
	return getFileURL(f.workflowUrlBase, filename), nil
}

func (f *fileRepo) RemoveWorkflow(ctx context.Context, u *url.URL) error {
	if u == nil {
		return nil
	}
	p := sanitize.Path(u.Path)
	if p == "" || !f.validateURL(u, f.workflowUrlBase) {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, filepath.Join(workflowsDir, filepath.Base(p)))
}

// helpers

func (f *fileRepo) read(ctx context.Context, filename string) (io.ReadCloser, error) {
	if filename == "" {
		return nil, rerror.ErrNotFound
	}

	file, err := f.fs.Open(filename)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, rerror.ErrNotFound
		}
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return file, nil
}

func (f *fileRepo) upload(ctx context.Context, filename string, content io.Reader) (int64, error) {
	if filename == "" {
		return 0, gateway.ErrFailedToUploadFile
	}

	if fnd := filepath.Dir(filename); fnd != "" {
		if err := f.fs.MkdirAll(fnd, 0755); err != nil {
			return 0, rerror.ErrInternalByWithContext(ctx, err)
		}
	}

	dest, err := f.fs.Create(filename)
	if err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, err)
	}
	defer func() {
		_ = dest.Close()
	}()

	size, err := io.Copy(dest, content)
	if err != nil {
		return 0, gateway.ErrFailedToUploadFile
	}

	return size, nil
}

func (f *fileRepo) delete(ctx context.Context, filename string) error {
	if filename == "" {
		return gateway.ErrFailedToUploadFile
	}

	if err := f.fs.RemoveAll(filename); err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	return nil
}

func getFileURL(base *url.URL, filename string) *url.URL {
	if base == nil {
		return nil
	}

	// https://github.com/golang/go/issues/38351
	b := *base
	b.Path = path.Join(b.Path, filename)
	return &b
}

func newAssetID() string {
	return id.NewAssetID().String()
}

func newWorkflowID() string {
	return id.NewWorkflowID().String()
}

func (f *fileRepo) validateURL(u *url.URL, base *url.URL) bool {
	if u == nil || base == nil {
		return false
	}
	return u.Scheme == base.Scheme &&
		u.Host == base.Host &&
		path.Dir(u.Path) == base.Path
}
