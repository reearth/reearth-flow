package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
)

// Enum-like fields stay strings so unknown engine values survive the round trip.
func ToDiagnostic(d *diagnostic.Diagnostic) *Diagnostic {
	if d == nil {
		return nil
	}

	res := &Diagnostic{
		Code:                 d.Code(),
		Category:             d.Category(),
		Severity:             d.Severity(),
		EffectiveDisposition: d.EffectiveDisposition(),
		NodeID:               d.NodeID(),
		ActionType:           d.ActionType(),
		Message:              d.Message(),
		Help:                 d.Help(),
	}

	if featureID := d.FeatureID(); featureID != nil {
		fid := ID(*featureID)
		res.FeatureID = &fid
	}

	if agg := d.Aggregated(); agg != nil {
		count := int(agg.Count())
		res.AggregatedCount = &count
		if sampleIDs := agg.SampleFeatureIDs(); len(sampleIDs) > 0 {
			res.SampleFeatureIds = make([]ID, len(sampleIDs))
			for i, s := range sampleIDs {
				res.SampleFeatureIds[i] = ID(s)
			}
		}
	}

	return res
}

// Never nil: a nil slice would marshal as GraphQL null rather than [].
func ToDiagnostics(ds []*diagnostic.Diagnostic) []*Diagnostic {
	res := make([]*Diagnostic, 0, len(ds))
	for _, d := range ds {
		if converted := ToDiagnostic(d); converted != nil {
			res = append(res, converted)
		}
	}
	return res
}
