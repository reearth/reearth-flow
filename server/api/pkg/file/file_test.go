package file

import (
	"io"
	"path/filepath"
	"testing"

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
