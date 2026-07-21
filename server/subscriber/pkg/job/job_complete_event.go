package job

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

// JobCompleteEvent is the pub/sub wire shape published by the engine on job
// completion. FailedNodes/AggregatedDiagnostics/DroppedEventCount are nil
// when the publishing engine predates diagnostics. FailedNodes absent vs.
// empty is meaningful (don't infer job failure from either), and fatality
// must be read from EffectiveDisposition, never Severity.
type JobCompleteEvent struct {
	Timestamp             time.Time                   `json:"timestamp"`
	DroppedEventCount     *uint64                     `json:"droppedEventCount,omitempty"`
	WorkflowID            string                      `json:"workflowId"`
	JobID                 string                      `json:"jobId"`
	Result                string                      `json:"result"`
	FailedNodes           []diagnostic.WireDiagnostic `json:"failedNodes,omitempty"`
	AggregatedDiagnostics []diagnostic.WireDiagnostic `json:"aggregatedDiagnostics,omitempty"`
}
