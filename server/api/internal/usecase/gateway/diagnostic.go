package gateway

type WireSourceSpan struct {
	Length *uint `json:"length,omitempty"`
	Offset uint  `json:"offset"`
}

type WireAggregateInfo struct {
	SampleFeatureIds []string `json:"sampleFeatureIds"`
	Count            uint64   `json:"count"`
}

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
