package job

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

// JobCompleteEvent is the pub/sub wire shape published by the engine on job
// completion (engine/schema/job_complete_event.json). FailedNodes,
// AggregatedDiagnostics and DroppedEventCount are optional (nil when the
// publishing engine predates diagnostics, or when the runner returned Err
// before a RunSummary existed).
//
// Consumer contract (mirrors the doc comment on the Rust struct):
//   - FailedNodes absent (nil) means no RunSummary was produced at all;
//     present-but-empty means a summary was produced with no structured
//     failures. Do NOT infer job failure from FailedNodes being empty or
//     absent — Result == "failed" with no FailedNodes remains common.
//   - A FailedNodes entry is not always the original per-feature
//     diagnostic: cascade failures are synthesized under code
//     "internal.unclassified".
//   - NodeID on any WireDiagnostic may be a composed id containing
//     subgraph-prefix dots, a comma-joined list of several composed ids, or
//     the fixed string "replay-injector" — never assume it is a bare UUID.
//   - Whether an entry is fatal must be read from EffectiveDisposition, not
//     Severity: Severity is the code's registry-authored default and is
//     never rewritten when a disposition policy promotes it to fatal.
//   - AggregatedDiagnostics never contains a fatal entry; same
//     absent-vs-empty contract as FailedNodes.
type JobCompleteEvent struct {
	Timestamp             time.Time                   `json:"timestamp"`
	DroppedEventCount     *uint64                     `json:"droppedEventCount,omitempty"`
	WorkflowID            string                      `json:"workflowId"`
	JobID                 string                      `json:"jobId"`
	Result                string                      `json:"result"`
	FailedNodes           []diagnostic.WireDiagnostic `json:"failedNodes,omitempty"`
	AggregatedDiagnostics []diagnostic.WireDiagnostic `json:"aggregatedDiagnostics,omitempty"`
}
