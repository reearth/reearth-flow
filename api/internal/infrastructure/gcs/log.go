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

type GCSClient interface {
	Bucket(name string) GCSBucket
}

type GCSBucket interface {
	ListObjects(ctx context.Context, prefix string) ([]string, error)
	ReadObject(ctx context.Context, objectName string) ([]byte, error)
}

type gcsLog struct {
	client     GCSClient
	bucketName string
}

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

type LogEntry struct {
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	NodeID     *string   `json:"nodeId"`
	Timestamp  time.Time `json:"timestamp"`
	LogLevel   log.Level `json:"logLevel"`
	Message    string    `json:"message"`
}

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

func dateTruncate(t time.Time) time.Time {
	t = t.UTC()
	y, m, d := t.Date()
	return time.Date(y, m, d, 0, 0, 0, 0, time.UTC)
}

func (g *gcsLog) GetLogs(
	ctx context.Context,
	since time.Time,
	until time.Time,
	workflowID id.WorkflowID,
	jobID id.JobID,
) ([]*log.Log, error) {
	since = since.UTC()
	until = until.UTC()

	start := dateTruncate(since)
	end := dateTruncate(until)

	const maxDays = 30
	if end.Sub(start).Hours() > float64(maxDays*24) {
		return nil, fmt.Errorf("date range too large, max %d days allowed", maxDays)
	}

	var allLogs []*log.Log
	bucket := g.client.Bucket(g.bucketName)

	for d := start; !d.After(end); d = d.AddDate(0, 0, 1) {
		// Example: artifacts/logs/2025/01/24/workflow-123/job-abc/
		prefix := fmt.Sprintf(
			"artifacts/logs/%04d/%02d/%02d/%s/%s/",
			d.Year(), d.Month(), d.Day(),
			workflowID.String(),
			jobID.String(),
		)

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

			entry.Timestamp = entry.Timestamp.UTC()

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
