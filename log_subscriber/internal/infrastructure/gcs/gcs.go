package gcs

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"cloud.google.com/go/storage"
	domainLog "github.com/reearth/reearth-flow/log-subscriber/pkg/log"
)

type GCSClient interface {
	Bucket(name string) *storage.BucketHandle
}

type GCSStorage struct {
	client     GCSClient
	bucketName string
}

func NewGCSStorage(client GCSClient, bucketName string) *GCSStorage {
	return &GCSStorage{
		client:     client,
		bucketName: bucketName,
	}
}

// Write LogEvent to GCS as a JSON file
func (g *GCSStorage) SaveLogToGCS(ctx context.Context, event *domainLog.LogEvent) error {
	// File path example: logs/yyyy/MM/dd/workflowId/jobId/timestamp.json
	year, month, day := event.Timestamp.UTC().Date()
	filePath := fmt.Sprintf("logs/%04d/%02d/%02d/%s/%s/%s.json",
		year, month, day,
		event.WorkflowID,
		event.JobID,
		event.Timestamp.UTC().Format(time.RFC3339),
	)

	bucket := g.client.Bucket(g.bucketName)
	obj := bucket.Object(filePath)
	writer := obj.NewWriter(ctx)
	defer writer.Close()

	writer.ContentType = "application/json"

	enc := json.NewEncoder(writer)
	if err := enc.Encode(event); err != nil {
		return err
	}

	return nil
}