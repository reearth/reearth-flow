use serde_json::{json, Value};

use super::quadtree::{geometric_error, root_ground_diagonal_m, GeoBox};

const CONTENT_URI_TEMPLATE: &str = "content/{level}/{x}/{y}.glb";
const SUBTREES_URI_TEMPLATE: &str = "subtrees/{level}.{x}.{y}.subtree";

/// One explicit root tile declaring 3D Tiles 1.1 implicit tiling;
/// descendants' bounding volume/geometric error are client-derived from
/// `level` alone. Which cells hold content lives in the paired `.subtree`
/// file(s) (`subtree.rs`), not here.
///
/// `max_contents` is the dataset-wide maximum same-tile content count (see
/// `mod.rs`'s same-tile splitting): 1 keeps the plain single-`content` form,
/// >1 switches every cell to a `contents` array so the array's positions line
/// up with each `.subtree` file's `contentAvailability` entries.
pub(super) fn build(root: &GeoBox, available_levels: u32, max_contents: usize) -> Value {
    let region = [
        root.west.to_radians(),
        root.south.to_radians(),
        root.east.to_radians(),
        root.north.to_radians(),
        root.min_height,
        root.max_height,
    ];
    let root_error = geometric_error(root_ground_diagonal_m(root), 0);

    let mut root_tile = serde_json::Map::new();
    root_tile.insert("boundingVolume".into(), json!({"region": region}));
    root_tile.insert("geometricError".into(), json!(root_error));
    root_tile.insert("refine".into(), json!("ADD"));
    if max_contents <= 1 {
        root_tile.insert("content".into(), json!({"uri": CONTENT_URI_TEMPLATE}));
    } else {
        let contents: Vec<Value> = (0..max_contents)
            .map(|n| json!({"uri": format!("content/{{level}}/{{x}}/{{y}}_{n}.glb")}))
            .collect();
        root_tile.insert("contents".into(), Value::Array(contents));
    }
    root_tile.insert(
        "implicitTiling".into(),
        json!({
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": super::subtree::SUBTREE_LEVELS,
            "availableLevels": available_levels,
            "subtrees": {"uri": SUBTREES_URI_TEMPLATE},
        }),
    );

    json!({
        "asset": {"version": "1.1"},
        "geometricError": root_error,
        "root": Value::Object(root_tile),
    })
}
