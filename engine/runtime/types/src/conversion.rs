// The GeoJSON <-> Feature conversion depends on the feature geometry type, so it
// splits by world: `geojson.rs` (old) vs `geojson_next.rs` (new-geometry).
#[cfg(not(feature = "new-geometry"))]
pub mod geojson;
#[cfg(feature = "new-geometry")]
#[path = "conversion/geojson_next.rs"]
pub mod geojson;
pub mod nusamai;
