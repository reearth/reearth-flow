package job

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

type JobCompleteEvent struct {
	Timestamp             time.Time                   `json:"timestamp"`
	DroppedEventCount     *uint64                     `json:"droppedEventCount,omitempty"`
	WorkflowID            string                      `json:"workflowId"`
	JobID                 string                      `json:"jobId"`
	Result                string                      `json:"result"`
	FailedNodes           []diagnostic.WireDiagnostic `json:"failedNodes,omitempty"`
	AggregatedDiagnostics []diagnostic.WireDiagnostic `json:"aggregatedDiagnostics,omitempty"`
}
