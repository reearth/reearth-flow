package mongo

import (
	"context"
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

// diagnosticIndexKeys mirrors the subscriber's index declarations
// (server/subscriber/internal/infrastructure/mongo/diagnostic.go) so both
// sides converge on the same {jobId,nodeId} + {jobId} indexes. No unique
// index: the subscriber's own rows are append-only (random ObjectID-suffixed
// IDs); this repo's SaveTerminalDiagnostics writes rely on SaveOne's upsert
// (ReplaceOne) semantics against deterministic IDs instead.
var diagnosticIndexKeys = []string{"jobId,nodeId", "jobId"}

// diagnosticHasCodeFilter scopes reads to rows carrying a top-level "code"
// field, excluding the single per-job JobDiagnosticsSummaryDocument row
// (see mongodoc.NewJobDiagnosticsSummaryDocument), which has no such field
// and would otherwise decode into a mostly-empty Diagnostic.
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

func (r *NodeDiagnostics) FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
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
	timestamp time.Time,
	failedNodes []*diagnostic.Diagnostic,
	aggregated []*diagnostic.Diagnostic,
	droppedEventCount *uint64,
) error {
	for _, fn := range failedNodes {
		doc := mongodoc.NewFailedNodeDocument(jobID, fn)
		if err := r.client.SaveOne(ctx, doc.ID, doc); err != nil {
			return rerror.ErrInternalByWithContext(ctx, err)
		}
	}

	// Each aggregated diagnostic gets its own row (like failedNodes above),
	// not a nested entry inside the summary row below: nesting would make it
	// invisible to FindByJobNodeID for the node it pertains to.
	for _, agg := range aggregated {
		doc := mongodoc.NewAggregatedDiagnosticDocument(jobID, agg)
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
