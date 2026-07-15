//! Conversions between the flattened leaf views and `i_overlay`'s shape model.
//!
//! `i_overlay` works over `Shapes` — a list of shapes, each a list of contours
//! (paths), each an *implicitly closed* list of `[f64; 2]` — and `Paths` for
//! open polylines. The direct template is the legacy
//! `algorithm/bool_ops/ioverlap_bridge.rs`, minus its `FlowCoord` newtype:
//! the new leaves' `[f64; 2]` layout is `FloatPointCompatible` as-is, so both
//! directions are plain buffer reshuffles.
//!
//! Mesh leaves do not enter face by face: their faces dissolve to
//! union-boundary rings first (see
//! [`boundary`](crate::predicates::relate::boundary)), which cancels shared
//! internal edges exactly — before `i_overlay`'s snap to its integer grid —
//! and preserves the interior-left direction of every surviving ring.

use crate::coordinate::CoordinateFrame;
use crate::line_string::LineString2D;
use crate::polygon::Polygon2D;
use crate::predicates::relate::boundary::union_boundary_rings;
use crate::predicates::view::{polygon2d_rings, Leaf2D, RingView};

use super::leaf_type_name;

/// One implicitly closed contour (ring) or open polyline.
pub(super) type Path = Vec<[f64; 2]>;
/// One areal shape: its outer contour and hole contours.
pub(super) type Shape = Vec<Path>;

/// Convert areal leaves into `i_overlay` shapes: a polygon contributes its
/// rings verbatim, a mesh its union-boundary rings. Empty leaves contribute
/// nothing. Errs with the leaf's type name on the first non-areal leaf.
pub(super) fn areal_shapes(leaves: &[Leaf2D<'_>]) -> Result<Vec<Shape>, &'static str> {
    let mut shapes = Vec::new();
    for leaf in leaves {
        let shape: Shape = match leaf {
            Leaf2D::Polygon(p) => polygon2d_rings(p)
                .map(|ring| ring_to_path(RingView::Slice(ring)))
                .filter(|path| !path.is_empty())
                .collect(),
            Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => {
                let area = leaf.area_view().expect("mesh leaves are areal");
                union_boundary_rings(&area)
                    .into_iter()
                    .map(|ring| ring_to_path(RingView::Slice(&ring)))
                    .filter(|path| !path.is_empty())
                    .collect()
            }
            Leaf2D::Point(_) | Leaf2D::Line(_) => return Err(leaf_type_name(leaf)),
        };
        if !shape.is_empty() {
            shapes.push(shape);
        }
    }
    Ok(shapes)
}

/// A ring as an implicitly closed `i_overlay` path: the stored vertices with
/// the closing duplicate (if stored) dropped.
fn ring_to_path(ring: RingView<'_>) -> Path {
    (0..ring.open_len()).map(|i| ring.coord(i)).collect()
}

/// Convert line leaves into open `i_overlay` paths, one per polyline. Empty
/// polylines contribute nothing. Errs with the leaf's type name on the first
/// non-line leaf.
pub(super) fn line_paths(leaves: &[Leaf2D<'_>]) -> Result<Vec<Path>, &'static str> {
    let mut paths = Vec::new();
    for leaf in leaves {
        match leaf {
            Leaf2D::Line(l) => {
                if !l.coords().is_empty() {
                    paths.push(l.coords().to_vec());
                }
            }
            _ => return Err(leaf_type_name(leaf)),
        }
    }
    Ok(paths)
}

/// Convert `i_overlay` result shapes back into polygons in `frame`, closing
/// each ring. The backend emits the outer contour first (CCW) and holes after
/// (CW) — Flow's winding convention — so rings pass through verbatim.
pub(super) fn shapes_to_polygons(shapes: Vec<Shape>, frame: &CoordinateFrame) -> Vec<Polygon2D> {
    shapes
        .into_iter()
        .filter_map(|shape| {
            let mut rings = shape.into_iter().map(close_path);
            let exterior = rings.next()?;
            Some(Polygon2D::from_rings(frame.clone(), exterior, rings))
        })
        .collect()
}

/// Convert `i_overlay` result paths back into polylines in `frame`.
pub(super) fn paths_to_line_strings(
    paths: Vec<Path>,
    frame: &CoordinateFrame,
) -> Vec<LineString2D> {
    paths
        .into_iter()
        .map(|path| LineString2D::from_coords(frame.clone(), path))
        .collect()
}

/// Close an implicitly closed path by appending its first vertex.
fn close_path(mut path: Path) -> Path {
    if let Some(&first) = path.first() {
        path.push(first);
    }
    path
}
