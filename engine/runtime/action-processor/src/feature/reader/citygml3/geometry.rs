//! Schema-agnostic CityGML 3 geometry extractor.
//!
//! Accepts an [`XmlNode`] tree (already xlink-resolved) and returns a
//! [`CityGmlGeometry`].  This module has no knowledge of xlink resolution,
//! codelist substitution, or subfeature extraction — those are earlier pipeline
//! stages.
//!
//! # Supported GML geometry elements
//!
//! | GML local name | [`GeometryType`] |
//! |---|---|
//! | `Solid` | `Solid` |
//! | `MultiSurface`, `CompositeSurface`, `Polygon` | `Surface` |
//! | `TriangulatedSurface`, `Tin` | `Triangle` |
//! | `MultiCurve`, `CompositeCurve`, `LineString`, `Curve` | `Curve` |
//! | `MultiPoint`, `Point` | `Point` |
//!
//! # LOD detection
//!
//! LOD is read from the **property element** that wraps the GML geometry, e.g.
//! `bldg:lod2MultiSurface` → LOD 2.  Any property whose local name starts with
//! `lod` followed by a single decimal digit is treated as a geometry property.
//! `tin` (no digit) maps to `GeometryType::Triangle` with `lod = None`.
//!
//! # `pos` / `len` semantics
//!
//! These fields index into the parallel `polygon_materials` / `polygon_textures`
//! / `polygon_uvs` arrays on [`CityGmlGeometry`].  Because appearance data is not
//! extracted here (deferred), those arrays are empty — but `pos` and `len` are
//! set correctly so downstream code (`filter_by_lod`, `split_feature`, …) works
//! without modification once appearance extraction is added.
//!
//! For non-polygon geometry types (Curve, Point) `pos` is set to `0` (unused).

use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::line_string::LineString3D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_types::{CityGmlGeometry, GeometryType, GmlGeometry};

use super::parser::{XmlChild, XmlNode};

// ------------------------------------------------------------------
// Public interface
// ------------------------------------------------------------------

/// Extract all geometry from a (xlink-resolved) feature node.
///
/// The returned [`CityGmlGeometry`] has empty `materials`, `textures`,
/// `polygon_materials`, `polygon_textures`, and `polygon_uvs` — appearance
/// extraction is a separate, later concern.
pub fn extract_geometry(node: &XmlNode) -> CityGmlGeometry {
    let mut gml_geometries: Vec<GmlGeometry> = Vec::new();
    let mut local_pos: u32 = 0;

    for child in element_children(node) {
        let ln = local_name(&child.name);

        // Detect geometry-bearing property elements.
        let lod = if ln == "tin" {
            Some(None) // tin has no LOD digit
        } else if let Some(lod) = extract_lod(ln) {
            Some(Some(lod))
        } else {
            None
        };

        let Some(lod_opt) = lod else { continue };

        // The geometry element is the first element child of the property.
        let Some(geom_node) = element_children(child).next() else {
            continue;
        };

        let geom_ln = local_name(&geom_node.name);
        let Some(ty) = gml_element_geometry_type(geom_ln) else {
            continue;
        };

        if let Some(gml_geom) = parse_gml_geom(geom_node, ty, lod_opt, &mut local_pos) {
            gml_geometries.push(gml_geom);
        }
    }

    CityGmlGeometry {
        gml_geometries,
        materials: Vec::new(),
        textures: Vec::new(),
        polygon_materials: Vec::new(),
        polygon_textures: Vec::new(),
        polygon_uvs: MultiPolygon2D::default(),
    }
}

// ------------------------------------------------------------------
// Per-geometry-element dispatch
// ------------------------------------------------------------------

fn parse_gml_geom(
    node: &XmlNode,
    ty: GeometryType,
    lod: Option<u8>,
    local_pos: &mut u32,
) -> Option<GmlGeometry> {
    let id = gml_id(node);
    let mut geom = GmlGeometry::new(ty, lod);
    geom.id = id;

    match ty {
        GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
            collect_polygons(node, &mut geom.polygons);
            geom.len = geom.polygons.len() as u32;
            geom.pos = *local_pos;
            *local_pos += geom.len;
        }
        GeometryType::Curve => {
            collect_line_strings(node, &mut geom.line_strings);
            geom.len = geom.line_strings.len() as u32;
            geom.pos = 0;
        }
        GeometryType::Point => {
            collect_points(node, &mut geom.points);
            geom.len = geom.points.len() as u32;
            geom.pos = 0;
        }
    }

    let empty = geom.polygons.is_empty() && geom.line_strings.is_empty() && geom.points.is_empty();

    if empty {
        None
    } else {
        Some(geom)
    }
}

// ------------------------------------------------------------------
// Polygon collection
// ------------------------------------------------------------------

/// Recursively collect all `gml:Polygon` leaves under `node` into `out`.
fn collect_polygons(node: &XmlNode, out: &mut Vec<Polygon3D<f64>>) {
    match local_name(&node.name) {
        // Solid wraps an exterior CompositeSurface/MultiSurface.
        "Solid" => {
            for child in element_children(node) {
                if local_name(&child.name) == "exterior" {
                    for inner in element_children(child) {
                        collect_polygons(inner, out);
                    }
                }
            }
        }
        // Surface aggregates / composites delegate to surfaceMember(s).
        "MultiSurface" | "CompositeSurface" => {
            for child in element_children(node) {
                let ln = local_name(&child.name);
                if ln == "surfaceMember" || ln == "surfaceMembers" {
                    for inner in element_children(child) {
                        collect_polygons(inner, out);
                    }
                }
            }
        }
        // Triangulated surfaces.
        "TriangulatedSurface" | "Tin" => {
            for child in element_children(node) {
                if local_name(&child.name) == "trianglePatches" {
                    for inner in element_children(child) {
                        if local_name(&inner.name) == "Triangle" {
                            if let Some(p) = parse_polygon(inner) {
                                out.push(p);
                            }
                        }
                    }
                }
            }
        }
        // Leaf polygon.
        "Polygon" | "Rectangle" | "Triangle" => {
            if let Some(p) = parse_polygon(node) {
                out.push(p);
            }
        }
        // Pass-through wrappers (e.g. surfaceMember, exterior encountered directly).
        _ => {
            for child in element_children(node) {
                collect_polygons(child, out);
            }
        }
    }
}

/// Parse one `gml:Polygon` (or compatible) node into a [`Polygon3D`].
fn parse_polygon(node: &XmlNode) -> Option<Polygon3D<f64>> {
    let mut exterior: Option<LineString3D<f64>> = None;
    let mut interiors: Vec<LineString3D<f64>> = Vec::new();

    for child in element_children(node) {
        match local_name(&child.name) {
            "exterior" => {
                if let Some(ring) = find_linear_ring(child) {
                    exterior = Some(parse_linear_ring(ring));
                }
            }
            "interior" => {
                if let Some(ring) = find_linear_ring(child) {
                    interiors.push(parse_linear_ring(ring));
                }
            }
            _ => {}
        }
    }

    exterior.map(|ext| Polygon3D::new(ext, interiors))
}

fn find_linear_ring(node: &XmlNode) -> Option<&XmlNode> {
    element_children(node).find(|c| local_name(&c.name) == "LinearRing")
}

/// Parse a `gml:LinearRing` into a [`LineString3D`].
fn parse_linear_ring(node: &XmlNode) -> LineString3D<f64> {
    let mut coords: Vec<Coordinate3D<f64>> = Vec::new();

    for child in element_children(node) {
        match local_name(&child.name) {
            "posList" => {
                coords = parse_pos_list(text_content(child));
                break;
            }
            "pos" => {
                if let Some(c) = parse_single_pos(text_content(child)) {
                    coords.push(c);
                }
            }
            _ => {}
        }
    }

    LineString3D::new(coords)
}

// ------------------------------------------------------------------
// Line-string collection
// ------------------------------------------------------------------

fn collect_line_strings(node: &XmlNode, out: &mut Vec<LineString3D<f64>>) {
    match local_name(&node.name) {
        "MultiCurve" | "CompositeCurve" => {
            for child in element_children(node) {
                let ln = local_name(&child.name);
                if ln == "curveMember" || ln == "curveMembers" {
                    for inner in element_children(child) {
                        collect_line_strings(inner, out);
                    }
                }
            }
        }
        "LineString" | "Curve" => {
            let mut coords: Vec<Coordinate3D<f64>> = Vec::new();
            for child in element_children(node) {
                match local_name(&child.name) {
                    "posList" => {
                        coords = parse_pos_list(text_content(child));
                        break;
                    }
                    "pos" => {
                        if let Some(c) = parse_single_pos(text_content(child)) {
                            coords.push(c);
                        }
                    }
                    _ => {}
                }
            }
            if !coords.is_empty() {
                out.push(LineString3D::new(coords));
            }
        }
        // Pass-through wrappers.
        _ => {
            for child in element_children(node) {
                collect_line_strings(child, out);
            }
        }
    }
}

// ------------------------------------------------------------------
// Point collection
// ------------------------------------------------------------------

fn collect_points(node: &XmlNode, out: &mut Vec<Coordinate3D<f64>>) {
    match local_name(&node.name) {
        "MultiPoint" => {
            for child in element_children(node) {
                let ln = local_name(&child.name);
                if ln == "pointMember" || ln == "pointMembers" {
                    for inner in element_children(child) {
                        collect_points(inner, out);
                    }
                }
            }
        }
        "Point" => {
            for child in element_children(node) {
                if local_name(&child.name) == "pos" {
                    if let Some(c) = parse_single_pos(text_content(child)) {
                        out.push(c);
                    }
                }
            }
        }
        _ => {}
    }
}

// ------------------------------------------------------------------
// Coordinate parsing
// ------------------------------------------------------------------

fn parse_pos_list(text: &str) -> Vec<Coordinate3D<f64>> {
    let values: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    values
        .chunks(3)
        .filter_map(|c| {
            if c.len() == 3 {
                Some(Coordinate3D::new__(c[0], c[1], c[2]))
            } else {
                None
            }
        })
        .collect()
}

fn parse_single_pos(text: &str) -> Option<Coordinate3D<f64>> {
    let vals: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if vals.len() >= 3 {
        Some(Coordinate3D::new__(vals[0], vals[1], vals[2]))
    } else {
        None
    }
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------

/// Returns the local part of a qualified name (`"bldg:lod1Solid"` → `"lod1Solid"`).
fn local_name(name: &str) -> &str {
    name.rfind(':').map(|i| &name[i + 1..]).unwrap_or(name)
}

/// Extract the LOD digit from a property local name like `lod2MultiSurface`.
/// Returns `None` if the name does not start with `lod[0-9]`.
fn extract_lod(local: &str) -> Option<u8> {
    if local.len() >= 4 && local.starts_with("lod") {
        local.chars().nth(3)?.to_digit(10).map(|d| d as u8)
    } else {
        None
    }
}

/// Map a GML geometry element's local name to a [`GeometryType`].
fn gml_element_geometry_type(local: &str) -> Option<GeometryType> {
    match local {
        "Solid" => Some(GeometryType::Solid),
        "MultiSurface" | "CompositeSurface" | "Polygon" | "Rectangle" => {
            Some(GeometryType::Surface)
        }
        "TriangulatedSurface" | "Tin" => Some(GeometryType::Triangle),
        "MultiCurve" | "CompositeCurve" | "LineString" | "Curve" => Some(GeometryType::Curve),
        "MultiPoint" | "Point" => Some(GeometryType::Point),
        _ => None,
    }
}

fn gml_id(node: &XmlNode) -> Option<String> {
    node.attrs
        .iter()
        .find(|(k, _)| k == "gml:id")
        .map(|(_, v)| v.clone())
}

/// Iterate only element children of a node (skip text nodes).
fn element_children(node: &XmlNode) -> impl Iterator<Item = &XmlNode> {
    node.children.iter().filter_map(|c| {
        if let XmlChild::Element(e) = c {
            Some(e.as_ref())
        } else {
            None
        }
    })
}

/// Concatenate all direct text children into a single string.
fn text_content(node: &XmlNode) -> &str {
    for child in &node.children {
        if let XmlChild::Text(t) = child {
            return t.as_str();
        }
    }
    ""
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::feature::reader::citygml3::parser::XmlChild;

    fn text_node(t: &str) -> XmlChild {
        XmlChild::Text(t.to_string())
    }

    fn elem(name: &str, attrs: Vec<(&str, &str)>, children: Vec<XmlChild>) -> XmlNode {
        XmlNode {
            name: name.to_string(),
            attrs: attrs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            children,
        }
    }

    fn elem_child(node: XmlNode) -> XmlChild {
        XmlChild::Element(Arc::new(node))
    }

    /// Build a minimal Polygon XmlNode from a flat list of coord triplets.
    fn polygon_node(coords: &[(f64, f64, f64)]) -> XmlNode {
        let pos_list: String = coords
            .iter()
            .map(|(x, y, z)| format!("{x} {y} {z}"))
            .collect::<Vec<_>>()
            .join(" ");

        elem(
            "gml:Polygon",
            vec![],
            vec![elem_child(elem(
                "gml:exterior",
                vec![],
                vec![elem_child(elem(
                    "gml:LinearRing",
                    vec![],
                    vec![elem_child(elem(
                        "gml:posList",
                        vec![],
                        vec![text_node(&pos_list)],
                    ))],
                ))],
            ))],
        )
    }

    #[test]
    fn test_parse_pos_list_basic() {
        let coords = parse_pos_list("1.0 2.0 3.0 4.0 5.0 6.0");
        assert_eq!(coords.len(), 2);
        assert_eq!(coords[0], Coordinate3D::new__(1.0, 2.0, 3.0));
        assert_eq!(coords[1], Coordinate3D::new__(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_parse_pos_list_truncated_triple_ignored() {
        // A trailing incomplete triple must be silently dropped.
        let coords = parse_pos_list("1.0 2.0 3.0 4.0 5.0");
        assert_eq!(coords.len(), 1);
    }

    #[test]
    fn test_parse_polygon() {
        let node = polygon_node(&[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)]);
        let poly = parse_polygon(&node).expect("expected a polygon");
        // Polygon3D::new closes the ring, so 3 input coords → 4 stored.
        assert_eq!(poly.exterior().0.len(), 4);
        assert!(poly.interiors().is_empty());
    }

    #[test]
    fn test_extract_lod() {
        assert_eq!(extract_lod("lod1Solid"), Some(1));
        assert_eq!(extract_lod("lod2MultiSurface"), Some(2));
        assert_eq!(extract_lod("lod0Point"), Some(0));
        assert_eq!(extract_lod("tin"), None);
        assert_eq!(extract_lod("description"), None);
    }

    #[test]
    fn test_extract_geometry_lod1_solid() {
        // Feature node:  bldg:lod1Solid > gml:Solid > gml:exterior >
        //                gml:CompositeSurface > gml:surfaceMember > gml:Polygon
        let polygon = polygon_node(&[
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ]);

        let surf_member = elem("gml:surfaceMember", vec![], vec![elem_child(polygon)]);
        let composite = elem(
            "gml:CompositeSurface",
            vec![],
            vec![elem_child(surf_member)],
        );
        let exterior = elem("gml:exterior", vec![], vec![elem_child(composite)]);
        let solid = elem(
            "gml:Solid",
            vec![("gml:id", "solid01")],
            vec![elem_child(exterior)],
        );
        let prop = elem("bldg:lod1Solid", vec![], vec![elem_child(solid)]);
        let feature = elem(
            "bldg:Building",
            vec![("gml:id", "BLD001")],
            vec![elem_child(prop)],
        );

        let geom = extract_geometry(&feature);

        assert_eq!(geom.gml_geometries.len(), 1);
        let g = &geom.gml_geometries[0];
        assert_eq!(g.ty, GeometryType::Solid);
        assert_eq!(g.lod, Some(1));
        assert_eq!(g.id, Some("solid01".to_string()));
        assert_eq!(g.polygons.len(), 1);
        assert_eq!(g.pos, 0);
        assert_eq!(g.len, 1);
    }

    #[test]
    fn test_extract_geometry_multi_lod_pos_tracking() {
        // Two LOD levels: lod1 (2 polygons) then lod2 (1 polygon).
        // lod2's pos must be 2, not 0.
        let make_prop = |prop_name: &str, gml_name: &str, n_polys: usize| {
            let polys: Vec<XmlChild> = (0..n_polys)
                .map(|_| {
                    elem_child(polygon_node(&[
                        (0.0, 0.0, 0.0),
                        (1.0, 0.0, 0.0),
                        (0.0, 1.0, 0.0),
                    ]))
                })
                .collect();
            let surf_members: Vec<XmlChild> = polys
                .into_iter()
                .map(|p| elem_child(elem("gml:surfaceMember", vec![], vec![p])))
                .collect();
            let ms = elem(gml_name, vec![], surf_members);
            elem(prop_name, vec![], vec![elem_child(ms)])
        };

        let feature = elem(
            "bldg:Building",
            vec![],
            vec![
                elem_child(make_prop("bldg:lod1MultiSurface", "gml:MultiSurface", 2)),
                elem_child(make_prop("bldg:lod2MultiSurface", "gml:MultiSurface", 1)),
            ],
        );

        let geom = extract_geometry(&feature);

        assert_eq!(geom.gml_geometries.len(), 2);
        assert_eq!(geom.gml_geometries[0].pos, 0);
        assert_eq!(geom.gml_geometries[0].len, 2);
        assert_eq!(geom.gml_geometries[1].pos, 2);
        assert_eq!(geom.gml_geometries[1].len, 1);
    }

    #[test]
    fn test_extract_geometry_no_geometry_property() {
        // A feature with only non-geometry children should return empty.
        let feature = elem(
            "bldg:Building",
            vec![],
            vec![elem_child(elem(
                "gml:description",
                vec![],
                vec![text_node("a building")],
            ))],
        );
        let geom = extract_geometry(&feature);
        assert!(geom.gml_geometries.is_empty());
    }
}
