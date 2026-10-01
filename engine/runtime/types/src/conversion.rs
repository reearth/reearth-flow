// The GeoJSON <-> Feature conversion depends on the feature geometry type, so it
// splits by world: `geojson.rs` (old) vs `geojson_next.rs` (new-geometry). What the
// two have in common is re-exported from both, so callers see one path.
mod geojson_shared;

#[cfg(not(feature = "new-geometry"))]
pub mod geojson;
#[cfg(feature = "new-geometry")]
#[path = "conversion/geojson_next.rs"]
pub mod geojson;
pub mod nusamai;

// What a write's coordinates are expressed in is not a GeoJSON question — the
// CityGML writer folds the same lattice to decide its `srsName` — so it is named
// here rather than reached for through `conversion::geojson`.
pub use geojson_shared::CrsCoverage;
