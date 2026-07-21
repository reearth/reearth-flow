package diagnostic

import (
	"errors"
	"time"
)

// DiagnosticSchemaV1 is the wire schema tag a DiagnosticEvent's Schema
// field must equal (engine/schema/diagnostic_event.json).
const DiagnosticSchemaV1 = "diagnostic.v1"

// ErrInvalidDiagnosticEvent is returned by NewDiagnosticEvent when the
// wire schema tag or jobId is missing/invalid.
var ErrInvalidDiagnosticEvent = errors.New("invalid diagnostic event data")

// DiagnosticEvent is the pub/sub wire event for a single structured
// Diagnostic (engine/schema/diagnostic_event.json). WireDiagnostic is
// embedded, not nested, so its fields sit flat alongside
// schema/workflowId/jobId/timestamp on the wire.
type DiagnosticEvent struct {
	Timestamp  time.Time `json:"timestamp"`
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	Schema     string    `json:"schema"`
	WireDiagnostic
}

// NewDiagnosticEvent constructs a DiagnosticEvent, validating the wire
// schema tag and required jobId. Category/Severity/EffectiveDisposition
// are not validated here or downstream — unknown values must round-trip
// verbatim (see WireDiagnostic).
func NewDiagnosticEvent(
	schema string,
	workflowID string,
	jobID string,
	timestamp time.Time,
	wire WireDiagnostic,
) (*DiagnosticEvent, error) {
	if schema != DiagnosticSchemaV1 || jobID == "" {
		return nil, ErrInvalidDiagnosticEvent
	}

	return &DiagnosticEvent{
		Schema:         schema,
		WorkflowID:     workflowID,
		JobID:          jobID,
		Timestamp:      timestamp,
		WireDiagnostic: wire,
	}, nil
}
