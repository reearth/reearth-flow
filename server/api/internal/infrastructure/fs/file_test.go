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
	f, err := NewFile(mockFs(), "", "")
	assert.NoError(t, err)
	assert.NotNil(t, f)
}

func TestFile_ReadAsset(t *testing.T) {
	f, _ := NewFile(mockFs(), "", "")

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
	f, _ := NewFile(fs, "https://example.com/assets", "https://example.com/workflows")

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

func TestFile_DeleteAsset(t *testing.T) {
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
			f, _ := NewFile(fs, "https://example.com/assets", "https://example.com/workflows")

			u, _ := url.Parse(tc.URL)
			err := f.DeleteAsset(context.Background(), u)

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
	assert.Equal(t, e, getFileURL(b, "xxx.yyy"))
}

func TestFile_ReadWorkflow(t *testing.T) {
	f, _ := NewFile(mockFs(), "", "")

	r, err := f.ReadWorkflow(context.Background(), "xxx.txt")
	assert.NoError(t, err)
	c, err := io.ReadAll(r)
	assert.NoError(t, err)
	assert.Equal(t, "hello", string(c))
	assert.NoError(t, r.Close())

	r, err = f.ReadWorkflow(context.Background(), "aaa.txt")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
	assert.Nil(t, r)
}

func TestFile_UploadWorkflow(t *testing.T) {
	fs := mockFs()
	f, _ := NewFile(fs, "https://example.com/assets", "https://example.com/workflows")

	u, err := f.UploadWorkflow(context.Background(), &file.File{
		Path:    "aaa.txt",
		Content: io.NopCloser(strings.NewReader("aaa")),
	})
	assert.NoError(t, err)
	assert.Equal(t, "https", u.Scheme)
	assert.Equal(t, "example.com", u.Host)
	assert.True(t, strings.HasPrefix(u.Path, "/workflows/"))
	assert.Equal(t, ".txt", filepath.Ext(u.Path))

	uf, _ := fs.Open(filepath.Join("workflows", filepath.Base(u.Path)))
	c, _ := io.ReadAll(uf)
	assert.Equal(t, "aaa", string(c))
}

func TestFile_RemoveWorkflow(t *testing.T) {
	cases := []struct {
		Name    string
		URL     string
		Deleted bool
		Err     error
	}{
		{
			Name:    "deleted",
			URL:     "https://example.com/workflows/xxx.txt",
			Deleted: true,
		},
		{
			Name: "not deleted 1",
			URL:  "https://example.com/workflows/aaa.txt",
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
			f, _ := NewFile(fs, "https://example.com/assets", "https://example.com/workflows")

			u, _ := url.Parse(tc.URL)
			err := f.RemoveWorkflow(context.Background(), u)

			if tc.Err == nil {
				assert.NoError(t, err)
			} else {
				assert.Same(t, tc.Err, err)
			}

			_, err = fs.Stat(filepath.Join("workflows", "xxx.txt"))
			if tc.Deleted {
				assert.ErrorIs(t, err, os.ErrNotExist)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestGetWorkflowFileURL(t *testing.T) {
	e, err := url.Parse("http://hoge.com/workflows/xxx.yyy")
	assert.NoError(t, err)
	b, err := url.Parse("http://hoge.com/workflows")
	assert.NoError(t, err)
	assert.Equal(t, e, getFileURL(b, "xxx.yyy"))
}

func TestFile_GetJobLogURL(t *testing.T) {
	f, _ := NewFile(mockFs(), "", "")

	url := f.GetJobLogURL("job123")
	assert.Equal(t, "file://metadata/job-job123.log", url)
}

func TestFile_GetJobWorkerLogURL(t *testing.T) {
	f, _ := NewFile(mockFs(), "", "")

	url := f.GetJobWorkerLogURL("job123")
	assert.Equal(t, "file://metadata/job-job123-worker.log", url)
}

func TestFile_CheckJobLogExists(t *testing.T) {
	fs := mockFs()
	f, _ := NewFile(fs, "", "")

	_ = fs.MkdirAll("metadata", 0755)
	flog, _ := fs.Create(filepath.Join("metadata", "job-exists.log"))
	_, _ = flog.WriteString("log content")
	_ = flog.Close()

	exists, err := f.CheckJobLogExists(context.Background(), "exists")
	assert.NoError(t, err)
	assert.True(t, exists)

	exists, err = f.CheckJobLogExists(context.Background(), "notexists")
	assert.NoError(t, err)
	assert.False(t, exists)
}

func TestFile_CheckJobWorkerLogExists(t *testing.T) {
	fs := mockFs()
	f, _ := NewFile(fs, "", "")

	_ = fs.MkdirAll("metadata", 0755)
	flog, _ := fs.Create(filepath.Join("metadata", "job-exists-worker.log"))
	_, _ = flog.WriteString("worker log content")
	_ = flog.Close()

	exists, err := f.CheckJobWorkerLogExists(context.Background(), "exists")
	assert.NoError(t, err)
	assert.True(t, exists)

	exists, err = f.CheckJobWorkerLogExists(context.Background(), "notexists")
	assert.NoError(t, err)
	assert.False(t, exists)
}

func TestFile_GetJobUserFacingLogURL(t *testing.T) {
	f, _ := NewFile(mockFs(), "", "")
	
	url := f.GetJobUserFacingLogURL("job123")
	assert.Equal(t, "file://metadata/job-job123-user-facing.log", url)
}

func TestFile_CheckJobUserFacingLogExists(t *testing.T) {
	fs := mockFs()
	f, _ := NewFile(fs, "", "")
	
	_ = fs.MkdirAll("metadata", 0755)
	flog, _ := fs.Create(filepath.Join("metadata", "job-exists-user-facing.log"))
	_, _ = flog.WriteString("user-facing log content")
	_ = flog.Close()
	
	exists, err := f.CheckJobUserFacingLogExists(context.Background(), "exists")
	assert.NoError(t, err)
	assert.True(t, exists)
	
	exists, err = f.CheckJobUserFacingLogExists(context.Background(), "notexists")
	assert.NoError(t, err)
	assert.False(t, exists)
}

func mockFs() afero.Fs {
	files := map[string]string{
		filepath.Join("assets", "xxx.txt"):    "hello",
		filepath.Join("published", "s.json"):  "{}",
		filepath.Join("workflows", "xxx.txt"): "hello",
	}

	fs := afero.NewMemMapFs()
	for name, content := range files {
		f, _ := fs.Create(name)
		_, _ = f.WriteString(content)
		_ = f.Close()
	}
	return fs
}
