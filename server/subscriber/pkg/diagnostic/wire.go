// Package diagnostic is duplicated in lockstep (not imported — separate Go
// modules) with the api-side equivalent in internal/usecase/gateway/diagnostic.go.
package diagnostic

type WireSourceSpan struct {
	Length *uint `json:"length,omitempty"`
	Offset uint  `json:"offset"`
}

type WireAggregateInfo struct {
	SampleFeatureIds []string `json:"sampleFeatureIds"`
	Count            uint64   `json:"count"`
}

// Category/Severity/EffectiveDisposition are unvalidated strings — unknown values round-trip verbatim.
// Fatality is EffectiveDisposition only, never Severity.
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
