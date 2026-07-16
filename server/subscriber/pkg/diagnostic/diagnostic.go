package diagnostic

import (
	"errors"
	"time"
)

// DiagnosticSchemaV1 is the wire schema tag for a DiagnosticEvent
// (engine/schema/diagnostic_event.json). DiagnosticEvent.Schema must equal
// this value.
const DiagnosticSchemaV1 = "diagnostic.v1"

// ErrInvalidDiagnosticEvent is returned by NewDiagnosticEvent when the
// wire schema tag or jobId is missing/invalid.
var ErrInvalidDiagnosticEvent = errors.New("invalid diagnostic event data")

// DiagnosticEvent is the pub/sub wire event for a single structured
// Diagnostic published by the engine (engine/schema/diagnostic_event.json).
// On the wire the WireDiagnostic fields (code/category/severity/...) are
// top-level siblings of schema/workflowId/jobId/timestamp, so
// WireDiagnostic is embedded (not nested) to match that flattened shape
// exactly on both Marshal and Unmarshal.
type DiagnosticEvent struct {
	Timestamp  time.Time `json:"timestamp"`
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	Schema     string    `json:"schema"`
	WireDiagnostic
}

// NewDiagnosticEvent constructs a DiagnosticEvent, validating the wire
// schema tag and required jobId.
//
// Category, Severity and EffectiveDisposition (carried inside wire) are
// NOT validated against a known set here or anywhere downstream: the
// engine may introduce new values over time and unknown strings must
// survive a round trip verbatim (forward-compat requirement, spec
// diagnostic.v1 4.7).
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
