// Package diagnostic holds the wire form of a structured engine Diagnostic,
// duplicated in lockstep (not imported — separate Go modules) with the
// api-side equivalent in internal/usecase/gateway/diagnostic.go.
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

// WireDiagnostic is the wire form of a structured Diagnostic published by
// the engine (engine/schema/diagnostic_event.json), used standalone and
// nested inside JobCompleteEvent's failedNodes/aggregatedDiagnostics.
// Category, Severity and EffectiveDisposition are plain strings, not a
// closed enum, so unknown/newer values round-trip verbatim — do not
// validate them against a known set.
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
