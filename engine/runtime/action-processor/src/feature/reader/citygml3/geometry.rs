use std::sync::Arc;

use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::line_string::LineString3D;
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_types::{GeometryType, GmlGeometry};

use super::utils::{local_name, XmlChild, XmlNode, GML_NS_ID};

pub fn extract_geometries(node: &Arc<XmlNode>) -> (Arc<XmlNode>, Vec<GmlGeometry>) {
    let mut out: Vec<GmlGeometry> = Vec::new();
    let stripped = strip_and_collect(node, &mut out);
    (stripped, out)
}

fn strip_and_collect(node: &Arc<XmlNode>, out: &mut Vec<GmlGeometry>) -> Arc<XmlNode> {
    let mut new_children: Option<Vec<XmlChild>> = None;

    for (i, child) in node.children.iter().enumerate() {
        let XmlChild::Element(e) = child else {
            if let Some(ref mut nc) = new_children {
                nc.push(child.clone());
            }
            continue;
        };

        let ln = local_name(&e.name.0);
        let lod_opt = if ln == "tin" {
            Some(None)
        } else {
            extract_lod(ln).map(Some)
        };

        if let Some(lod) = lod_opt {
            collect_geometry_from_property(e, lod, out);
            if new_children.is_none() {
                new_children = Some(node.children[..i].to_vec());
            }
        } else {
            let stripped_child = strip_and_collect(e, out);
            match new_children {
                None => {
                    if !Arc::ptr_eq(&stripped_child, e) {
                        let mut nc = node.children[..i].to_vec();
                        nc.push(XmlChild::Element(stripped_child));
                        new_children = Some(nc);
                    }
                }
                Some(ref mut nc) => {
                    nc.push(XmlChild::Element(stripped_child));
                }
            }
        }
    }

    match new_children {
        None => Arc::clone(node),
        Some(children) => Arc::new(XmlNode {
            name: node.name.clone(),
            attrs: node.attrs.clone(),
            children,
        }),
    }
}

fn collect_geometry_from_property(prop: &XmlNode, lod: Option<u8>, out: &mut Vec<GmlGeometry>) {
    let Some(geom_node) = element_children(prop).next() else {
        return;
    };
    let geom_ln = local_name(&geom_node.name.0);
    if geom_ln == "ImplicitGeometry" {
        tracing::warn!("citygml3 geometry: transformationMatrix/referencePoint not supported");
        if let Some(rel_geom) = find_child(geom_node, "relativeGeometry") {
            collect_geometry_from_property(rel_geom, lod, out);
        }
    } else if geom_ln == "GeometricComplex" {
        parse_geometric_complex(geom_node, lod, out);
    } else if geom_ln == "MultiGeometry" {
        parse_multi_geometry(geom_node, lod, out);
    } else if let Some(ty) = gml_element_geometry_type(geom_ln) {
        if let Some(g) = parse_gml_geom(geom_node, ty, lod) {
            out.push(g);
        }
    } else {
        tracing::warn!(
            element = geom_ln,
            "citygml3 geometry: unrecognized geometry element inside lod property, skipped"
        );
    }
}

fn parse_gml_geom(node: &XmlNode, ty: GeometryType, lod: Option<u8>) -> Option<GmlGeometry> {
    let mut geom = GmlGeometry::new(ty, lod);
    geom.id = gml_id(node);

    match ty {
        GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
            collect_polygons(node, &mut geom.polygons);
            geom.len = geom.polygons.len() as u32;
        }
        GeometryType::Curve => {
            collect_line_strings(node, &mut geom.line_strings);
            geom.len = geom.line_strings.len() as u32;
        }
        GeometryType::Point => {
            collect_points(node, &mut geom.points);
            geom.len = geom.points.len() as u32;
        }
    }

    let empty = geom.polygons.is_empty() && geom.line_strings.is_empty() && geom.points.is_empty();
    if empty {
        None
    } else {
        Some(geom)
    }
}

fn collect_polygons(node: &XmlNode, out: &mut Vec<Polygon3D<f64>>) {
    match local_name(&node.name.0) {
        "Solid" => {
            for child in element_children(node) {
                match local_name(&child.name.0) {
                    "exterior" => {
                        for inner in element_children(child) {
                            collect_polygons(inner, out);
                        }
                    }
                    "interior" => {
                        tracing::warn!(
                            element = local_name(&child.name.0),
                            "citygml3 geometry: interior of Solid is not supported, skipped"
                        );
                    }
                    _ => {}
                }
            }
        }
        "MultiSolid" | "CompositeSolid" => {
            for child in element_children(node) {
                let ln = local_name(&child.name.0);
                if ln == "solidMember" || ln == "solidMembers" {
                    for inner in element_children(child) {
                        collect_polygons(inner, out);
                    }
                }
            }
        }
        "Shell" | "MultiSurface" | "CompositeSurface" => {
            for child in element_children(node) {
                let ln = local_name(&child.name.0);
                if ln == "surfaceMember" || ln == "surfaceMembers" {
                    for inner in element_children(child) {
                        collect_polygons(inner, out);
                    }
                }
            }
        }
        "Surface" | "PolyhedralSurface" => {
            for child in element_children(node) {
                if local_name(&child.name.0) == "patches" {
                    for inner in element_children(child) {
                        collect_polygons(inner, out);
                    }
                }
            }
        }
        "OrientableSurface" => {
            for child in element_children(node) {
                if local_name(&child.name.0) == "baseSurface" {
                    for inner in element_children(child) {
                        collect_polygons(inner, out);
                    }
                }
            }
        }
        "TriangulatedSurface" | "Tin" => {
            for child in element_children(node) {
                let ln = local_name(&child.name.0);
                if ln == "patches" || ln == "trianglePatches" {
                    for inner in element_children(child) {
                        collect_polygons(inner, out);
                    }
                }
            }
        }
        "Polygon" | "PolygonPatch" | "Rectangle" | "Triangle" => {
            if let Some(p) = parse_polygon(node) {
                out.push(p);
            }
        }
        _ => {
            tracing::warn!(
                element = local_name(&node.name.0),
                "citygml3 geometry: unhandled element in polygon collection"
            );
        }
    }
}

// GeometricComplex: each <gml:element> child holds exactly one primitive.
fn parse_geometric_complex(node: &XmlNode, lod: Option<u8>, out: &mut Vec<GmlGeometry>) {
    let mut curves: Vec<LineString3D<f64>> = Vec::new();
    let mut polys: Vec<Polygon3D<f64>> = Vec::new();
    let mut points: Vec<Coordinate3D<f64>> = Vec::new();

    for child in element_children(node) {
        if local_name(&child.name.0) == "element" {
            for prim in element_children(child) {
                dispatch_primitive(prim, &mut curves, &mut polys, &mut points);
            }
        }
    }
    emit_typed_geoms(lod, curves, polys, points, out);
}

fn parse_multi_geometry(node: &XmlNode, lod: Option<u8>, out: &mut Vec<GmlGeometry>) {
    let mut curves: Vec<LineString3D<f64>> = Vec::new();
    let mut polys: Vec<Polygon3D<f64>> = Vec::new();
    let mut points: Vec<Coordinate3D<f64>> = Vec::new();

    for child in element_children(node) {
        let ln = local_name(&child.name.0);
        if ln == "geometryMember" || ln == "geometryMembers" {
            for member in element_children(child) {
                dispatch_primitive(member, &mut curves, &mut polys, &mut points);
            }
        }
    }
    emit_typed_geoms(lod, curves, polys, points, out);
}

fn dispatch_primitive(
    node: &XmlNode,
    curves: &mut Vec<LineString3D<f64>>,
    polys: &mut Vec<Polygon3D<f64>>,
    points: &mut Vec<Coordinate3D<f64>>,
) {
    let ln = local_name(&node.name.0);
    match gml_element_geometry_type(ln) {
        Some(GeometryType::Curve) => collect_line_strings(node, curves),
        Some(GeometryType::Surface | GeometryType::Triangle | GeometryType::Solid) => {
            collect_polygons(node, polys)
        }
        Some(GeometryType::Point) => collect_points(node, points),
        None => tracing::warn!(
            element = ln,
            "citygml3 geometry: unrecognized primitive in aggregate geometry, skipped"
        ),
    }
}

fn emit_typed_geoms(
    lod: Option<u8>,
    curves: Vec<LineString3D<f64>>,
    polys: Vec<Polygon3D<f64>>,
    points: Vec<Coordinate3D<f64>>,
    out: &mut Vec<GmlGeometry>,
) {
    if !curves.is_empty() {
        let mut g = GmlGeometry::new(GeometryType::Curve, lod);
        g.len = curves.len() as u32;
        g.line_strings = curves;
        out.push(g);
    }
    if !polys.is_empty() {
        let mut g = GmlGeometry::new(GeometryType::Surface, lod);
        g.len = polys.len() as u32;
        g.polygons = polys;
        out.push(g);
    }
    if !points.is_empty() {
        let mut g = GmlGeometry::new(GeometryType::Point, lod);
        g.len = points.len() as u32;
        g.points = points;
        out.push(g);
    }
}

fn parse_polygon(node: &XmlNode) -> Option<Polygon3D<f64>> {
    let mut exterior: Option<LineString3D<f64>> = None;
    let mut interiors: Vec<LineString3D<f64>> = Vec::new();

    for child in element_children(node) {
        match local_name(&child.name.0) {
            "exterior" => {
                if let Some(ring) = find_child(child, "LinearRing") {
                    exterior = Some(parse_polygon_ring(ring, "exterior")?);
                }
            }
            "interior" => {
                if let Some(ring) = find_child(child, "LinearRing") {
                    interiors.push(parse_polygon_ring(ring, "interior")?);
                }
            }
            _ => {}
        }
    }

    exterior.map(|ext| Polygon3D::new(ext, interiors))
}

fn collect_coords(node: &XmlNode) -> Result<Vec<Coordinate3D<f64>>, &'static str> {
    let mut coords: Vec<Coordinate3D<f64>> = Vec::new();
    for child in element_children(node) {
        match local_name(&child.name.0) {
            "posList" => {
                return parse_pos_list(text_content(child));
            }
            "pos" => {
                coords.push(parse_single_pos(text_content(child))?);
            }
            other => {
                tracing::warn!(
                    element = other,
                    parent = local_name(&node.name.0),
                    "citygml3 geometry: unexpected element in coordinate position, skipped"
                );
            }
        }
    }
    Ok(coords)
}

fn parse_linear_ring(node: &XmlNode) -> Result<LineString3D<f64>, &'static str> {
    Ok(LineString3D::new(collect_coords(node)?))
}

fn parse_polygon_ring(node: &XmlNode, role: &'static str) -> Option<LineString3D<f64>> {
    parse_linear_ring(node)
        .map_err(|err| {
            tracing::error!(
                error = %err,
                ring_role = role,
                "citygml3 geometry: invalid LinearRing coordinates, skipped polygon"
            );
        })
        .ok()
}

fn collect_line_strings(node: &XmlNode, out: &mut Vec<LineString3D<f64>>) {
    match local_name(&node.name.0) {
        "MultiCurve" | "CompositeCurve" => {
            for child in element_children(node) {
                let ln = local_name(&child.name.0);
                if ln == "curveMember" || ln == "curveMembers" {
                    for inner in element_children(child) {
                        collect_line_strings(inner, out);
                    }
                }
            }
        }
        "OrientableCurve" => {
            for child in element_children(node) {
                if local_name(&child.name.0) == "baseCurve" {
                    for inner in element_children(child) {
                        collect_line_strings(inner, out);
                    }
                }
            }
        }
        "LineString" => {
            if let Some(line_string) = parse_line_string(node, "LineString") {
                out.push(line_string);
            }
        }
        "Curve" => {
            for child in element_children(node) {
                if local_name(&child.name.0) == "segments" {
                    for seg in element_children(child) {
                        let seg_ln = local_name(&seg.name.0);
                        if seg_ln == "LineStringSegment" {
                            if let Some(line_string) = parse_line_string(seg, "LineStringSegment") {
                                out.push(line_string);
                            }
                        } else {
                            tracing::warn!(
                                element = seg_ln,
                                "citygml3 geometry: unsupported curve segment type, skipped"
                            );
                        }
                    }
                }
            }
        }
        _ => {
            tracing::warn!(
                element = local_name(&node.name.0),
                "citygml3 geometry: unhandled element in line string collection"
            );
        }
    }
}

fn collect_points(node: &XmlNode, out: &mut Vec<Coordinate3D<f64>>) {
    match local_name(&node.name.0) {
        "MultiPoint" => {
            for child in element_children(node) {
                let ln = local_name(&child.name.0);
                if ln == "pointMember" || ln == "pointMembers" {
                    for inner in element_children(child) {
                        collect_points(inner, out);
                    }
                }
            }
        }
        "Point" => {
            for child in element_children(node) {
                if local_name(&child.name.0) == "pos" {
                    if let Some(point) = parse_point_pos(child) {
                        out.push(point);
                    }
                }
            }
        }
        _ => {
            tracing::warn!(
                element = local_name(&node.name.0),
                "citygml3 geometry: unhandled element in point collection"
            );
        }
    }
}

fn parse_line_string(node: &XmlNode, geometry_type: &'static str) -> Option<LineString3D<f64>> {
    collect_coords(node)
        .map(LineString3D::new)
        .map_err(|err| {
            tracing::error!(
                error = %err,
                geometry_type,
                "citygml3 geometry: invalid coordinates, skipped"
            );
        })
        .ok()
        .filter(|line_string| !line_string.is_empty())
}

fn parse_point_pos(node: &XmlNode) -> Option<Coordinate3D<f64>> {
    parse_single_pos(text_content(node))
        .map_err(|err| {
            tracing::error!(
                error = %err,
                "citygml3 geometry: invalid Point coordinates, skipped"
            );
        })
        .ok()
}

fn parse_pos_list(text: &str) -> Result<Vec<Coordinate3D<f64>>, &'static str> {
    let values: Vec<f64> = text
        .split_whitespace()
        .map(|s| s.parse::<f64>().map_err(|_| "invalid gml:posList content"))
        .collect::<Result<_, _>>()?;

    if !values.len().is_multiple_of(3) {
        return Err("invalid gml:posList content");
    }

    Ok(values
        .chunks_exact(3)
        .map(|c| Coordinate3D::new__(c[0], c[1], c[2]))
        .collect())
}

fn parse_single_pos(text: &str) -> Result<Coordinate3D<f64>, &'static str> {
    let vals: Vec<f64> = text
        .split_whitespace()
        .map(|s| s.parse::<f64>().map_err(|_| "invalid gml:pos content"))
        .collect::<Result<_, _>>()?;

    if vals.len() != 3 {
        return Err("invalid gml:pos content");
    }

    Ok(Coordinate3D::new__(vals[0], vals[1], vals[2]))
}

fn extract_lod(local: &str) -> Option<u8> {
    if local.len() >= 4 && local.starts_with("lod") {
        local.chars().nth(3)?.to_digit(10).map(|d| d as u8)
    } else {
        None
    }
}

// `AbstractGeometry` subtypes that can appear as the top-level value of
// an lod/tin property or as a direct member of `MultiGeometry`/`GeometricComplex`
fn gml_element_geometry_type(local: &str) -> Option<GeometryType> {
    match local {
        "Solid" | "MultiSolid" | "CompositeSolid" => Some(GeometryType::Solid),
        "MultiSurface" | "CompositeSurface" | "Surface" | "PolyhedralSurface"
        | "OrientableSurface" | "Polygon" => Some(GeometryType::Surface),
        "TriangulatedSurface" | "Tin" => Some(GeometryType::Triangle),
        "MultiCurve" | "CompositeCurve" | "OrientableCurve" | "LineString" | "Curve" => {
            Some(GeometryType::Curve)
        }
        "MultiPoint" | "Point" => Some(GeometryType::Point),
        _ => None,
    }
}

fn gml_id(node: &XmlNode) -> Option<String> {
    node.attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "id" && *ns == GML_NS_ID)
        .map(|(_, v)| v.clone())
}

fn find_child<'a>(node: &'a XmlNode, local: &str) -> Option<&'a XmlNode> {
    element_children(node).find(|c| local_name(&c.name.0) == local)
}

/// Iterate element children.
fn element_children(node: &XmlNode) -> impl Iterator<Item = &XmlNode> {
    node.children.iter().filter_map(|c| match c {
        XmlChild::Element(e) => Some(e.as_ref()),
        XmlChild::Text(_) => None,
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
    use crate::feature::reader::citygml3::utils::{
        NsId, XmlChild, EMPTY_NS_ID, GML_NS, GML_NS_ID, XLINK_NS, XLINK_NS_ID,
    };

    fn text_node(t: &str) -> XmlChild {
        XmlChild::Text(t.to_string())
    }

    fn ns_id(ns: &str) -> NsId {
        match ns {
            GML_NS => GML_NS_ID,
            XLINK_NS => XLINK_NS_ID,
            _ => EMPTY_NS_ID,
        }
    }

    fn elem(name: &str, attrs: Vec<(&str, &str, &str)>, children: Vec<XmlChild>) -> XmlNode {
        XmlNode {
            name: (name.to_string(), EMPTY_NS_ID),
            attrs: attrs
                .into_iter()
                .map(|(q, ns, v)| ((q.to_string(), ns_id(ns)), v.to_string()))
                .collect(),
            children,
        }
    }

    fn elem_child(node: XmlNode) -> XmlChild {
        XmlChild::Element(Arc::new(node))
    }

    fn polygon_node(coords: &[(f64, f64, f64)]) -> XmlNode {
        let pos_list = coords
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
        let coords = parse_pos_list("1.0 2.0 3.0 4.0 5.0 6.0").expect("expected valid posList");
        assert_eq!(coords.len(), 2);
        assert_eq!(coords[0], Coordinate3D::new__(1.0, 2.0, 3.0));
        assert_eq!(coords[1], Coordinate3D::new__(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_parse_pos_list_truncated_triple_errors() {
        assert!(parse_pos_list("1.0 2.0 3.0 4.0 5.0").is_err());
    }

    #[test]
    fn test_parse_pos_list_invalid_number_errors() {
        assert!(parse_pos_list("1.0 2.0 nope").is_err());
    }

    #[test]
    fn test_parse_single_pos_requires_exactly_three_ordinates() {
        assert!(parse_single_pos("1.0 2.0 3.0 4.0").is_err());
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
    fn test_extract_geometries_lod1_solid() {
        let polygon = polygon_node(&[
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ]);
        let solid = elem(
            "gml:Solid",
            vec![("gml:id", "http://www.opengis.net/gml/3.2", "solid01")],
            vec![elem_child(elem(
                "gml:exterior",
                vec![],
                vec![elem_child(elem(
                    "gml:CompositeSurface",
                    vec![],
                    vec![elem_child(elem(
                        "gml:surfaceMember",
                        vec![],
                        vec![elem_child(polygon)],
                    ))],
                ))],
            ))],
        );
        let feature = Arc::new(elem(
            "bldg:Building",
            vec![("gml:id", "http://www.opengis.net/gml/3.2", "BLD001")],
            vec![elem_child(elem(
                "bldg:lod1Solid",
                vec![],
                vec![elem_child(solid)],
            ))],
        ));

        let (_, geoms) = extract_geometries(&feature);
        assert_eq!(geoms.len(), 1);
        let g = &geoms[0];
        assert_eq!(g.ty, GeometryType::Solid);
        assert_eq!(g.lod, Some(1));
        assert_eq!(g.id, Some("solid01".to_string()));
        assert_eq!(g.polygons.len(), 1);
        assert_eq!(g.len, 1);
    }

    #[test]
    fn test_extract_geometries_multi_lod_len() {
        let make_prop = |prop_name: &str, gml_name: &str, n_polys: usize| {
            let surf_members: Vec<XmlChild> = (0..n_polys)
                .map(|_| {
                    elem_child(elem(
                        "gml:surfaceMember",
                        vec![],
                        vec![elem_child(polygon_node(&[
                            (0.0, 0.0, 0.0),
                            (1.0, 0.0, 0.0),
                            (0.0, 1.0, 0.0),
                        ]))],
                    ))
                })
                .collect();
            elem(
                prop_name,
                vec![],
                vec![elem_child(elem(gml_name, vec![], surf_members))],
            )
        };

        let feature = Arc::new(elem(
            "bldg:Building",
            vec![],
            vec![
                elem_child(make_prop("bldg:lod1MultiSurface", "gml:MultiSurface", 2)),
                elem_child(make_prop("bldg:lod2MultiSurface", "gml:MultiSurface", 1)),
            ],
        ));

        let (_, geoms) = extract_geometries(&feature);
        assert_eq!(geoms.len(), 2);
        assert_eq!(geoms[0].len, 2);
        assert_eq!(geoms[1].len, 1);
    }

    #[test]
    fn test_implicit_geometry_relative_geom_parsed() {
        let pos_list = "0 0 0 1 0 0 1 1 0 0 1 0 0 0 0";
        let rel_geom = elem(
            "gml:MultiSurface",
            vec![("gml:id", "http://www.opengis.net/gml/3.2", "geom_template1")],
            vec![elem_child(elem(
                "gml:surfaceMember",
                vec![],
                vec![elem_child(elem(
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
                                vec![text_node(pos_list)],
                            ))],
                        ))],
                    ))],
                ))],
            ))],
        );
        let implicit = elem(
            "core:ImplicitGeometry",
            vec![],
            vec![
                elem_child(elem(
                    "core:transformationMatrix",
                    vec![],
                    vec![text_node("1 0 0 10  0 1 0 20  0 0 1 0  0 0 0 1")],
                )),
                elem_child(elem(
                    "core:relativeGeometry",
                    vec![],
                    vec![elem_child(rel_geom)],
                )),
            ],
        );
        let feature = Arc::new(elem(
            "frn:CityFurniture",
            vec![("gml:id", "http://www.opengis.net/gml/3.2", "furniture2")],
            vec![elem_child(elem(
                "core:lod2ImplicitRepresentation",
                vec![],
                vec![elem_child(implicit)],
            ))],
        ));

        let (_, geoms) = extract_geometries(&feature);
        assert_eq!(geoms.len(), 1);
        let g = &geoms[0];
        assert_eq!(g.ty, GeometryType::Surface);
        assert_eq!(g.lod, Some(2));
        // transform is not applied; coordinates are from relativeGeometry as-is
        let first = g.polygons[0].exterior().0[0];
        assert!((first.x - 0.0).abs() < 1e-10);
        assert!((first.y - 0.0).abs() < 1e-10);
        assert!((first.z - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_extract_geometries_no_geometry_property() {
        let feature = Arc::new(elem(
            "bldg:Building",
            vec![],
            vec![elem_child(elem(
                "gml:description",
                vec![],
                vec![text_node("a building")],
            ))],
        ));
        let (stripped, geoms) = extract_geometries(&feature);
        assert!(geoms.is_empty());
        assert!(
            Arc::ptr_eq(&stripped, &feature),
            "no-geometry node must be returned as-is"
        );
    }

    #[test]
    fn test_element_children_yields_elements() {
        let polygon = Arc::new(polygon_node(&[
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
        ]));
        let surface_member = elem(
            "gml:surfaceMember",
            vec![],
            vec![XmlChild::Element(Arc::clone(&polygon))],
        );
        let mut children = element_children(&surface_member);
        let child = children.next().expect("expected one child");
        assert_eq!(local_name(&child.name.0), "Polygon");
        assert!(children.next().is_none());
    }

    #[test]
    fn test_extract_geometries_via_element_child() {
        // Shell -> surfaceMember -> Polygon. extract_geometries must find the polygon.
        let polygon = Arc::new(polygon_node(&[
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ]));
        let surface_member = elem(
            "gml:surfaceMember",
            vec![],
            vec![XmlChild::Element(Arc::clone(&polygon))],
        );
        let shell = elem("gml:Shell", vec![], vec![elem_child(surface_member)]);
        let exterior = elem("gml:exterior", vec![], vec![elem_child(shell)]);
        let solid = elem(
            "gml:Solid",
            vec![("gml:id", "http://www.opengis.net/gml/3.2", "solid01")],
            vec![elem_child(exterior)],
        );
        let feature = Arc::new(elem(
            "bldg:Building",
            vec![],
            vec![elem_child(elem(
                "bldg:lod2Solid",
                vec![],
                vec![elem_child(solid)],
            ))],
        ));

        let (_, geoms) = extract_geometries(&feature);
        assert_eq!(geoms.len(), 1);
        assert_eq!(geoms[0].polygons.len(), 1);
    }
}
