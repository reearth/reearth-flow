package gcs

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	reearth_log "github.com/reearth/reearthx/log"
)

// GCSClient is an interface for retrieving GCS buckets.
type GCSClient interface {
	Bucket(name string) GCSBucket
}

// GCSBucket is an interface that provides the ability to list and read objects.
type GCSBucket interface {
	ListObjects(ctx context.Context, prefix string) ([]string, error)
	ReadObject(ctx context.Context, objectName string) ([]byte, error)
}

// gcsLog is an implementation for reading logs from GCS.
type gcsLog struct {
	client     GCSClient
	bucketName string
}

// NewGCSLog returns a new gcsLog instance.
func NewGCSLog(client GCSClient, bucketName string) (*gcsLog, error) {
	if client == nil {
		return nil, errors.New("gcs client is nil")
	}
	if bucketName == "" {
		return nil, errors.New("bucketName must not be empty")
	}
	return &gcsLog{
		client:     client,
		bucketName: bucketName,
	}, nil
}

// LogEntry is the structure of a log stored in GCS as JSON.
type LogEntry struct {
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	NodeID     *string   `json:"nodeId"`
	Timestamp  time.Time `json:"timestamp"`
	LogLevel   log.Level `json:"logLevel"`
	Message    string    `json:"message"`
}

// ToLogEntry is a helper function to convert a domain's log.Log to a LogEntry for GCS.
func ToLogEntry(l *log.Log) *LogEntry {
	if l == nil {
		return nil
	}
	var nid *string
	if l.NodeID() != nil {
		s := l.NodeID().String()
		nid = &s
	}
	return &LogEntry{
		WorkflowID: l.WorkflowID().String(),
		JobID:      l.JobID().String(),
		NodeID:     nid,
		Timestamp:  l.Timestamp().UTC(),
		LogLevel:   l.Level(),
		Message:    l.Message(),
	}
}

// ToDomain converts a GCS LogEntry to a domain's log.Log.
func (e *LogEntry) ToDomain() (*log.Log, error) {
	wid, err := id.WorkflowIDFrom(e.WorkflowID)
	if err != nil {
		return nil, err
	}
	jid, err := id.JobIDFrom(e.JobID)
	if err != nil {
		return nil, err
	}

	var nodeID *log.NodeID
	if e.NodeID != nil && *e.NodeID != "" {
		nid, err2 := id.NodeIDFrom(*e.NodeID)
		if err2 == nil {
			nodeID = &nid
		}
	}

	return log.NewLog(
		wid,
		jid,
		nodeID,
		e.Timestamp.UTC(),
		e.LogLevel,
		e.Message,
	), nil
}

// dateTruncate rounds the given time to the nearest day in UTC (00:00:00).
func dateTruncate(t time.Time) time.Time {
	t = t.UTC()
	y, m, d := t.Date()
	return time.Date(y, m, d, 0, 0, 0, 0, time.UTC)
}

// GetLogs retrieves and returns logs for the specified workflowID and jobID from GCS.
// Returns only logs from since to until (inclusive) in UTC.
func (g *gcsLog) GetLogs(
	ctx context.Context,
	since time.Time,
	until time.Time,
	workflowID id.WorkflowID,
	jobID id.JobID,
) ([]*log.Log, error) {
	// Ensure both times are in UTC
	since = since.UTC()
	until = until.UTC()

	// Truncate to daily boundaries
	start := dateTruncate(since)
	end := dateTruncate(until)

	// Enforce a maximum date span of 30 days
	const maxDays = 30
	if end.Sub(start).Hours() > float64(maxDays*24) {
		return nil, fmt.Errorf("date range too large, max %d days allowed", maxDays)
	}

	var allLogs []*log.Log
	bucket := g.client.Bucket(g.bucketName)

	// Iterate day by day from start to end
	for d := start; !d.After(end); d = d.AddDate(0, 0, 1) {
		// Object prefix on GCS
		// Example: artifacts/logs/2025/01/24/workflow-123/job-abc/
		prefix := fmt.Sprintf(
			"artifacts/logs/%04d/%02d/%02d/%s/%s/",
			d.Year(), d.Month(), d.Day(),
			workflowID.String(),
			jobID.String(),
		)

		// List all objects under the prefix
		objectNames, err := bucket.ListObjects(ctx, prefix)
		if err != nil {
			return nil, fmt.Errorf("failed to list objects: %w", err)
		}

		for _, objName := range objectNames {
			data, err := bucket.ReadObject(ctx, objName)
			if err != nil {
				reearth_log.Warnfc(ctx, "gcsLog: failed to read object (%s): %v", objName, err)
				continue
			}

			var entry LogEntry
			if err := json.Unmarshal(data, &entry); err != nil {
				reearth_log.Warnfc(ctx, "gcsLog: failed to unmarshal json (%s): %v", objName, err)
				continue
			}

			// Force timestamp to UTC
			entry.Timestamp = entry.Timestamp.UTC()

			// Filter out logs outside [since, until]
			if entry.Timestamp.Before(since) || entry.Timestamp.After(until) {
				continue
			}

			dlog, err := entry.ToDomain()
			if err != nil {
				reearth_log.Warnfc(ctx, "gcsLog: failed to convert to domain (%s): %v", objName, err)
				continue
			}

			allLogs = append(allLogs, dlog)
		}
	}

	return allLogs, nil
}
