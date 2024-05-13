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
	fs      afero.Fs
	urlBase *url.URL
}

func NewFile(fs afero.Fs, urlBase string) (gateway.File, error) {
	var b *url.URL
	var err error
	b, err = url.Parse(urlBase)
	if err != nil {
		return nil, errors.New("invalid base URL")
	}

	return &fileRepo{
		fs:      fs,
		urlBase: b,
	}, nil
}

// asset

func (f *fileRepo) ReadAsset(ctx context.Context, filename string) (io.ReadCloser, error) {
	return f.read(ctx, filepath.Join(assetDir, sanitize.Path(filename)))
}

func (f *fileRepo) UploadAsset(ctx context.Context, file *file.File) (*url.URL, int64, error) {
	filename := sanitize.Path(newAssetID() + filepath.Ext(file.Path))
	size, err := f.upload(ctx, filepath.Join(assetDir, filename), file.Content)
	if err != nil {
		return nil, 0, err
	}
	return getAssetFileURL(f.urlBase, filename), size, nil
}

func (f *fileRepo) RemoveAsset(ctx context.Context, u *url.URL) error {
	if u == nil {
		return nil
	}
	p := sanitize.Path(u.Path)
	if p == "" || f.urlBase == nil || u.Scheme != f.urlBase.Scheme || u.Host != f.urlBase.Host || path.Dir(p) != f.urlBase.Path {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, filepath.Join(assetDir, filepath.Base(p)))
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

func getAssetFileURL(base *url.URL, filename string) *url.URL {
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
