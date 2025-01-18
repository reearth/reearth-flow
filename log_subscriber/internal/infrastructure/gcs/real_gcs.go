package gcs

import (
	"context"

	"cloud.google.com/go/storage"
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

func (b *realGCSBucket) Object(name string) GCSObject {
	return &realGCSObject{obj: b.bucket.Object(name)}
}

type realGCSObject struct {
	obj *storage.ObjectHandle
}

func (o *realGCSObject) NewWriter(ctx context.Context) GCSWriter {
	return &realGCSWriter{writer: o.obj.NewWriter(ctx)}
}

type realGCSWriter struct {
	writer *storage.Writer
}

func (w *realGCSWriter) Write(p []byte) (n int, err error) {
	return w.writer.Write(p)
}

func (w *realGCSWriter) Close() error {
	return w.writer.Close()
}

func (w *realGCSWriter) SetContentType(ct string) {
	w.writer.ContentType = ct
}
