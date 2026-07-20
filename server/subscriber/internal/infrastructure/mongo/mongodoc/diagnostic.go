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

// JobDiagnosticNodeSegment is the sentinel node segment used for a
// diagnostic with no node context (job-level) — both in the
// {jobId}:{nodeId}:{ObjectID} document ID and in the nodeId bson field
// itself. Storing the sentinel in the field too (not just the ID) keeps the
// api's FindByJobNodeID field-equality lookups symmetric with the ID
// convention: previously an absent/empty nodeId left the bson field nil (or
// the raw empty string) while the ID still got the "_job" segment, so a
// "_job" lookup against the field could never match these rows.
const JobDiagnosticNodeSegment = "_job"

// normalizedNodeSegment returns nodeID's value, or JobDiagnosticNodeSegment
// when nodeID is nil or the empty string.
func normalizedNodeSegment(nodeID *string) string {
	if nodeID != nil && *nodeID != "" {
		return *nodeID
	}
	return JobDiagnosticNodeSegment
}

// NewDiagnosticDocument builds the persisted row for a DiagnosticEvent. The
// id is synthesized as {jobId}:{nodeId-or-_job}:{ObjectID} so rows append
// rather than collide; the nodeId bson field carries the same
// nodeId-or-_job value (see JobDiagnosticNodeSegment) so field-equality
// reads stay consistent with the ID convention.
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
