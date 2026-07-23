package diagnostic

import (
	"errors"
	"time"
)

const DiagnosticSchemaV1 = "diagnostic.v1"

var ErrInvalidDiagnosticEvent = errors.New("invalid diagnostic event data")

type DiagnosticEvent struct {
	Timestamp  time.Time `json:"timestamp"`
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	Schema     string    `json:"schema"`
	WireDiagnostic
}

// Only schema and jobId are validated; Category/Severity/EffectiveDisposition round-trip unvalidated.
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
