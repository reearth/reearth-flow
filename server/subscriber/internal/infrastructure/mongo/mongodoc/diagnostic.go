package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

// diagnosticSourceSpanDocument / diagnosticAggregateInfoDocument mirror
// diagnostic.WireSourceSpan / WireAggregateInfo in bson (those pkg types
// only carry json tags).
type diagnosticSourceSpanDocument struct {
	Length *uint `bson:"length,omitempty"`
	Offset uint  `bson:"offset"`
}

type diagnosticAggregateInfoDocument struct {
	SampleFeatureIDs []string `bson:"sampleFeatureIds"`
	Count            uint64   `bson:"count"`
}

// DiagnosticDocument is a single per-node (or per-job, when NodeID is nil)
// row in the nodeDiagnostics collection. Unlike NodeExecutionDocument, rows
// are appended rather than upserted: ID is unique per event.
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

// JobDiagnosticNodeSegment is the sentinel node segment for a diagnostic
// with no node context, used both in the {jobId}:{nodeId}:{ObjectID}
// document ID and the nodeId bson field itself, so the api's
// FindByJobNodeID field-equality lookups stay symmetric with the ID.
const JobDiagnosticNodeSegment = "_job"

// normalizedNodeSegment returns nodeID's value, or JobDiagnosticNodeSegment
// when nodeID is nil or the empty string.
func normalizedNodeSegment(nodeID *string) string {
	if nodeID != nil && *nodeID != "" {
		return *nodeID
	}
	return JobDiagnosticNodeSegment
}

// NewDiagnosticDocument builds the persisted row for a DiagnosticEvent; the
// id is {jobId}:{nodeId-or-_job}:{ObjectID} so rows append rather than
// collide.
func NewDiagnosticDocument(event *diagnostic.DiagnosticEvent) DiagnosticDocument {
	nodeSegment := normalizedNodeSegment(event.NodeID)

	doc := DiagnosticDocument{
		Timestamp:            event.Timestamp,
		EffectiveDisposition: event.EffectiveDisposition,
		NodeID:               &nodeSegment,
		ActionType:           event.ActionType,
		FeatureID:            event.FeatureID,
		Help:                 event.Help,
		ID:                   event.JobID + ":" + nodeSegment + ":" + primitive.NewObjectID().Hex(),
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
