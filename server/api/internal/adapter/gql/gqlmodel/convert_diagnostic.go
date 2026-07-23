package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
)

// category/severity/effectiveDisposition pass through as plain strings, no
// enum mapping — by design, so unknown values survive.
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

// Always returns non-nil (even for empty input): a nil slice would marshal
// as GraphQL null instead of [] for this [Diagnostic!] field.
func ToDiagnostics(ds []*diagnostic.Diagnostic) []*Diagnostic {
	res := make([]*Diagnostic, 0, len(ds))
	for _, d := range ds {
		if converted := ToDiagnostic(d); converted != nil {
			res = append(res, converted)
		}
	}
	return res
}
