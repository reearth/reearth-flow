// Package health provides the Redis and GCS probes for the /health endpoint.
package health

import (
	"context"

	storage "cloud.google.com/go/storage"
	"github.com/redis/go-redis/v9"
	"google.golang.org/api/iterator"
	"google.golang.org/api/option"
)

// RedisPinger probes Redis liveness via PING.
type RedisPinger struct {
	client *redis.Client
}

// NewRedisPinger builds a RedisPinger from a redis:// URL.
func NewRedisPinger(redisURL string) (*RedisPinger, error) {
	opt, err := redis.ParseURL(redisURL)
	if err != nil {
		return nil, err
	}
	return &RedisPinger{client: redis.NewClient(opt)}, nil
}

// Ping issues a Redis PING.
func (p *RedisPinger) Ping(ctx context.Context) error {
	return p.client.Ping(ctx).Err()
}

// Close releases the underlying Redis connection pool.
func (p *RedisPinger) Close() error { return p.client.Close() }

// GCSLister probes GCS by listing a single object in the bucket.
type GCSLister struct {
	bucket string
	client *storage.Client
}

// NewGCSLister builds a GCSLister for bucket. endpoint, when non-empty,
// overrides the GCS endpoint (fake-gcs in dev).
func NewGCSLister(ctx context.Context, bucket, endpoint string) (*GCSLister, error) {
	var opts []option.ClientOption
	if endpoint != "" {
		// Mirror cmd/websocket: a custom endpoint (fake-gcs in dev/test) has no
		// credentials, so disable auth to avoid ADC lookups failing the probe.
		opts = append(opts, option.WithEndpoint(endpoint), option.WithoutAuthentication())
	}
	client, err := storage.NewClient(ctx, opts...)
	if err != nil {
		return nil, err
	}
	return &GCSLister{bucket: bucket, client: client}, nil
}

// List lists a single object to confirm bucket reachability and access.
func (l *GCSLister) List(ctx context.Context) error {
	it := l.client.Bucket(l.bucket).Objects(ctx, &storage.Query{})
	it.PageInfo().MaxSize = 1
	_, err := it.Next()
	if err == iterator.Done {
		return nil // empty bucket is healthy
	}
	return err
}

// Close releases the underlying GCS client.
func (l *GCSLister) Close() error { return l.client.Close() }
