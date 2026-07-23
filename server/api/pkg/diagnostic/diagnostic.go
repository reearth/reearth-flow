package diagnostic

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

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

// NodeID/Category/Severity/EffectiveDisposition are plain strings, not
// id.NodeID or enums: engine node IDs aren't always UUIDs, and unknown
// engine values must round-trip verbatim.
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
	// terminal is internal-only, never wire/GraphQL-exposed.
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
