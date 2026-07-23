package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// Mirrors (bson field-for-field) the subscriber's mongodoc.DiagnosticDocument
// — keep tags in lockstep with that file.
type diagnosticSourceSpanDocument struct {
	Length *uint `bson:"length,omitempty"`
	Offset uint  `bson:"offset"`
}

type diagnosticAggregateInfoDocument struct {
	SampleFeatureIDs []string `bson:"sampleFeatureIds"`
	Count            uint64   `bson:"count"`
}

// Terminal rows (see NewFailedNodeDocument / NewAggregatedDiagnosticDocument)
// use a deterministic ID, not a random suffix, so redeliveries upsert
// instead of duplicating.
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

// Mirrors the subscriber's constant — keep in lockstep. Model() strips it
// back to nil before it reaches GraphQL.
const JobDiagnosticNodeSegment = "_job"

func normalizedNodeSegment(nodeID *string) string {
	if nodeID != nil && *nodeID != "" {
		return *nodeID
	}
	return JobDiagnosticNodeSegment
}

type DiagnosticConsumer = Consumer[*DiagnosticDocument, *diagnostic.Diagnostic]

func NewDiagnosticConsumer() *DiagnosticConsumer {
	return NewConsumer[*DiagnosticDocument](func(d *diagnostic.Diagnostic) bool {
		return true
	})
}

func (d *DiagnosticDocument) Model() (*diagnostic.Diagnostic, error) {
	if d == nil {
		return nil, nil
	}

	jobID, err := id.JobIDFrom(d.JobID)
	if err != nil {
		return nil, err
	}

	nodeID := d.NodeID
	if nodeID != nil && *nodeID == JobDiagnosticNodeSegment {
		nodeID = nil
	}

	b := diagnostic.NewBuilder().
		JobID(jobID).
		Timestamp(d.Timestamp).
		Code(d.Code).
		Category(d.Category).
		Severity(d.Severity).
		EffectiveDisposition(d.EffectiveDisposition).
		NodeID(nodeID).
		ActionType(d.ActionType).
		FeatureID(d.FeatureID).
		Message(d.Message).
		Help(d.Help).
		Terminal(d.Schema == jobCompleteDiagnosticSchemaTag)

	if d.Aggregated != nil {
		b = b.Aggregated(diagnostic.NewAggregateInfo(d.Aggregated.Count, d.Aggregated.SampleFeatureIDs))
	}
	if d.SourceSpan != nil {
		b = b.SourceSpan(diagnostic.NewSourceSpan(d.SourceSpan.Offset, d.SourceSpan.Length))
	}

	return b.Build()
}

const jobCompleteDiagnosticSchemaTag = "job-complete.v1"

func NewFailedNodeDocument(jobID id.JobID, workflowID string, d *diagnostic.Diagnostic) DiagnosticDocument {
	return newTerminalDiagnosticDocument(jobID, workflowID, d, "failed")
}

func NewAggregatedDiagnosticDocument(jobID id.JobID, workflowID string, d *diagnostic.Diagnostic) DiagnosticDocument {
	return newTerminalDiagnosticDocument(jobID, workflowID, d, "aggregated")
}

func newTerminalDiagnosticDocument(jobID id.JobID, workflowID string, d *diagnostic.Diagnostic, kind string) DiagnosticDocument {
	nodeSegment := normalizedNodeSegment(d.NodeID())

	doc := DiagnosticDocument{
		Timestamp:            d.Timestamp(),
		EffectiveDisposition: d.EffectiveDisposition(),
		NodeID:               &nodeSegment,
		ActionType:           d.ActionType(),
		FeatureID:            d.FeatureID(),
		Help:                 d.Help(),
		ID:                   jobID.String() + ":" + nodeSegment + ":" + kind + ":" + d.Code(),
		JobID:                jobID.String(),
		WorkflowID:           workflowID,
		Schema:               jobCompleteDiagnosticSchemaTag,
		Code:                 d.Code(),
		Category:             d.Category(),
		Severity:             d.Severity(),
		Message:              d.Message(),
	}

	if agg := d.Aggregated(); agg != nil {
		doc.Aggregated = &diagnosticAggregateInfoDocument{Count: agg.Count(), SampleFeatureIDs: agg.SampleFeatureIDs()}
	}
	if ss := d.SourceSpan(); ss != nil {
		doc.SourceSpan = &diagnosticSourceSpanDocument{Offset: ss.Offset(), Length: ss.Length()}
	}

	return doc
}

// Deliberately doesn't reuse DiagnosticDocument's shape: the
// {"code": {"$exists": true}} filter in FindByJobNodeID/FindByJobID relies
// on this row having no code field.
type JobDiagnosticsSummaryDocument struct {
	Timestamp         time.Time `bson:"timestamp"`
	DroppedEventCount *uint64   `bson:"droppedEventCount,omitempty"`
	ID                string    `bson:"id"`
	JobID             string    `bson:"jobId"`
}

func JobDiagnosticsSummaryID(jobID id.JobID) string {
	return jobID.String() + ":" + JobDiagnosticNodeSegment + ":summary"
}

func NewJobDiagnosticsSummaryDocument(
	jobID id.JobID,
	timestamp time.Time,
	droppedEventCount *uint64,
) JobDiagnosticsSummaryDocument {
	return JobDiagnosticsSummaryDocument{
		Timestamp:         timestamp,
		DroppedEventCount: droppedEventCount,
		ID:                JobDiagnosticsSummaryID(jobID),
		JobID:             jobID.String(),
	}
}

// DroppedEventCount is never nil here: SaveTerminalDiagnostics only writes
// this row when it's non-nil.
func (d *JobDiagnosticsSummaryDocument) Model() (*uint64, error) {
	if d == nil {
		return nil, nil
	}
	return d.DroppedEventCount, nil
}

type JobDiagnosticsSummaryConsumer = Consumer[*JobDiagnosticsSummaryDocument, *uint64]

func NewJobDiagnosticsSummaryConsumer() *JobDiagnosticsSummaryConsumer {
	return NewConsumer[*JobDiagnosticsSummaryDocument](nil)
}
