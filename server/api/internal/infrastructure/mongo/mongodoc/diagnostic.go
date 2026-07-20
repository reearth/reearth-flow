package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// diagnosticSourceSpanDocument / diagnosticAggregateInfoDocument /
// DiagnosticDocument mirror (bson field-for-field) the subscriber's
// mongodoc.DiagnosticDocument
// (server/subscriber/internal/infrastructure/mongo/mongodoc/diagnostic.go),
// which writes the nodeDiagnostics collection this file reads. Keep the bson
// tags in lockstep with that file.
type diagnosticSourceSpanDocument struct {
	Length *uint `bson:"length,omitempty"`
	Offset uint  `bson:"offset"`
}

type diagnosticAggregateInfoDocument struct {
	SampleFeatureIDs []string `bson:"sampleFeatureIds"`
	Count            uint64   `bson:"count"`
}

// DiagnosticDocument is a single per-node (or per-job, when NodeID is the
// JobDiagnosticNodeSegment sentinel) diagnostic row in the nodeDiagnostics
// collection. It is used both for the subscriber's append-only live
// diagnostic rows (read-only from this module's perspective) and for the
// terminal failed-node/aggregated rows this module itself writes at
// job-completion merge time (see NewFailedNodeDocument /
// NewAggregatedDiagnosticDocument); the latter use a deterministic ID
// instead of a random ObjectID suffix so JobCompleteEvent redeliveries
// upsert the same row.
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
// {jobId}:{nodeId}:... document ID and in the nodeId bson field itself
// (mirrors the subscriber's mongodoc.JobDiagnosticNodeSegment; keep the two
// in lockstep). Model() below strips the sentinel back to nil so it never
// leaks past the domain layer into GraphQL's Diagnostic.nodeId.
const JobDiagnosticNodeSegment = "_job"

// normalizedNodeSegment returns nodeID's value, or JobDiagnosticNodeSegment
// when nodeID is nil or the empty string.
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

	// The nodeId bson field carries the JobDiagnosticNodeSegment sentinel for
	// job-level rows (see normalizedNodeSegment); translate it back to nil
	// here so the domain/GraphQL layer keeps its existing nil-means-job-level
	// semantics instead of leaking the storage convention.
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
		Help(d.Help)

	if d.Aggregated != nil {
		b = b.Aggregated(diagnostic.NewAggregateInfo(d.Aggregated.Count, d.Aggregated.SampleFeatureIDs))
	}
	if d.SourceSpan != nil {
		b = b.SourceSpan(diagnostic.NewSourceSpan(d.SourceSpan.Offset, d.SourceSpan.Length))
	}

	return b.Build()
}

// jobCompleteDiagnosticSchemaTag marks DiagnosticDocument rows written by
// NewFailedNodeDocument / NewAggregatedDiagnosticDocument (job-completion
// merge persistence) as distinct from the subscriber's live "diagnostic.v1"
// event rows, for provenance when inspecting the collection directly.
const jobCompleteDiagnosticSchemaTag = "job-complete.v1"

// NewFailedNodeDocument builds a terminal-diagnostic row for one
// JobCompleteEvent failed node, persisted at job-completion merge time
// (interactor/job.go, before the source event is deleted from Redis). The ID
// is deterministic ({jobId}:{nodeId-or-_job}:failed:{code}) rather than a
// random ObjectID suffix: JobCompleteEvent redeliveries (retry-after-
// persist-failure) must upsert the SAME row instead of appending duplicates.
func NewFailedNodeDocument(jobID id.JobID, d *diagnostic.Diagnostic) DiagnosticDocument {
	return newTerminalDiagnosticDocument(jobID, d, "failed")
}

// NewAggregatedDiagnosticDocument builds a terminal-diagnostic row for one
// JobCompleteEvent aggregatedDiagnostics entry, persisted at job-completion
// merge time. Like failed-node rows, each aggregated diagnostic gets its own
// row rather than being nested inside the per-job summary row, so it stays
// discoverable through FindByJobNodeID for the node it pertains to. The ID
// is deterministic ({jobId}:{nodeId-or-_job}:aggregated:{code}), idempotent
// across JobCompleteEvent redeliveries.
func NewAggregatedDiagnosticDocument(jobID id.JobID, d *diagnostic.Diagnostic) DiagnosticDocument {
	return newTerminalDiagnosticDocument(jobID, d, "aggregated")
}

func newTerminalDiagnosticDocument(jobID id.JobID, d *diagnostic.Diagnostic, kind string) DiagnosticDocument {
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

// JobDiagnosticsSummaryDocument is the single per-job row persisted at
// job-completion merge time (interactor/job.go) for the JobCompleteEvent's
// droppedEventCount. It deliberately does NOT reuse DiagnosticDocument's
// per-event shape (no top-level code/category/severity/message field): this
// is a job-level counter, not an individual diagnostic, and the
// nodeDiagnostics read path (FindByJobNodeID/FindByJobID in
// ../diagnostic.go) filters on {"code": {"$exists": true}} specifically to
// exclude this row from the generic per-diagnostic decode path
// (DiagnosticDocument.Model() would otherwise silently decode it into a
// mostly-empty Diagnostic). AggregatedDiagnostics entries are NOT nested
// here: each gets its own row via NewAggregatedDiagnosticDocument, so they
// stay visible through FindByJobNodeID for the node they pertain to. Read via
// its deterministic ID by ../diagnostic.go's FindJobSummary, backing the
// GraphQL Job.droppedEventCount resolver.
type JobDiagnosticsSummaryDocument struct {
	Timestamp         time.Time `bson:"timestamp"`
	DroppedEventCount *uint64   `bson:"droppedEventCount,omitempty"`
	ID                string    `bson:"id"`
	JobID             string    `bson:"jobId"`
}

// JobDiagnosticsSummaryID is the deterministic ID of the single per-job
// summary row (see JobDiagnosticsSummaryDocument), exported so callers can
// look the row up directly (upsert or read, see ../diagnostic.go's
// FindJobSummary) without recomputing the convention.
func JobDiagnosticsSummaryID(jobID id.JobID) string {
	return jobID.String() + ":" + JobDiagnosticNodeSegment + ":summary"
}

// NewJobDiagnosticsSummaryDocument builds the single per-job summary row
// from a JobCompleteEvent's droppedEventCount.
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

// Model implements mongodoc.Model, letting JobDiagnosticsSummaryDocument
// plug into the same Consumer machinery as DiagnosticDocument. The "model"
// here is just the droppedEventCount pointer, not a domain type: this row
// has no other GraphQL-visible shape (see JobDiagnosticsSummaryDocument's
// own doc comment). SaveTerminalDiagnostics only ever writes this row when
// droppedEventCount is non-nil, so a decoded row's count is never nil.
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
