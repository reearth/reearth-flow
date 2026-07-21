// Package diagnostic holds the domain representation of a structured engine
// Diagnostic, read either from the live per-event stream the subscriber
// persists (Mongo/Redis) or from a terminal snapshot persisted at job
// completion (interactor/job.go's status merge).
package diagnostic

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

// SourceSpan mirrors the wire SourceSpan: an optional byte offset+length
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
// NodeID is a plain string, not id.NodeID: the engine may emit composed
// node identities (e.g. "subgraph-a.sink-writer-2") that aren't valid
// UUIDs. Category/Severity/EffectiveDisposition stay plain strings too, so
// unknown/newer engine values survive a round trip verbatim.
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
	// terminal is true when persisted at job-completion merge time rather
	// than mirrored live from the subscriber's event stream. Internal
	// read-path signal only — never wire/GraphQL-exposed — used by
	// interactor/diagnostic.go's dedupeDiagnostics.
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
