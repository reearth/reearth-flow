use serde_json::{json, Value};

use super::quadtree::{geometric_error, root_ground_diagonal_m, GeoBox};

const CONTENT_URI_TEMPLATE: &str = "content/{level}/{x}/{y}.glb";
const SUBTREES_URI_TEMPLATE: &str = "subtrees/{level}.{x}.{y}.subtree";

/// One explicit root tile declaring 3D Tiles 1.1 implicit tiling;
/// descendants' bounding volume/geometric error are client-derived from
/// `level` alone. Which cells hold content lives in the paired `.subtree`
/// file(s) (`subtree.rs`), not here.
pub(super) fn build(root: &GeoBox, available_levels: u32) -> Value {
    let region = [
        root.west.to_radians(),
        root.south.to_radians(),
        root.east.to_radians(),
        root.north.to_radians(),
        root.min_height,
        root.max_height,
    ];
    let root_error = geometric_error(root_ground_diagonal_m(root), 0);

    json!({
        "asset": {"version": "1.1"},
        "geometricError": root_error,
        "root": {
            "boundingVolume": {"region": region},
            "geometricError": root_error,
            "refine": "ADD",
            "content": {"uri": CONTENT_URI_TEMPLATE},
            "implicitTiling": {
                "subdivisionScheme": "QUADTREE",
                "subtreeLevels": super::subtree::SUBTREE_LEVELS,
                "availableLevels": available_levels,
                "subtrees": {"uri": SUBTREES_URI_TEMPLATE},
            },
        },
    })
}
