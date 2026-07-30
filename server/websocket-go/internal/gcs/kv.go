package gcs

import (
	"context"
	"errors"
	"io"

	"cloud.google.com/go/storage"
	"google.golang.org/api/iterator"
)

// kv is the minimal GCS object store the adapter needs: get/put/delete by exact
// name, and list names under a prefix. Every list is prefix-scoped so no
// operation enumerates across projects.
type kv struct {
	bucket *storage.BucketHandle
}

// errNotFound is returned by get when the object does not exist (404).
var errNotFound = errors.New("gcs: object not found")

func (s kv) get(ctx context.Context, name string) ([]byte, error) {
	r, err := s.bucket.Object(name).NewReader(ctx)
	if err != nil {
		if errors.Is(err, storage.ErrObjectNotExist) {
			return nil, errNotFound
		}
		return nil, err
	}
	defer func() { _ = r.Close() }()
	return io.ReadAll(r)
}

func (s kv) put(ctx context.Context, name string, data []byte) error {
	w := s.bucket.Object(name).NewWriter(ctx)
	if _, err := w.Write(data); err != nil {
		_ = w.Close()
		return err
	}
	return w.Close()
}

func (s kv) delete(ctx context.Context, name string) error {
	err := s.bucket.Object(name).Delete(ctx)
	if errors.Is(err, storage.ErrObjectNotExist) {
		return nil // idempotent
	}
	return err
}

// list returns every object name with the given prefix. The caller MUST scope
// the prefix to a project (no unscoped bucket list).
func (s kv) list(ctx context.Context, prefix string) ([]string, error) {
	var out []string
	it := s.bucket.Objects(ctx, &storage.Query{Prefix: prefix})
	for {
		attrs, err := it.Next()
		if errors.Is(err, iterator.Done) {
			break
		}
		if err != nil {
			return nil, err
		}
		out = append(out, attrs.Name)
	}
	return out, nil
}

// listPrefixes returns the immediate child "{id}/" prefixes under prefix using a
// "/" delimiter, retaining the trailing slash. Not a recursive object walk.
func (s kv) listPrefixes(ctx context.Context, prefix string) ([]string, error) {
	var out []string
	it := s.bucket.Objects(ctx, &storage.Query{Prefix: prefix, Delimiter: "/"})
	for {
		attrs, err := it.Next()
		if errors.Is(err, iterator.Done) {
			break
		}
		if err != nil {
			return nil, err
		}
		if attrs.Prefix != "" {
			out = append(out, attrs.Prefix)
		}
	}
	return out, nil
}

// listAttrs is like list but returns full attrs (used for UpdatedAt metadata).
func (s kv) listAttrs(ctx context.Context, prefix string) ([]*storage.ObjectAttrs, error) {
	var out []*storage.ObjectAttrs
	it := s.bucket.Objects(ctx, &storage.Query{Prefix: prefix})
	for {
		attrs, err := it.Next()
		if errors.Is(err, iterator.Done) {
			break
		}
		if err != nil {
			return nil, err
		}
		out = append(out, attrs)
	}
	return out, nil
}
