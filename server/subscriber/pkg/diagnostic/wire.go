// Package diagnostic holds the wire form of a structured engine Diagnostic,
// shared (by duplication, not by import — subscriber and api are independent
// Go modules) with the api-side equivalent in
// internal/usecase/gateway/diagnostic.go.
package diagnostic

// WireSourceSpan is the wire mirror of reearth_flow_diagnostics::SourceSpan.
type WireSourceSpan struct {
	Length *uint `json:"length,omitempty"`
	Offset uint  `json:"offset"`
}

// WireAggregateInfo is the wire mirror of reearth_flow_diagnostics::AggregateInfo.
type WireAggregateInfo struct {
	SampleFeatureIds []string `json:"sampleFeatureIds"`
	Count            uint64   `json:"count"`
}

// WireDiagnostic is the wire form of a structured Diagnostic published by the
// engine (see engine/schema/diagnostic_event.json and
// engine/schema/job_complete_event.json, generated from the Rust
// reearth_flow_diagnostics types). It is used both standalone (embedded in a
// future DiagnosticEvent) and nested inside JobCompleteEvent's failedNodes /
// aggregatedDiagnostics arrays.
//
// Category, Severity and EffectiveDisposition are carried as plain strings
// rather than a closed Go enum: the engine renders them as their snake_case
// string values specifically so unknown or newer values survive a Go round
// trip verbatim instead of failing to deserialize (forward-compat
// requirement, spec diagnostic.v1 4.7). Do NOT validate these strings
// against a known set.
type WireDiagnostic struct {
	Aggregated           *WireAggregateInfo `json:"aggregated,omitempty"`
	SourceSpan           *WireSourceSpan    `json:"sourceSpan,omitempty"`
	EffectiveDisposition *string            `json:"effectiveDisposition,omitempty"`
	NodeID               *string            `json:"nodeId,omitempty"`
	ActionType           *string            `json:"actionType,omitempty"`
	FeatureID            *string            `json:"featureId,omitempty"`
	Help                 *string            `json:"help,omitempty"`
	Code                 string             `json:"code"`
	Category             string             `json:"category"`
	Severity             string             `json:"severity"`
	Message              string             `json:"message"`
}
