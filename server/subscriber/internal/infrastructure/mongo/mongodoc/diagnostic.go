package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

// diagnosticSourceSpanDocument / diagnosticAggregateInfoDocument are bson
// mirrors of diagnostic.WireSourceSpan / diagnostic.WireAggregateInfo (those
// pkg types only carry json tags, since they are the pub/sub wire shape).
type diagnosticSourceSpanDocument struct {
	Length *uint `bson:"length,omitempty"`
	Offset uint  `bson:"offset"`
}

type diagnosticAggregateInfoDocument struct {
	SampleFeatureIDs []string `bson:"sampleFeatureIds"`
	Count            uint64   `bson:"count"`
}

// DiagnosticDocument is a single per-node (or per-job, when NodeID is nil)
// diagnostic row in the nodeDiagnostics collection. Unlike
// NodeExecutionDocument, rows are appended rather than upserted: ID is
// unique per event, not per {jobId,nodeId}.
type DiagnosticDocument struct {
	Timestamp            time.Time                        `bson:"timestamp"`
	Aggregated           *diagnosticAggregateInfoDocument `bson:"aggregated,omitempty"`
	SourceSpan           *diagnosticSourceSpanDocument    `bson:"sourceSpan,omitempty"`
	EffectiveDisposition *string                          `bson:"effectiveDisposition,omitempty"`
	NodeID               *string                          `bson:"nodeId,omitempty"`
	ActionType           *string                          `bson:"actionType,omitempty"`
	FeatureID            *string                          `bson:"featureId,omitempty"`
	Help                 *string                          `bson:"help,omitempty"`
	ID                   string                           `bson:"id"`
	JobID                string                           `bson:"jobId"`
	WorkflowID           string                           `bson:"workflowId"`
	Schema               string                           `bson:"schema"`
	Code                 string                           `bson:"code"`
	Category             string                           `bson:"category"`
	Severity             string                           `bson:"severity"`
	Message              string                           `bson:"message"`
}

// NewDiagnosticDocument builds the persisted row for a DiagnosticEvent. The
// id is synthesized as {jobId}:{nodeId-or-_job}:{ObjectID} so rows append
// rather than collide, while the jobId/nodeId bson fields stay queryable
// (NodeID is nil, not the literal "_job", when the event has no nodeId).
func NewDiagnosticDocument(event *diagnostic.DiagnosticEvent) DiagnosticDocument {
	doc := DiagnosticDocument{
		Timestamp:            event.Timestamp,
		EffectiveDisposition: event.EffectiveDisposition,
		NodeID:               event.NodeID,
		ActionType:           event.ActionType,
		FeatureID:            event.FeatureID,
		Help:                 event.Help,
		ID:                   diagnosticDocumentID(event),
		JobID:                event.JobID,
		WorkflowID:           event.WorkflowID,
		Schema:               event.Schema,
		Code:                 event.Code,
		Category:             event.Category,
		Severity:             event.Severity,
		Message:              event.Message,
	}

	if event.SourceSpan != nil {
		doc.SourceSpan = &diagnosticSourceSpanDocument{
			Offset: event.SourceSpan.Offset,
			Length: event.SourceSpan.Length,
		}
	}

	if event.Aggregated != nil {
		doc.Aggregated = &diagnosticAggregateInfoDocument{
			Count:            event.Aggregated.Count,
			SampleFeatureIDs: event.Aggregated.SampleFeatureIds,
		}
	}

	return doc
}

func diagnosticDocumentID(event *diagnostic.DiagnosticEvent) string {
	nodeSegment := "_job"
	if event.NodeID != nil && *event.NodeID != "" {
		nodeSegment = *event.NodeID
	}
	return event.JobID + ":" + nodeSegment + ":" + primitive.NewObjectID().Hex()
}
