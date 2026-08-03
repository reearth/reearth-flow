package gcs

import (
	"context"
	"errors"
	"io"
	"net/http"

	"cloud.google.com/go/storage"
	"google.golang.org/api/googleapi"
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

// putWithMeta writes data and attaches custom object metadata, so listAttrs can
// return a snapshot's label and uncompressed size without reading its payload.
func (s kv) putWithMeta(ctx context.Context, name string, data []byte, meta map[string]string) error {
	w := s.bucket.Object(name).NewWriter(ctx)
	w.Metadata = meta
	if _, err := w.Write(data); err != nil {
		_ = w.Close()
		return err
	}
	return w.Close()
}

// errObjectExists reports that a create-only write lost to an existing object.
var errObjectExists = errors.New("gcs: object already exists")

// putWithMetaIfAbsent is putWithMeta that REFUSES to overwrite, returning
// errObjectExists instead. Snapshot records are write-once by contract (ids are
// never reused within a room), and a plain put makes an id collision destroy the
// previous snapshot's payload, label and timestamp with no error anywhere. The
// precondition converts that silent data loss into a caller-visible failure.
func (s kv) putWithMetaIfAbsent(ctx context.Context, name string, data []byte, meta map[string]string) error {
	w := s.bucket.Object(name).If(storage.Conditions{DoesNotExist: true}).NewWriter(ctx)
	w.Metadata = meta
	if _, err := w.Write(data); err != nil {
		_ = w.Close()
		return asObjectExists(err)
	}
	return asObjectExists(w.Close())
}

// asObjectExists maps a failed DoesNotExist precondition (HTTP 412) onto
// errObjectExists, leaving every other error untouched.
func asObjectExists(err error) error {
	if err == nil {
		return nil
	}
	var gerr *googleapi.Error
	if errors.As(err, &gerr) && gerr.Code == http.StatusPreconditionFailed {
		return errObjectExists
	}
	return err
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
