//! Minimal, hand-rolled `tileset.json`: one root tile carrying all of the
//! dataset's geometry directly.
//!
//! No implicit tiling, no `.subtree` files, no quadtree subdivision. The
//! containment-placement tiling redesign (§6.2 of the geometry design doc) is
//! a deliberately separate, later pass — this proves the geometry → glTF →
//! tileset pipeline in isolation first, on a dataset small enough to need no
//! subdivision at all.

use serde_json::{json, Value};

/// Build `tileset.json` for one root tile whose content is `content_uri`,
/// covering the WGS84 geographic extent of `geographic_vertices` (lon/lat in
/// degrees, height in metres — the convention the geometry crate's `Reproject`
/// op already normalizes to).
///
/// `geometricError` is a modest, non-zero placeholder at both the tileset and
/// root-tile level. A real writer would derive this from the tile's on-screen
/// error at some reference distance; this pass has no multi-LOD refinement to
/// calibrate against, so a fixed placeholder stands in — but it must not be
/// `0`, which known-good tilesets in `testing/data/results` never use even on
/// leaf content tiles, and which risks confusing a renderer's screen-space-
/// error-based tile selection (bounding-volume-only checks, like flying the
/// camera to a tile, don't need this — but deciding whether to fetch and
/// render a tile's content does).
const PLACEHOLDER_GEOMETRIC_ERROR: f64 = 100.0;

pub(super) fn build(geographic_vertices: &[[f64; 3]], content_uri: &str) -> Value {
    let (min, max) = bounds(geographic_vertices);
    let region = [
        min[0].to_radians(), // west
        min[1].to_radians(), // south
        max[0].to_radians(), // east
        max[1].to_radians(), // north
        min[2],              // minimum height (metres)
        max[2],              // maximum height (metres)
    ];

    json!({
        "asset": {"version": "1.1"},
        "geometricError": PLACEHOLDER_GEOMETRIC_ERROR,
        "root": {
            "boundingVolume": {"region": region},
            "geometricError": PLACEHOLDER_GEOMETRIC_ERROR,
            "refine": "REPLACE",
            "content": {"uri": content_uri},
        },
    })
}

fn bounds(points: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for p in points {
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    if points.is_empty() {
        // No renderable geometry: an empty, non-degenerate region rather than
        // MAX/MIN leaking into the JSON.
        return ([0.0; 3], [0.0; 3]);
    }
    (min, max)
}
