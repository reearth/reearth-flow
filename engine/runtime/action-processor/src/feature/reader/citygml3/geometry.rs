//! CityGML 3 geometry extractor.
//!
//! Single entry point: [`extract_geometry`] takes a (xlink-resolved) [`XmlNode`]
//! and returns a [`CityGmlGeometry`].
//!
//! LOD is read from the wrapping property element name (e.g. `bldg:lod2MultiSurface`
//! → LOD 2). `pos`/`len` on each [`GmlGeometry`] index into the parallel polygon
//! appearance arrays; non-polygon types set `pos = 0`. Appearance data is not
//! extracted here — those arrays are empty until a later pass adds them.

use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::line_string::LineString3D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_types::{CityGmlGeometry, GeometryType, GmlGeometry};

use super::parser::{XmlChild, XmlNode};

pub fn extract_geometry(node: &XmlNode) -> CityGmlGeometry {
    let mut gml_geometries: Vec<GmlGeometry> = Vec::new();
    let mut local_pos: u32 = 0;

    for child in element_children(node) {
        let ln = local_name(&child.name);

        let lod_opt = if ln == "tin" {
            Some(None)
        } else if let Some(lod) = extract_lod(ln) {
            Some(Some(lod))
        } else {
            None
        };

        let Some(lod) = lod_opt else { continue };
        let Some(geom_node) = element_children(child).next() else { continue };
        let geom_ln = local_name(&geom_node.name);

        if geom_ln == "ImplicitGeometry" {
            if let Some(g) = parse_implicit_geom(geom_node, lod, &mut local_pos) {
                gml_geometries.push(g);
            }
            continue;
        }

        let Some(ty) = gml_element_geometry_type(geom_ln) else { continue };
        if let Some(g) = parse_gml_geom(geom_node, ty, lod, &mut local_pos) {
            gml_geometries.push(g);
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

fn parse_gml_geom(
    node: &XmlNode,
    ty: GeometryType,
    lod: Option<u8>,
    local_pos: &mut u32,
) -> Option<GmlGeometry> {
    let mut geom = GmlGeometry::new(ty, lod);
    geom.id = gml_id(node);

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
    if empty { None } else { Some(geom) }
}

// ImplicitGeometry (OGC 21-006r2 §9.3): apply the 4×4 transformationMatrix to
// the relativeGeometry template to obtain world-space coordinates.
// referencePoint is not added — the translation column of the matrix encodes
// the world-space origin and referencePoint is a redundant spatial index hint.
fn parse_implicit_geom(
    node: &XmlNode,
    lod: Option<u8>,
    local_pos: &mut u32,
) -> Option<GmlGeometry> {
    let matrix = find_child(node, "transformationMatrix")
        .and_then(|n| parse_matrix4(text_content(n)))?;
    let rel_geom_node = find_child(node, "relativeGeometry")?;
    let geom_node = element_children(rel_geom_node).next()?;
    let ty = gml_element_geometry_type(local_name(&geom_node.name))?;

    let mut geom = GmlGeometry::new(ty, lod);
    geom.id = gml_id(geom_node);

    match ty {
        GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
            let mut polys = Vec::new();
            collect_polygons(geom_node, &mut polys);
            geom.polygons = polys.into_iter().map(|p| transform_polygon(p, &matrix)).collect();
            geom.len = geom.polygons.len() as u32;
            geom.pos = *local_pos;
            *local_pos += geom.len;
        }
        GeometryType::Curve => {
            let mut ls = Vec::new();
            collect_line_strings(geom_node, &mut ls);
            geom.line_strings = ls.into_iter().map(|l| transform_line_string(l, &matrix)).collect();
            geom.len = geom.line_strings.len() as u32;
            geom.pos = 0;
        }
        GeometryType::Point => {
            let mut pts = Vec::new();
            collect_points(geom_node, &mut pts);
            geom.points = pts.into_iter().map(|p| transform_coord(p, &matrix)).collect();
            geom.len = geom.points.len() as u32;
            geom.pos = 0;
        }
    }

    let empty = geom.polygons.is_empty() && geom.line_strings.is_empty() && geom.points.is_empty();
    if empty { None } else { Some(geom) }
}

fn parse_matrix4(text: &str) -> Option<[f64; 16]> {
    let vals: Vec<f64> = text.split_whitespace().filter_map(|s| s.parse().ok()).collect();
    if vals.len() == 16 {
        let mut m = [0.0f64; 16];
        m.copy_from_slice(&vals);
        Some(m)
    } else {
        None
    }
}

#[inline]
fn transform_coord(c: Coordinate3D<f64>, m: &[f64; 16]) -> Coordinate3D<f64> {
    Coordinate3D::new__(
        m[0] * c.x + m[1] * c.y + m[2] * c.z + m[3],
        m[4] * c.x + m[5] * c.y + m[6] * c.z + m[7],
        m[8] * c.x + m[9] * c.y + m[10] * c.z + m[11],
    )
}

fn transform_line_string(ls: LineString3D<f64>, m: &[f64; 16]) -> LineString3D<f64> {
    LineString3D::new(ls.0.into_iter().map(|c| transform_coord(c, m)).collect())
}

fn transform_polygon(poly: Polygon3D<f64>, m: &[f64; 16]) -> Polygon3D<f64> {
    let (ext, ints) = poly.into_inner();
    Polygon3D::new(
        transform_line_string(ext, m),
        ints.into_iter().map(|i| transform_line_string(i, m)).collect(),
    )
}

fn collect_polygons(node: &XmlNode, out: &mut Vec<Polygon3D<f64>>) {
    match local_name(&node.name) {
        "Solid" => {
            for child in element_children(node) {
                if local_name(&child.name) == "exterior" {
                    for inner in element_children(child) {
                        collect_polygons(inner, out);
                    }
                }
            }
        }
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
        "Polygon" | "Rectangle" | "Triangle" => {
            if let Some(p) = parse_polygon(node) {
                out.push(p);
            }
        }
        _ => {
            for child in element_children(node) {
                collect_polygons(child, out);
            }
        }
    }
}

fn parse_polygon(node: &XmlNode) -> Option<Polygon3D<f64>> {
    let mut exterior: Option<LineString3D<f64>> = None;
    let mut interiors: Vec<LineString3D<f64>> = Vec::new();

    for child in element_children(node) {
        match local_name(&child.name) {
            "exterior" => {
                if let Some(ring) = find_child(child, "LinearRing") {
                    exterior = Some(parse_linear_ring(ring));
                }
            }
            "interior" => {
                if let Some(ring) = find_child(child, "LinearRing") {
                    interiors.push(parse_linear_ring(ring));
                }
            }
            _ => {}
        }
    }

    exterior.map(|ext| Polygon3D::new(ext, interiors))
}

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
        _ => {
            for child in element_children(node) {
                collect_line_strings(child, out);
            }
        }
    }
}

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

fn parse_pos_list(text: &str) -> Vec<Coordinate3D<f64>> {
    let values: Vec<f64> = text.split_whitespace().filter_map(|s| s.parse().ok()).collect();
    values
        .chunks(3)
        .filter_map(|c| {
            if c.len() == 3 { Some(Coordinate3D::new__(c[0], c[1], c[2])) } else { None }
        })
        .collect()
}

fn parse_single_pos(text: &str) -> Option<Coordinate3D<f64>> {
    let vals: Vec<f64> = text.split_whitespace().filter_map(|s| s.parse().ok()).collect();
    if vals.len() >= 3 { Some(Coordinate3D::new__(vals[0], vals[1], vals[2])) } else { None }
}

fn local_name(name: &str) -> &str {
    name.rfind(':').map(|i| &name[i + 1..]).unwrap_or(name)
}

fn extract_lod(local: &str) -> Option<u8> {
    if local.len() >= 4 && local.starts_with("lod") {
        local.chars().nth(3)?.to_digit(10).map(|d| d as u8)
    } else {
        None
    }
}

fn gml_element_geometry_type(local: &str) -> Option<GeometryType> {
    match local {
        "Solid" => Some(GeometryType::Solid),
        "MultiSurface" | "CompositeSurface" | "Polygon" | "Rectangle" => Some(GeometryType::Surface),
        "TriangulatedSurface" | "Tin" => Some(GeometryType::Triangle),
        "MultiCurve" | "CompositeCurve" | "LineString" | "Curve" => Some(GeometryType::Curve),
        "MultiPoint" | "Point" => Some(GeometryType::Point),
        _ => None,
    }
}

fn gml_id(node: &XmlNode) -> Option<String> {
    node.attrs.iter().find(|(k, _)| k == "gml:id").map(|(_, v)| v.clone())
}

fn find_child<'a>(node: &'a XmlNode, local: &str) -> Option<&'a XmlNode> {
    element_children(node).find(|c| local_name(&c.name) == local)
}

fn element_children(node: &XmlNode) -> impl Iterator<Item = &XmlNode> {
    node.children.iter().filter_map(|c| {
        if let XmlChild::Element(e) = c { Some(e.as_ref()) } else { None }
    })
}

fn text_content(node: &XmlNode) -> &str {
    for child in &node.children {
        if let XmlChild::Text(t) = child {
            return t.as_str();
        }
    }
    ""
}

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
            attrs: attrs.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            children,
        }
    }

    fn elem_child(node: XmlNode) -> XmlChild {
        XmlChild::Element(Arc::new(node))
    }

    fn polygon_node(coords: &[(f64, f64, f64)]) -> XmlNode {
        let pos_list = coords.iter().map(|(x, y, z)| format!("{x} {y} {z}")).collect::<Vec<_>>().join(" ");
        elem("gml:Polygon", vec![], vec![elem_child(elem(
            "gml:exterior", vec![], vec![elem_child(elem(
                "gml:LinearRing", vec![], vec![elem_child(elem(
                    "gml:posList", vec![], vec![text_node(&pos_list)],
                ))],
            ))],
        ))])
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
        let coords = parse_pos_list("1.0 2.0 3.0 4.0 5.0");
        assert_eq!(coords.len(), 1);
    }

    #[test]
    fn test_parse_polygon() {
        let node = polygon_node(&[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)]);
        let poly = parse_polygon(&node).expect("expected a polygon");
        assert_eq!(poly.exterior().0.len(), 4); // Polygon3D::new closes the ring
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
        let polygon = polygon_node(&[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 0.0)]);
        let solid = elem("gml:Solid", vec![("gml:id", "solid01")], vec![elem_child(elem(
            "gml:exterior", vec![], vec![elem_child(elem(
                "gml:CompositeSurface", vec![], vec![elem_child(elem(
                    "gml:surfaceMember", vec![], vec![elem_child(polygon)],
                ))],
            ))],
        ))]);
        let feature = elem("bldg:Building", vec![("gml:id", "BLD001")], vec![
            elem_child(elem("bldg:lod1Solid", vec![], vec![elem_child(solid)])),
        ]);

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
        // lod1 (2 polygons), lod2 (1 polygon): lod2's pos must be 2.
        let make_prop = |prop_name: &str, gml_name: &str, n_polys: usize| {
            let surf_members: Vec<XmlChild> = (0..n_polys)
                .map(|_| elem_child(elem("gml:surfaceMember", vec![], vec![
                    elem_child(polygon_node(&[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)])),
                ])))
                .collect();
            elem(prop_name, vec![], vec![elem_child(elem(gml_name, vec![], surf_members))])
        };

        let feature = elem("bldg:Building", vec![], vec![
            elem_child(make_prop("bldg:lod1MultiSurface", "gml:MultiSurface", 2)),
            elem_child(make_prop("bldg:lod2MultiSurface", "gml:MultiSurface", 1)),
        ]);

        let geom = extract_geometry(&feature);
        assert_eq!(geom.gml_geometries.len(), 2);
        assert_eq!(geom.gml_geometries[0].pos, 0);
        assert_eq!(geom.gml_geometries[0].len, 2);
        assert_eq!(geom.gml_geometries[1].pos, 2);
        assert_eq!(geom.gml_geometries[1].len, 1);
    }

    #[test]
    fn test_implicit_geometry_transform() {
        // transformationMatrix: identity + translate (10, 20, 0)
        // relativeGeometry: unit square at local z=0
        // Expected: vertices shifted by (+10, +20, 0).
        let pos_list = "0 0 0 1 0 0 1 1 0 0 1 0 0 0 0";
        let rel_geom = elem("gml:MultiSurface", vec![("gml:id", "geom_template1")], vec![
            elem_child(elem("gml:surfaceMember", vec![], vec![elem_child(elem(
                "gml:Polygon", vec![], vec![elem_child(elem(
                    "gml:exterior", vec![], vec![elem_child(elem(
                        "gml:LinearRing", vec![], vec![elem_child(elem(
                            "gml:posList", vec![], vec![text_node(pos_list)],
                        ))],
                    ))],
                ))],
            ))])),
        ]);
        let implicit = elem("core:ImplicitGeometry", vec![], vec![
            elem_child(elem("core:transformationMatrix", vec![], vec![
                text_node("1 0 0 10  0 1 0 20  0 0 1 0  0 0 0 1"),
            ])),
            elem_child(elem("core:relativeGeometry", vec![], vec![elem_child(rel_geom)])),
        ]);
        let feature = elem("frn:CityFurniture", vec![("gml:id", "furniture2")], vec![
            elem_child(elem("core:lod2ImplicitRepresentation", vec![], vec![elem_child(implicit)])),
        ]);

        let geom = extract_geometry(&feature);
        assert_eq!(geom.gml_geometries.len(), 1);
        let g = &geom.gml_geometries[0];
        assert_eq!(g.ty, GeometryType::Surface);
        assert_eq!(g.lod, Some(2));
        let first = g.polygons[0].exterior().0[0];
        assert!((first.x - 10.0).abs() < 1e-10);
        assert!((first.y - 20.0).abs() < 1e-10);
        assert!((first.z).abs() < 1e-10);
        let third = g.polygons[0].exterior().0[2];
        assert!((third.x - 11.0).abs() < 1e-10);
        assert!((third.y - 21.0).abs() < 1e-10);
    }

    #[test]
    fn test_extract_geometry_no_geometry_property() {
        let feature = elem("bldg:Building", vec![], vec![
            elem_child(elem("gml:description", vec![], vec![text_node("a building")])),
        ]);
        let geom = extract_geometry(&feature);
        assert!(geom.gml_geometries.is_empty());
    }
}
