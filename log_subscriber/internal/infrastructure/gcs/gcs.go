package gcs

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

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
	// File path example: artifacts/logs/yyyy/MM/dd/workflowId/jobId/timestamp.json
	year, month, day := event.Timestamp.UTC().Date()
	filePath := fmt.Sprintf("artifacts/logs/%04d/%02d/%02d/%s/%s/%s.json",
		year, month, day,
		event.WorkflowID,
		event.JobID,
		event.Timestamp.UTC().Format("2006-01-02T15:04:05.000000Z"),
	)

	bucket := g.client.Bucket(g.bucketName)
	obj := bucket.Object(filePath)
	writer := obj.NewWriter(ctx)
	defer func() {
		if err := writer.Close(); err != nil {
			log.Printf("failed to close writer: %v", err)
		}
	}()

	writer.ContentType = "application/json"

	enc := json.NewEncoder(writer)
	if err := enc.Encode(event); err != nil {
		return err
	}

	return nil
}
