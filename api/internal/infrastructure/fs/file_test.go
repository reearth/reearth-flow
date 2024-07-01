package fs

import (
	"context"
	"io"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearthx/rerror"
	"github.com/spf13/afero"
	"github.com/stretchr/testify/assert"
)

func TestNewFile(t *testing.T) {
	f, err := NewFile(mockFs(), "")
	assert.NoError(t, err)
	assert.NotNil(t, f)
}

func TestFile_ReadAsset(t *testing.T) {
	f, _ := NewFile(mockFs(), "")

	r, err := f.ReadAsset(context.Background(), "xxx.txt")
	assert.NoError(t, err)
	c, err := io.ReadAll(r)
	assert.NoError(t, err)
	assert.Equal(t, "hello", string(c))
	assert.NoError(t, r.Close())

	r, err = f.ReadAsset(context.Background(), "aaa.txt")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
	assert.Nil(t, r)

	r, err = f.ReadAsset(context.Background(), "../published/s.json")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
	assert.Nil(t, r)
}

func TestFile_UploadAsset(t *testing.T) {
	fs := mockFs()
	f, _ := NewFile(fs, "https://example.com/assets")

	u, s, err := f.UploadAsset(context.Background(), &file.File{
		Path:    "aaa.txt",
		Content: io.NopCloser(strings.NewReader("aaa")),
	})
	assert.NoError(t, err)
	assert.Equal(t, int64(3), s)
	assert.Equal(t, "https", u.Scheme)
	assert.Equal(t, "example.com", u.Host)
	assert.True(t, strings.HasPrefix(u.Path, "/assets/"))
	assert.Equal(t, ".txt", filepath.Ext(u.Path))

	uf, _ := fs.Open(filepath.Join("assets", filepath.Base(u.Path)))
	c, _ := io.ReadAll(uf)
	assert.Equal(t, "aaa", string(c))
}

func TestFile_RemoveAsset(t *testing.T) {
	cases := []struct {
		Name    string
		URL     string
		Deleted bool
		Err     error
	}{
		{
			Name:    "deleted",
			URL:     "https://example.com/assets/xxx.txt",
			Deleted: true,
		},
		{
			Name: "not deleted 1",
			URL:  "https://example.com/assets/aaa.txt",
			Err:  nil,
		},
		{
			Name: "not deleted 2",
			URL:  "https://example.com/plugins/xxx.txt",
			Err:  gateway.ErrInvalidFile,
		},
	}

	for _, tc := range cases {
		tc := tc
		t.Run(tc.Name, func(t *testing.T) {
			t.Parallel()

			fs := mockFs()
			f, _ := NewFile(fs, "https://example.com/assets")

			u, _ := url.Parse(tc.URL)
			err := f.RemoveAsset(context.Background(), u)

			if tc.Err == nil {
				assert.NoError(t, err)
			} else {
				assert.Same(t, tc.Err, err)
			}

			_, err = fs.Stat(filepath.Join("assets", "xxx.txt"))
			if tc.Deleted {
				assert.ErrorIs(t, err, os.ErrNotExist)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestGetAssetFileURL(t *testing.T) {
	e, err := url.Parse("http://hoge.com/assets/xxx.yyy")
	assert.NoError(t, err)
	b, err := url.Parse("http://hoge.com/assets")
	assert.NoError(t, err)
	assert.Equal(t, e, getAssetFileURL(b, "xxx.yyy"))
}

func mockFs() afero.Fs {
	files := map[string]string{
		filepath.Join("assets", "xxx.txt"):              "hello",
		filepath.Join("published", "s.json"):            "{}",
	}

	fs := afero.NewMemMapFs()
	for name, content := range files {
		f, _ := fs.Create(name)
		_, _ = f.WriteString(content)
		_ = f.Close()
	}
	return fs
}
