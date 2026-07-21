package mongo

import (
	"context"
	"errors"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo/options"
)

// diagnosticIndexKeys mirrors the subscriber's index declarations so both
// sides converge on the same indexes. No unique index: SaveTerminalDiagnostics
// relies on SaveOne's upsert (ReplaceOne) semantics against deterministic
// IDs instead.
var diagnosticIndexKeys = []string{"jobId,nodeId", "jobId"}

// diagnosticHasCodeFilter scopes reads to rows with a top-level "code"
// field, excluding the per-job JobDiagnosticsSummaryDocument row (which has
// none and would otherwise decode into a mostly-empty Diagnostic).
var diagnosticHasCodeFilter = bson.M{"$exists": true}

type NodeDiagnostics struct {
	client *mongox.ClientCollection
}

func NewNodeDiagnostics(client *mongox.Client) repo.NodeDiagnostics {
	return &NodeDiagnostics{client: client.WithCollection("nodeDiagnostics")}
}

func (r *NodeDiagnostics) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, diagnosticIndexKeys, nil)
}

// FindByJobNodeID scopes to one node's rows. An empty nodeID (mirroring
// gateway.Redis.GetNodeDiagnostics' "" → "_job" fallback) reads the
// job-level bucket via the normalized sentinel nodeId field.
func (r *NodeDiagnostics) FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	if nodeID == "" {
		nodeID = mongodoc.JobDiagnosticNodeSegment
	}
	filter := bson.M{
		"jobId":  jobID.String(),
		"nodeId": nodeID,
		"code":   diagnosticHasCodeFilter,
	}
	return r.find(ctx, filter)
}

func (r *NodeDiagnostics) FindByJobID(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	filter := bson.M{
		"jobId": jobID.String(),
		"code":  diagnosticHasCodeFilter,
	}
	return r.find(ctx, filter)
}

// FindJobSummary reads the single per-job summary row by its deterministic
// ID (mongodoc.JobDiagnosticsSummaryID), returning (nil, nil) when no such
// row exists (no droppedEventCount was ever persisted for this job).
func (r *NodeDiagnostics) FindJobSummary(ctx context.Context, jobID id.JobID) (*uint64, error) {
	filter := bson.M{"id": mongodoc.JobDiagnosticsSummaryID(jobID)}
	c := mongodoc.NewJobDiagnosticsSummaryConsumer()
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		if errors.Is(err, rerror.ErrNotFound) {
			return nil, nil
		}
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	if len(c.Result) == 0 {
		return nil, nil
	}
	return c.Result[0], nil
}

func (r *NodeDiagnostics) find(ctx context.Context, filter interface{}) ([]*diagnostic.Diagnostic, error) {
	c := mongodoc.NewDiagnosticConsumer()
	sortByTimestamp := options.Find().SetSort(bson.D{{Key: "timestamp", Value: 1}})
	if err := r.client.Find(ctx, filter, c, sortByTimestamp); err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, nil
}

func (r *NodeDiagnostics) SaveTerminalDiagnostics(
	ctx context.Context,
	jobID id.JobID,
	workflowID string,
	timestamp time.Time,
	failedNodes []*diagnostic.Diagnostic,
	aggregated []*diagnostic.Diagnostic,
	droppedEventCount *uint64,
) error {
	for _, fn := range failedNodes {
		doc := mongodoc.NewFailedNodeDocument(jobID, workflowID, fn)
		if err := r.client.SaveOne(ctx, doc.ID, doc); err != nil {
			return rerror.ErrInternalByWithContext(ctx, err)
		}
	}

	// Each aggregated diagnostic gets its own row (like failedNodes above)
	// so it stays visible to FindByJobNodeID.
	for _, agg := range aggregated {
		doc := mongodoc.NewAggregatedDiagnosticDocument(jobID, workflowID, agg)
		if err := r.client.SaveOne(ctx, doc.ID, doc); err != nil {
			return rerror.ErrInternalByWithContext(ctx, err)
		}
	}

	if droppedEventCount == nil {
		return nil
	}

	summary := mongodoc.NewJobDiagnosticsSummaryDocument(jobID, timestamp, droppedEventCount)
	if err := r.client.SaveOne(ctx, summary.ID, summary); err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}

	return nil
}
