package app

// baseActions is the curated set of actions exposed in the Re:Earth Flow SaaS UI palette.
// All other actions in the schema are available for workflow execution but hidden from the UI.
//
// An action is listed here only if it BOTH runs in the shipped build (action-standard.md §7.1)
// and has passed an engine-side review against that standard. Four reasons an action is absent,
// each with the trigger that would bring it back. The full inventory, with preliminary findings
// per action, is engine/dev-docs/action-review-findings.md.
//
//  1. Does not run in the shipped build. Its process/start exists only under
//     #[cfg(not(feature = "new-geometry"))], so it hits the trait default and errors on every
//     feature. Re-add as each port lands — tracked in Notion FLOW-DEV-182 "GEOM: Actions
//     Migration".
//
//  2. Pending audit. Runs, but has never had an engine-side pass against the action standard,
//     so its metadata is in whatever state it was authored in. Re-add per action as it is
//     audited.
//
//  3. Flagged for removal. Horizontal Reprojector and Vertical Reprojector are absent
//     permanently: the tracker marks both "To Be Removed", and their new-geometry impls are
//     stubs that error with a pointer to Coordinate Frame Reprojector, which is exposed below
//     and supersedes both. They owe an engine-side deletion, not a re-exposure.
//
//  4. Retired on design grounds, both unused in any workflow:
//     Attribute File Path Info Extractor duplicates File Property Extractor exactly (same five
//     output attributes, same values); the File one is documented and stays.
//     Attribute Bulk Array Joiner joins every array attribute with a hard-coded comma and an
//     opt-out list, and silently drops non-scalar elements. Its purpose looks to be CityGML
//     attribute flattening; it needs a decision on scope before it is offered.
var baseActions = map[string]bool{
	// Input
	"Feature Creator":     true,
	"File Path Extractor": true,
	"GeoJSON Reader":      true,
	"GeoPackage Reader":   true,
	"glTF Reader":         true,
	"JSON Reader":         true,
	"OBJ Reader":          true,
	"Shapefile Reader":    true,
	"SQL Reader":          true,
	// Output
	"Cesium 3D Tiles Writer": true,
	"CSV Writer":             true,
	"Echo Sink":              true,
	"GeoJSON Writer":         true,
	"JSON Writer":            true,
	"MVT Writer":             true,
	"Noop Sink":              true,
	"Shapefile Writer":       true,
	"XML Writer":             true,
	"Zip File Writer":        true,
	// Geometry
	"Appearance Remover":           true,
	"Bounds Extractor":             true,
	"Coordinate Frame Reprojector": true,
	"Dissolver":                    true,
	"Footprint Replacer":           true,
	"Geometry Extractor":           true,
	"Geometry Remover":             true,
	"Geometry Replacer":            true,
	"Geometry Splitter":            true,
	"Geometry Validator":           true,
	"Two Dimension Forcer":         true,
	// Attribute
	"Attribute Aggregator":       true,
	"Attribute Conversion Table": true,
	"Attribute Flattener":        true,
	"Attribute Manager":          true,
	"Attribute Mapper":           true,
	"Attribute Range Mapper":     true,
	"Attribute Table Extractor":  true,
	"Bulk Attribute Renamer":     true,
	"Date Time Converter":        true,
	"Null Attribute Mapper":      true,
	"Statistics Calculator":      true,
	// Feature / Flow
	"Feature CityGML 2 Reader":    true,
	"Feature Counter":             true,
	"Feature File Path Extractor": true,
	"Feature Filter":              true,
	"Feature Joiner":              true,
	"Feature Merger":              true,
	"Feature Sorter":              true,
	"Feature Transformer":         true,
	"Feature Type Filter":         true,
	"Input Router":                true,
	"Output Router":               true,
	// Utility
	"Directory Decompressor":  true,
	"Echo Processor":          true,
	"File Property Extractor": true,
	"List Exploder":           true,
	"Noop Processor":          true,
	"XML Fragmenter":          true,
	"XML Validator":           true,
}
