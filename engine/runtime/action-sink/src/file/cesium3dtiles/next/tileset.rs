//! `tileset.json` for the containment-placement quadtree: one explicit root
//! tile declaring 3D Tiles 1.1 implicit tiling. Every descendant tile's
//! bounding volume and geometric error are derived by the client from
//! `level` alone, so nothing per-tile is written here beyond the root.
//!
//! Which cells actually hold content lives in the paired `.subtree` file
//! (`subtree.rs`), not in this JSON.

use serde_json::{json, Value};

use super::quadtree::{geometric_error, root_ground_diagonal_m, GeoBox};

const CONTENT_URI_TEMPLATE: &str = "content/{level}/{x}/{y}.glb";
const SUBTREES_URI_TEMPLATE: &str = "subtrees/{level}.{x}.{y}.subtree";

pub(super) fn build(root: &GeoBox, subtree_levels: u32) -> Value {
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
                "subtreeLevels": subtree_levels,
                "availableLevels": subtree_levels,
                "subtrees": {"uri": SUBTREES_URI_TEMPLATE},
            },
        },
    })
}
