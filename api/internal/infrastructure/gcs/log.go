package gcs

import (
	"context"
	"time"

	"cloud.google.com/go/storage"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type gcsLog struct {
	client *storage.Client
}

func NewGCSLog(client *storage.Client) *gcsLog {
	return &gcsLog{client: client}
}

func (g *gcsLog) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID) ([]*log.Log, error) {
	// TODO: Implement
	dummyLogs := []*log.Log{
		log.NewLog(id.NewWorkflowID(), id.NewJobID(), nil, log.LevelInfo, "Test log message 1 from gcs"),
		log.NewLog(id.NewWorkflowID(), id.NewJobID(), nil, log.LevelDebug, "Test log message 2 from gcs"),
	}

	return dummyLogs, nil
}
