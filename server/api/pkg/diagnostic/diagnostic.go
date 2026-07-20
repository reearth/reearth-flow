// Package diagnostic holds the domain representation of a structured engine
// Diagnostic, read either from the live per-event stream the subscriber
// persists (nodeDiagnostics Mongo collection / diagnostics:{jobId}[:{nodeId}]
// Redis lists — see server/subscriber's diagnostic ingestion) or from a
// terminal snapshot persisted at job completion (interactor/job.go's status
// merge).
package diagnostic

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

// SourceSpan mirrors the wire SourceSpan (gateway.WireSourceSpan / the
// subscriber's diagnostic.WireSourceSpan): an optional byte offset+length
// into the source document a diagnostic refers to.
type SourceSpan struct {
	length *uint
	offset uint
}

func NewSourceSpan(offset uint, length *uint) *SourceSpan {
	return &SourceSpan{offset: offset, length: length}
}

func (s *SourceSpan) Offset() uint {
	return s.offset
}

func (s *SourceSpan) Length() *uint {
	return s.length
}

// AggregateInfo mirrors the wire AggregateInfo: how many times a diagnostic
// was aggregated/deduplicated, plus a sample of affected feature IDs.
type AggregateInfo struct {
	sampleFeatureIDs []string
	count            uint64
}

func NewAggregateInfo(count uint64, sampleFeatureIDs []string) *AggregateInfo {
	return &AggregateInfo{count: count, sampleFeatureIDs: sampleFeatureIDs}
}

func (a *AggregateInfo) Count() uint64 {
	return a.count
}

func (a *AggregateInfo) SampleFeatureIDs() []string {
	return a.sampleFeatureIDs
}

// Diagnostic is a single structured diagnostic row.
//
// NodeID is a plain string, NOT id.NodeID: the engine may emit composed node
// identities with dots (e.g. "subgraph-a.sink-writer-2") or a synthesized
// cascade id, neither of which is a valid UUID (see
// engine/schema/job_complete_event.json's consumer contract, commit
// 46a4bfd25). Category/Severity/EffectiveDisposition stay plain strings for
// the same forward-compat reason gateway.WireDiagnostic does: unknown/newer
// engine-emitted values must survive a round trip verbatim rather than fail
// to deserialize.
type Diagnostic struct {
	timestamp            time.Time
	featureID            *string
	sourceSpan           *SourceSpan
	effectiveDisposition *string
	nodeID               *string
	actionType           *string
	aggregated           *AggregateInfo
	help                 *string
	code                 string
	category             string
	severity             string
	message              string
	jobID                id.JobID
	// terminal is true when this row was persisted at job-completion merge
	// time (interactor/job.go's persistTerminalDiagnostics, mongodoc schema
	// "job-complete.v1") rather than mirrored live off the subscriber's
	// per-event DiagnosticEvent stream (mongodoc schema "diagnostic.v1").
	// It is an internal read-path signal only — never wire/GraphQL-exposed
	// (see gqlmodel.ToDiagnostic, which does not carry it across) — used to
	// dedupe a diagnostic that rode both paths (see
	// interactor/diagnostic.go's dedupeDiagnostics) in favor of its durable
	// terminal copy.
	terminal bool
}

func (d *Diagnostic) JobID() id.JobID {
	return d.jobID
}

func (d *Diagnostic) NodeID() *string {
	return d.nodeID
}

func (d *Diagnostic) Timestamp() time.Time {
	return d.timestamp
}

func (d *Diagnostic) Code() string {
	return d.code
}

func (d *Diagnostic) Category() string {
	return d.category
}

func (d *Diagnostic) Severity() string {
	return d.severity
}

func (d *Diagnostic) EffectiveDisposition() *string {
	return d.effectiveDisposition
}

func (d *Diagnostic) ActionType() *string {
	return d.actionType
}

func (d *Diagnostic) FeatureID() *string {
	return d.featureID
}

func (d *Diagnostic) Message() string {
	return d.message
}

func (d *Diagnostic) Help() *string {
	return d.help
}

func (d *Diagnostic) Aggregated() *AggregateInfo {
	return d.aggregated
}

func (d *Diagnostic) SourceSpan() *SourceSpan {
	return d.sourceSpan
}

func (d *Diagnostic) Terminal() bool {
	return d.terminal
}
