package gateway

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

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

func (w WireDiagnostic) ToDomain(jobID id.JobID, timestamp time.Time) (*diagnostic.Diagnostic, error) {
	b := diagnostic.NewBuilder().
		JobID(jobID).
		Timestamp(timestamp).
		Code(w.Code).
		Category(w.Category).
		Severity(w.Severity).
		EffectiveDisposition(w.EffectiveDisposition).
		NodeID(w.NodeID).
		ActionType(w.ActionType).
		FeatureID(w.FeatureID).
		Message(w.Message).
		Help(w.Help)

	if w.Aggregated != nil {
		b = b.Aggregated(diagnostic.NewAggregateInfo(w.Aggregated.Count, w.Aggregated.SampleFeatureIds))
	}
	if w.SourceSpan != nil {
		b = b.SourceSpan(diagnostic.NewSourceSpan(w.SourceSpan.Offset, w.SourceSpan.Length))
	}

	return b.Build()
}
