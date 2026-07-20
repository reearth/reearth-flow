package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
)

// ToDiagnostic converts a domain Diagnostic into its GraphQL representation.
// category/severity/effectiveDisposition pass through as plain strings
// verbatim (see gql/diagnostic.graphql's doc comment on the Diagnostic
// type) — no enum mapping/validation here, by design.
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

// ToDiagnostics converts a slice of domain Diagnostics, skipping nil entries.
func ToDiagnostics(ds []*diagnostic.Diagnostic) []*Diagnostic {
	if ds == nil {
		return nil
	}

	res := make([]*Diagnostic, 0, len(ds))
	for _, d := range ds {
		if converted := ToDiagnostic(d); converted != nil {
			res = append(res, converted)
		}
	}
	return res
}
