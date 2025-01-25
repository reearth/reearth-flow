package gcs

import (
	"context"
	"io"

	"cloud.google.com/go/storage"
	"github.com/reearth/reearthx/log"
	"google.golang.org/api/iterator"
)

type realGCSClient struct {
	client *storage.Client
}

func NewRealGCSClient(client *storage.Client) GCSClient {
	return &realGCSClient{client: client}
}

func (r *realGCSClient) Bucket(name string) GCSBucket {
	return &realGCSBucket{bucket: r.client.Bucket(name)}
}

type realGCSBucket struct {
	bucket *storage.BucketHandle
}

func (b *realGCSBucket) ListObjects(ctx context.Context, prefix string) ([]string, error) {
	var names []string

	it := b.bucket.Objects(ctx, &storage.Query{
		Prefix: prefix,
	})
	for {
		attrs, err := it.Next()
		if err == iterator.Done {
			break
		}
		if err != nil {
			return nil, err
		}
		names = append(names, attrs.Name)
	}

	return names, nil
}

func (b *realGCSBucket) ReadObject(ctx context.Context, objectName string) ([]byte, error) {
	r, err := b.bucket.Object(objectName).NewReader(ctx)
	if err != nil {
		return nil, err
	}
	defer func() {
		if closeErr := r.Close(); closeErr != nil {
			log.Errorf("failed to close reader: %v", closeErr)
			if err == nil {
				err = closeErr
			}
		}
	}()

	data, err := io.ReadAll(r)
	if err != nil {
		return nil, err
	}
	return data, nil
}
