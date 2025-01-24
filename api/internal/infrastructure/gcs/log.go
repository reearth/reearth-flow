package gcs

import (
	"context"
	"errors"
	"time"

	"cloud.google.com/go/storage"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type gcsLog struct {
	client *storage.Client
}

func NewGCSLog(client *storage.Client) (*gcsLog, error) {
	if client == nil {
		return nil, errors.New("client is nil")
	}

	return &gcsLog{client: client}, nil
}

func (g *gcsLog) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID) ([]*log.Log, error) {
	// TODO: Implement
	nodeID := log.NodeID(id.NewNodeID())
	dummyLogs := []*log.Log{
		log.NewLog(id.NewWorkflowID(), id.NewJobID(), nil, time.Now().UTC(), log.LevelInfo, "Test log message 1 from gcs"),
		log.NewLog(id.NewWorkflowID(), id.NewJobID(), &nodeID, time.Now().UTC(), log.LevelDebug, "Test log message 2 from gcs"),
	}

	return dummyLogs, nil
}
