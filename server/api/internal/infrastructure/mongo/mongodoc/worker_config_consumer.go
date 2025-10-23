package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"go.mongodb.org/mongo-driver/bson"
)

type WorkerConfigConsumer struct {
	Result    []*batchconfig.WorkerConfig
	workspace id.WorkspaceID
}

func NewWorkerConfigConsumer(workspace id.WorkspaceID) *WorkerConfigConsumer {
	return &WorkerConfigConsumer{workspace: workspace}
}

func (c *WorkerConfigConsumer) Consume(raw bson.Raw) error {
	doc := &WorkerConfigDocument{}
	if err := bson.Unmarshal(raw, doc); err != nil {
		return err
	}
	if doc.CreatedAt.IsZero() {
		doc.CreatedAt = time.Now()
	}
	if doc.UpdatedAt.IsZero() {
		doc.UpdatedAt = doc.CreatedAt
	}
	cfg := doc.Model()
	if cfg == nil {
		return nil
	}
	if workspaceIsZero(c.workspace) || cfg.Workspace() == c.workspace {
		c.Result = append(c.Result, cfg)
	}
	return nil
}

func workspaceIsZero(ws id.WorkspaceID) bool {
	var zero id.WorkspaceID
	return ws == zero
}
