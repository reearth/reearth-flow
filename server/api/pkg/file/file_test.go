package file

import (
	"context"
	"io"
	"mime"
	"net/http"
	"os"
	"path"
	"path/filepath"
	"testing"

	"github.com/jarcoal/httpmock"
	"github.com/samber/lo"
	"github.com/spf13/afero"
	"github.com/stretchr/testify/assert"
)

func TestSimpleIterator(t *testing.T) {
	a := NewSimpleIterator([]File{{Path: "a"}, {Path: "b"}, {Path: "c"}})

	n, err := a.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "a"}, n)

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "b"}, n)

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "c"}, n)

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Nil(t, n)

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Nil(t, n)
}

func TestPrefixIterator(t *testing.T) {
	ba := NewSimpleIterator([]File{
		{Path: "a"}, {Path: "b"}, {Path: "c/d"}, {Path: "e"}, {Path: "f/g/h"}, {Path: "c/i/j"},
	})
	a := NewPrefixIterator(ba, "c")

	n, err := a.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "d"}, n)

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "i/j"}, n)

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Nil(t, n)

	ba2 := NewSimpleIterator([]File{
		{Path: "a"}, {Path: "b"},
	})
	a2 := NewPrefixIterator(ba2, "")

	n2, err := a2.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "a"}, n2)

	n2, err = a2.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "b"}, n2)

	n2, err = a2.Next()
	assert.NoError(t, err)
	assert.Nil(t, n2)
}

func TestFilteredIterator(t *testing.T) {
	var paths []string
	ba := NewSimpleIterator([]File{
		{Path: "0"}, {Path: "1"}, {Path: "2"},
	})
	a := NewFilteredIterator(ba, func(p string) bool {
		paths = append(paths, p)
		return p == "1"
	})

	n, err := a.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "0"}, n)

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Equal(t, &File{Path: "2"}, n)

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Nil(t, n)
	assert.Equal(t, []string{"0", "1", "2"}, paths)
}

func TestFsIterator(t *testing.T) {
	fs := afero.NewMemMapFs()
	_ = fs.MkdirAll(filepath.Join("a", "b"), 0755)
	f, _ := fs.Create("b")
	_, _ = f.WriteString("hello")
	_ = f.Close()
	_, _ = fs.Create(filepath.Join("a", "b", "c"))

	a, err := NewFsIterator(fs)
	assert.NoError(t, err)

	n, err := a.Next()
	assert.NoError(t, err)
	assert.Equal(t, filepath.Join("a", "b", "c"), n.Path)
	nd, err := io.ReadAll(n.Content)
	assert.NoError(t, err)
	assert.Equal(t, []byte{}, nd)
	assert.NoError(t, n.Content.Close())

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Equal(t, "b", n.Path)
	nd, err = io.ReadAll(n.Content)
	assert.NoError(t, err)
	assert.Equal(t, "hello", string(nd))
	assert.NoError(t, n.Content.Close())

	n, err = a.Next()
	assert.NoError(t, err)
	assert.Nil(t, n)
}

func TestFromURL(t *testing.T) {
	ctx := context.Background()

	httpmock.Activate()
	defer httpmock.Deactivate()

	t.Run("with gzip encoding", func(t *testing.T) {
		URL := "https://cms.com/xyz/test.txt.gz"
		f := lo.Must(os.Open("testdata/test.txt"))
		defer func(f *os.File) {
			err := f.Close()
			if err != nil {
				t.Fatalf("failed to close file: %v", err)
			}
		}(f)
		z := lo.Must(io.ReadAll(f))

		httpmock.RegisterResponder("GET", URL, func(r *http.Request) (*http.Response, error) {
			res := httpmock.NewBytesResponse(200, z)
			res.Header.Set("Content-Type", mime.TypeByExtension(path.Ext(URL)))
			res.Header.Set("Content-Length", "123")
			res.Header.Set("Content-Encoding", "gzip")
			return res, nil
		})

		got, err := FromURL(ctx, URL)
		assert.NoError(t, err)
		assert.Equal(t, "gzip", got.ContentEncoding)
	})

	t.Run("normal", func(t *testing.T) {
		URL := "https://cms.com/xyz/test.txt"
		f := lo.Must(os.Open("testdata/test.txt"))
		defer func(f *os.File) {
			err := f.Close()
			if err != nil {
				t.Fatalf("failed to close file: %v", err)
			}
		}(f)
		z := lo.Must(io.ReadAll(f))

		httpmock.RegisterResponder("GET", URL, func(r *http.Request) (*http.Response, error) {
			res := httpmock.NewBytesResponse(200, z)
			res.Header.Set("Content-Type", mime.TypeByExtension(path.Ext(URL)))
			res.Header.Set("Content-Length", "123")
			res.Header.Set("Content-Disposition", `attachment; filename="filename.txt"`)
			return res, nil
		})

		expected := File{Path: "filename.txt", Content: f, Size: 123}

		got, err := FromURL(ctx, URL)
		assert.NoError(t, err)
		assert.Equal(t, expected.Path, got.Path)
		assert.Equal(t, z, lo.Must(io.ReadAll(got.Content)))

		httpmock.RegisterResponder("GET", URL, func(r *http.Request) (*http.Response, error) {
			res := httpmock.NewBytesResponse(200, z)
			res.Header.Set("Content-Type", mime.TypeByExtension(path.Ext(URL)))
			return res, nil
		})

		expected = File{Path: "test.txt", Content: f, Size: 0}

		got, err = FromURL(ctx, URL)
		assert.NoError(t, err)
		assert.Equal(t, expected.Path, got.Path)
		assert.Equal(t, z, lo.Must(io.ReadAll(got.Content)))
	})
}
