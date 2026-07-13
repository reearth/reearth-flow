//! Pass-1 geometry extraction for the CityGML reader.
//!
//! Splits geometry out of a feature's attribute tree while streaming: each
//! `lod<N>…` / `tin` property is converted into a [`GeomNode`] and removed from
//! the tree, so coordinate text is never carried into the feature's attributes.
//! Fully-inline geometry is parsed to [`Euclidean3DGeometry`] here; containers are
//! left as [`Unresolved`] nodes and any `xlink:href` as a [`GeomNode::Ref`], both
//! assembled in pass 2. See `GEOMETRY_MAPPING.md` for the type mapping.

use std::sync::Arc;

use reearth_flow_geometry::coordinate::CoordinateFrame;
use reearth_flow_geometry::line_string::LineString3D;
use reearth_flow_geometry::point::Point3D;
use reearth_flow_geometry::polygon::Polygon3D;
use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
use reearth_flow_geometry::Euclidean3DGeometry;

use super::parser::{raw_gml_id, RawChild, RawNode};
use super::resolver::{
    FaceIds, GeomNode, GeomRegistry, GmlGeometryType, LeafIds, Role, Unresolved,
};
use super::srsname::parse_epsg;
use super::utils::{local_name, srs_name_attr, GML_NS_311_ID, GML_NS_ID};

/// A geometry carved from a feature, tagged with the LOD of the property it came
/// from (`None` for `tin`) and the `gml:id`s of its enclosing elements (nearest
/// first), used to attach it to the right feature when `flatten` hoists children.
pub(super) struct PendingGeom {
    pub(super) lod: Option<u8>,
    pub(super) node: GeomNode,
    pub(super) owner_ids: Vec<String>,
}

/// A `gml:id`-bearing ancestor, chained on the stack so the enclosing-id list is
/// materialized only where a geometry is actually found.
struct Owner<'a> {
    id: &'a str,
    parent: Option<&'a Owner<'a>>,
}

/// The enclosing `gml:id`s, nearest first.
fn owner_chain(mut owner: Option<&Owner>) -> Vec<String> {
    let mut ids = Vec::new();
    while let Some(o) = owner {
        ids.push(o.id.to_string());
        owner = o.parent;
    }
    ids
}

/// Strip every geometry property from `node`, returning the geometry-free
/// attribute tree and the geometries carved out of it. Geometries carrying a
/// `gml:id` are registered in `registry` as reference targets. `track_owners`
/// records each geometry's enclosing `gml:id`s, needed only when children will be
/// hoisted.
pub(super) fn split_geometry(
    node: &Arc<RawNode>,
    frame: &CoordinateFrame,
    registry: &mut GeomRegistry,
    track_owners: bool,
) -> (Arc<RawNode>, Vec<PendingGeom>) {
    let mut geoms = Vec::new();
    let stripped = strip(node, frame, None, registry, &mut geoms, track_owners);
    (stripped, geoms)
}

/// Recursively rebuild `node` without its geometry-property children, collecting
/// the parsed geometries into `geoms`. `owner` is the chain of enclosing
/// `gml:id`s above `node`.
fn strip(
    node: &Arc<RawNode>,
    frame: &CoordinateFrame,
    owner: Option<&Owner>,
    registry: &mut GeomRegistry,
    geoms: &mut Vec<PendingGeom>,
    track_owners: bool,
) -> Arc<RawNode> {
    let here = gml_id_ref(node).map(|id| Owner { id, parent: owner });
    let owner = here.as_ref().or(owner);
    let mut new_children: Option<Vec<RawChild>> = None;

    for (i, child) in node.children.iter().enumerate() {
        let RawChild::Element(e) = child else {
            if let Some(ref mut nc) = new_children {
                nc.push(child.clone());
            }
            continue;
        };

        let ln = local_name(&e.name.0);
        let lod = if ln == "tin" {
            Some(None)
        } else {
            extract_lod(ln).map(Some)
        };
        // A name match alone is not enough: keep an element that merely shares the
        // `lod<N>…` / `tin` naming but wraps no geometry, rather than stripping it.
        let lod = lod.filter(|_| is_geometry_property(e));

        if let Some(lod) = lod {
            if let Some(gnode) = property_geometry(e, frame, registry) {
                let owner_ids = if track_owners {
                    owner_chain(owner)
                } else {
                    Vec::new()
                };
                geoms.push(PendingGeom {
                    lod,
                    node: gnode,
                    owner_ids,
                });
            }
            if new_children.is_none() {
                new_children = Some(node.children[..i].to_vec());
            }
        } else {
            let stripped_child = strip(e, frame, owner, registry, geoms, track_owners);
            match new_children {
                None => {
                    if !Arc::ptr_eq(&stripped_child, e) {
                        let mut nc = node.children[..i].to_vec();
                        nc.push(RawChild::Element(stripped_child));
                        new_children = Some(nc);
                    }
                }
                Some(ref mut nc) => nc.push(RawChild::Element(stripped_child)),
            }
        }
    }

    match new_children {
        None => Arc::clone(node),
        Some(children) => Arc::new(RawNode {
            name: node.name.clone(),
            attrs: node.attrs.clone(),
            children,
            source_url: Arc::clone(&node.source_url),
        }),
    }
}

/// Error on a geometry-level `srsName` naming a CRS other than the file's: the
/// reader can't honour a per-geometry override, so the geometry is left
/// mis-referenced. A redundant restatement of the file CRS is silent.
fn check_local_srs(node: &RawNode, frame: &CoordinateFrame) {
    let Some(srs_name) = srs_name_attr(&node.attrs) else {
        return;
    };
    let file_epsg = match frame {
        CoordinateFrame::Crs(epsg) => Some(*epsg),
        _ => None,
    };
    if file_epsg.is_some() && parse_epsg(srs_name) == file_epsg {
        return;
    }
    tracing::error!(
        srs_name,
        "citygml geometry: local srsName is not honoured; geometry left in the file CRS"
    );
}

/// Borrow a node's `gml:id`, if any.
fn gml_id_ref(node: &RawNode) -> Option<&str> {
    node.attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "id" && (*ns == GML_NS_ID || *ns == GML_NS_311_ID))
        .map(|(_, v)| v.as_str())
}

/// Whether a `lod<N>…` / `tin`-named element actually wraps a geometry: its first
/// non-text child is a recognized GML geometry element or an `xlink:href`
/// reference. Mirrors [`property_geometry`]'s child selection.
fn is_geometry_property(prop: &RawNode) -> bool {
    for child in &prop.children {
        match child {
            RawChild::Ref(_) => return true,
            RawChild::Element(e) => return geometry_type(local_name(&e.name.0)).is_some(),
            RawChild::Text(_) => {}
        }
    }
    false
}

/// Parse the single geometry element (or `xlink:href`) inside a `lod<N>…` / `tin`
/// property into a [`GeomNode`].
fn property_geometry(
    prop: &RawNode,
    frame: &CoordinateFrame,
    registry: &mut GeomRegistry,
) -> Option<GeomNode> {
    for child in &prop.children {
        match child {
            RawChild::Ref(key) => return Some(GeomNode::Ref(key.clone())),
            RawChild::Element(e) => return geometry_node(e, frame, registry),
            RawChild::Text(_) => {}
        }
    }
    None
}

/// Convert a geometry element into a [`GeomNode`]: an inline leaf is parsed to a
/// concrete geometry, a container is left [`Unresolved`]. A node with a `gml:id`
/// is moved into `registry` and replaced by a [`GeomNode::Ref`].
fn geometry_node(
    node: &RawNode,
    frame: &CoordinateFrame,
    registry: &mut GeomRegistry,
) -> Option<GeomNode> {
    let Some(ty) = geometry_type(local_name(&node.name.0)) else {
        tracing::warn!(
            element = local_name(&node.name.0),
            "citygml geometry: unrecognized element, skipped"
        );
        return None;
    };
    check_local_srs(node, frame);
    let id = raw_gml_id(node);
    let file = node.source_url.as_str().to_string();
    let geom = if ty.is_inline() {
        let (geometry, faces) = build_leaf(node, &ty, frame)?;
        GeomNode::Resolved(geometry, LeafIds { file, faces })
    } else {
        GeomNode::Unresolved(Unresolved {
            ty,
            id: id.clone(),
            file,
            members: collect_members(node, frame, registry),
        })
    };
    Some(register(node, id, geom, registry))
}

/// Register a geometry as an `xlink` target if it has a `gml:id`, returning a
/// [`GeomNode::Ref`] in its place; otherwise return it unchanged.
fn register(
    node: &RawNode,
    id: Option<String>,
    geom: GeomNode,
    registry: &mut GeomRegistry,
) -> GeomNode {
    match id {
        Some(id) => {
            let key = (node.source_url.as_str().to_string(), id);
            registry.insert(key.clone(), geom);
            GeomNode::Ref(key)
        }
        None => geom,
    }
}

/// Collect a container's members, tagging each with the role of the property that
/// owns it.
fn collect_members(
    node: &RawNode,
    frame: &CoordinateFrame,
    registry: &mut GeomRegistry,
) -> Vec<(Role, GeomNode)> {
    let mut members = Vec::new();
    for prop in element_children(node) {
        let role = property_role(local_name(&prop.name.0));
        for child in &prop.children {
            match child {
                RawChild::Ref(key) => members.push((role, GeomNode::Ref(key.clone()))),
                RawChild::Element(inner) => {
                    if let Some(g) = geometry_node(inner, frame, registry) {
                        members.push((role, g));
                    }
                }
                RawChild::Text(_) => {}
            }
        }
    }
    members
}

/// Parse an inline geometry leaf into a concrete geometry and the per-face gml:ids
/// captured alongside it, dispatching on its type. Container types and unsupported
/// leaves yield `None`.
fn build_leaf(
    node: &RawNode,
    ty: &GmlGeometryType,
    frame: &CoordinateFrame,
) -> Option<(Euclidean3DGeometry, Vec<FaceIds>)> {
    match ty {
        GmlGeometryType::Point => build_point(node, frame),
        GmlGeometryType::LineString => build_line_string(node, frame),
        // TODO: gml:Curve support is deferred. Its segments may be arcs or splines,
        // which cannot be handled by sweeping control points into a straight chain.
        GmlGeometryType::Curve => {
            tracing::warn!("citygml geometry: gml:Curve is not supported yet, skipped");
            None
        }
        GmlGeometryType::LinearRing => build_ring_polygon(node, frame),
        GmlGeometryType::Polygon => build_polygon(node, frame),
        GmlGeometryType::TriangulatedSurface | GmlGeometryType::Tin => {
            build_triangulated(node, frame)
        }
        _ => None,
    }
}

/// Build a `Point` from the element's first coordinate; `None` if it carries none.
/// A point has no faces, so no gml:ids are captured.
fn build_point(
    node: &RawNode,
    frame: &CoordinateFrame,
) -> Option<(Euclidean3DGeometry, Vec<FaceIds>)> {
    let coords = gather_coords(node);
    coords.first().map(|&c| {
        (
            Euclidean3DGeometry::Point(Point3D::new(frame.clone(), c)),
            Vec::new(),
        )
    })
}

/// Build a `LineString` from the element's coordinates; `None` if it carries none.
/// A line has no faces, so no gml:ids are captured.
fn build_line_string(
    node: &RawNode,
    frame: &CoordinateFrame,
) -> Option<(Euclidean3DGeometry, Vec<FaceIds>)> {
    let coords = gather_coords(node);
    if coords.is_empty() {
        return None;
    }
    let line = Euclidean3DGeometry::LineString(LineString3D::from_coords(frame.clone(), coords));
    Some((line, Vec::new()))
}

/// Build a single-ring `Polygon` from a `LinearRing`'s coordinates; `None` if it
/// carries none. The one face's sole ring id is the `LinearRing`'s own gml:id.
fn build_ring_polygon(
    node: &RawNode,
    frame: &CoordinateFrame,
) -> Option<(Euclidean3DGeometry, Vec<FaceIds>)> {
    let exterior = gather_coords(node);
    if exterior.is_empty() {
        return None;
    }
    let polygon = Polygon3D::from_rings(frame.clone(), exterior, Vec::<Vec<[f64; 3]>>::new());
    let face = FaceIds {
        surface: None,
        rings: vec![raw_gml_id(node)],
    };
    Some((Euclidean3DGeometry::Polygon(Box::new(polygon)), vec![face]))
}

/// Build a `Polygon` from an element's exterior and interior ring properties.
fn build_polygon(
    node: &RawNode,
    frame: &CoordinateFrame,
) -> Option<(Euclidean3DGeometry, Vec<FaceIds>)> {
    let (polygon, face) = polygon_from_rings(node, frame)?;
    Some((Euclidean3DGeometry::Polygon(Box::new(polygon)), vec![face]))
}

/// Build a `TriangularMesh` from a `TriangulatedSurface`/`Tin`'s triangle patches,
/// taking the first three coordinates of each; `None` if it has no patches. One
/// `FaceIds` per triangle, in triangle order: each face's surface id the whole
/// surface's own gml:id (a texture drapes the surface, not a single triangle) and
/// its sole ring id the triangle's exterior `LinearRing` gml:id.
fn build_triangulated(
    node: &RawNode,
    frame: &CoordinateFrame,
) -> Option<(Euclidean3DGeometry, Vec<FaceIds>)> {
    let surface = raw_gml_id(node);
    let mut soup: Vec<[f64; 3]> = Vec::new();
    let mut face_ids: Vec<FaceIds> = Vec::new();
    for prop in element_children(node) {
        if matches!(local_name(&prop.name.0), "trianglePatches" | "patches") {
            for triangle in element_children(prop) {
                let ring = gather_coords(triangle);
                if ring.len() >= 3 {
                    soup.extend_from_slice(&ring[..3]);
                    face_ids.push(FaceIds {
                        surface: surface.clone(),
                        rings: vec![triangle_ring_id(triangle)],
                    });
                }
            }
        }
    }
    if soup.is_empty() {
        return None;
    }
    let mesh = Euclidean3DGeometry::TriangularMesh(Box::new(TriangularMesh3D::from_soup(
        frame.clone(),
        soup,
    )));
    Some((mesh, face_ids))
}

/// The gml:id of a triangle patch's exterior `LinearRing`, if any.
fn triangle_ring_id(triangle: &RawNode) -> Option<String> {
    element_children(triangle)
        .find(|e| local_name(&e.name.0) == "exterior")
        .and_then(|ext| element_children(ext).find(|e| local_name(&e.name.0) == "LinearRing"))
        .and_then(raw_gml_id)
}

/// Build a polygon from an element's `exterior` / `interior` ring properties,
/// capturing the element's own gml:id as the face's surface id and each
/// `LinearRing`'s gml:id in exterior-first, holes-next order.
fn polygon_from_rings(node: &RawNode, frame: &CoordinateFrame) -> Option<(Polygon3D, FaceIds)> {
    let mut exterior: Option<Ring> = None;
    let mut interiors: Vec<Ring> = Vec::new();
    for prop in element_children(node) {
        match local_name(&prop.name.0) {
            "exterior" => {
                let ring = read_ring(prop);
                if !ring.coords.is_empty() {
                    exterior = Some(ring);
                }
            }
            "interior" => {
                let ring = read_ring(prop);
                if !ring.coords.is_empty() {
                    interiors.push(ring);
                }
            }
            _ => {}
        }
    }
    let exterior = exterior?;
    let mut rings = Vec::with_capacity(1 + interiors.len());
    rings.push(exterior.id);
    let interior_coords: Vec<Vec<[f64; 3]>> = interiors
        .into_iter()
        .map(|ring| {
            rings.push(ring.id);
            ring.coords
        })
        .collect();
    let polygon = Polygon3D::from_rings(frame.clone(), exterior.coords, interior_coords);
    let face = FaceIds {
        surface: raw_gml_id(node),
        rings,
    };
    Some((polygon, face))
}

/// A single ring's coordinates and the enclosing `LinearRing`'s gml:id.
struct Ring {
    id: Option<String>,
    coords: Vec<[f64; 3]>,
}

/// Read a `exterior` / `interior` ring property: its coordinates and the enclosing
/// `LinearRing`'s gml:id.
fn read_ring(prop: &RawNode) -> Ring {
    let id = element_children(prop)
        .find(|e| local_name(&e.name.0) == "LinearRing")
        .and_then(raw_gml_id);
    Ring {
        id,
        coords: gather_coords(prop),
    }
}

/// Collect every coordinate under `node`, descending through property and ring
/// wrappers to the `posList` / `pos` leaves.
fn gather_coords(node: &RawNode) -> Vec<[f64; 3]> {
    let mut out = Vec::new();
    gather_coords_into(node, &mut out);
    out
}

/// Collect every coordinate under `node` into `out`, descending through wrappers
/// to the `posList` / `pos` leaves.
fn gather_coords_into(node: &RawNode, out: &mut Vec<[f64; 3]>) {
    for child in element_children(node) {
        match local_name(&child.name.0) {
            "posList" => out.extend(parse_pos_list(text_content(child))),
            "pos" => out.extend(parse_pos(text_content(child))),
            _ => gather_coords_into(child, out),
        }
    }
}

/// Parse whitespace-separated ordinates, returning `None` if any token is not a
/// valid number.
fn parse_ordinates(text: &str) -> Option<Vec<f64>> {
    text.split_whitespace()
        .map(|s| s.parse::<f64>().ok())
        .collect()
}

/// Parse a `gml:posList` into ordinate triples, in the source's own axis order.
/// A list with any unparseable token, or whose ordinate count is not a multiple
/// of 3, is skipped whole rather than producing misaligned coordinates.
fn parse_pos_list(text: &str) -> Vec<[f64; 3]> {
    let Some(values) = parse_ordinates(text) else {
        tracing::warn!("citygml geometry: invalid gml:posList content, skipped");
        return Vec::new();
    };
    if !values.len().is_multiple_of(3) {
        tracing::warn!("citygml geometry: gml:posList length not a multiple of 3, skipped");
        return Vec::new();
    }
    values.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

/// Parse a single `gml:pos` into one ordinate triple, in the source's own axis order.
fn parse_pos(text: &str) -> Option<[f64; 3]> {
    let Some(values) = parse_ordinates(text) else {
        tracing::warn!("citygml geometry: invalid gml:pos content, skipped");
        return None;
    };
    if values.len() != 3 {
        return None;
    }
    Some([values[0], values[1], values[2]])
}

/// Extract the LOD digit from a `lod<N>…` property name.
fn extract_lod(local: &str) -> Option<u8> {
    if local.len() >= 4 && local.starts_with("lod") {
        local.chars().nth(3)?.to_digit(10).map(|d| d as u8)
    } else {
        None
    }
}

/// The GML property a member fills within its parent.
fn property_role(local: &str) -> Role {
    match local {
        "exterior" => Role::Exterior,
        "interior" => Role::Interior,
        _ => Role::Member,
    }
}

/// Map a geometry element's local name to its type.
fn geometry_type(local: &str) -> Option<GmlGeometryType> {
    Some(match local {
        "Point" => GmlGeometryType::Point,
        "LineString" => GmlGeometryType::LineString,
        "Curve" => GmlGeometryType::Curve,
        "LinearRing" => GmlGeometryType::LinearRing,
        // A surface patch is a polygon: welded into its enclosing surface mesh.
        "Polygon" | "PolygonPatch" | "Rectangle" | "Triangle" => GmlGeometryType::Polygon,
        "Surface" => GmlGeometryType::Surface,
        "PolyhedralSurface" => GmlGeometryType::PolyhedralSurface,
        "TriangulatedSurface" => GmlGeometryType::TriangulatedSurface,
        "Tin" => GmlGeometryType::Tin,
        "MultiPoint" => GmlGeometryType::MultiPoint,
        "OrientableCurve" => GmlGeometryType::OrientableCurve,
        "CompositeCurve" => GmlGeometryType::CompositeCurve,
        "MultiCurve" => GmlGeometryType::MultiCurve,
        "Ring" => GmlGeometryType::Ring,
        "OrientableSurface" => GmlGeometryType::OrientableSurface,
        "CompositeSurface" => GmlGeometryType::CompositeSurface,
        "MultiSurface" => GmlGeometryType::MultiSurface,
        "Shell" => GmlGeometryType::Shell,
        "Solid" => GmlGeometryType::Solid,
        "CompositeSolid" => GmlGeometryType::CompositeSolid,
        "MultiSolid" => GmlGeometryType::MultiSolid,
        "MultiGeometry" => GmlGeometryType::MultiGeometry,
        "GeometricComplex" => GmlGeometryType::GeometricComplex,
        // TODO: ImplicitGeometry support is deferred. Placing it correctly needs
        // its transformationMatrix/referencePoint applied to the relativeGeometry
        // prototype; emitting the untransformed prototype puts geometry at the
        // wrong position, so it is dropped rather than mis-placed for now.
        _ => return None,
    })
}

/// Iterate a node's element children.
fn element_children(node: &RawNode) -> impl Iterator<Item = &RawNode> {
    node.children.iter().filter_map(|c| match c {
        RawChild::Element(e) => Some(e.as_ref()),
        _ => None,
    })
}

/// The first text child of a node, or the empty string.
fn text_content(node: &RawNode) -> &str {
    node.children
        .iter()
        .find_map(|c| match c {
            RawChild::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citygml_parser::parser::{Parser, ParserOutput};
    use crate::citygml_parser::resolver::resolve_root_bare;
    use url::Url;

    /// Parse one CityGML feature wrapping `inner`, returning its stripped attribute
    /// tree, its carved geometries, and the geometry registry.
    fn parse_one_feature(inner: &str) -> (Arc<RawNode>, Vec<PendingGeom>, GeomRegistry) {
        let xml = format!(
            r#"<core:CityModel
                 xmlns:core="http://www.opengis.net/citygml/3.0"
                 xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
                 xmlns:con="http://www.opengis.net/citygml/construction/3.0"
                 xmlns:dem="http://www.opengis.net/citygml/relief/3.0"
                 xmlns:urc="https://www.geospatial.jp/iur/urc/4.0"
                 xmlns:gml="http://www.opengis.net/gml/3.2"
                 xmlns:xlink="http://www.w3.org/1999/xlink">
               <core:cityObjectMember><bldg:Building gml:id="b1">{inner}</bldg:Building></core:cityObjectMember>
             </core:CityModel>"#
        );
        let url = Url::parse("file:///test.gml").unwrap();
        let mut parser = Parser::new();
        parser.parse(xml.as_bytes(), &url).unwrap();
        let ParserOutput {
            pending,
            geom_registry,
            ..
        } = parser.finish();
        let feature = pending.into_iter().next().expect("one feature");
        (feature.root, feature.geoms, geom_registry)
    }

    /// Parse one CityGML feature wrapping `inner`, returning its carved geometries
    /// and the geometry registry.
    fn parse_one(inner: &str) -> (Vec<PendingGeom>, GeomRegistry) {
        let (_root, geoms, registry) = parse_one_feature(inner);
        (geoms, registry)
    }

    const POLYGON: &str = r#"<gml:Polygon><gml:exterior><gml:LinearRing>
        <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
      </gml:LinearRing></gml:exterior></gml:Polygon>"#;

    fn collection_len(geometry: &Euclidean3DGeometry) -> usize {
        match geometry {
            Euclidean3DGeometry::Collection(c) => c.len(),
            other => panic!("expected a collection, got {other:?}"),
        }
    }

    /// The sole member of a one-member collection.
    fn only_member(geometry: &Euclidean3DGeometry) -> &Euclidean3DGeometry {
        match geometry {
            Euclidean3DGeometry::Collection(c) => {
                assert_eq!(c.len(), 1, "expected exactly one member");
                &c.members()[0]
            }
            other => panic!("expected a collection, got {other:?}"),
        }
    }

    #[test]
    fn inline_multisurface_resolves_to_collection() {
        let (geoms, registry) = parse_one(&format!(
            "<bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>{POLYGON}</gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>"
        ));
        assert_eq!(geoms.len(), 1);
        assert_eq!(geoms[0].lod, Some(2));
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        assert_eq!(collection_len(&geometry), 1);
    }

    #[test]
    fn inline_solid_resolves_to_solid() {
        let (geoms, registry) = parse_one(&format!(
            "<bldg:lod2Solid><gml:Solid><gml:exterior><gml:Shell><gml:surfaceMember>{POLYGON}</gml:surfaceMember></gml:Shell></gml:exterior></gml:Solid></bldg:lod2Solid>"
        ));
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        assert!(matches!(geometry, Euclidean3DGeometry::Solid(_)));
    }

    #[test]
    fn triangulated_surface_resolves_to_triangular_mesh() {
        let (geoms, registry) = parse_one(
            r#"<bldg:lod1MultiSurface><gml:TriangulatedSurface><gml:trianglePatches>
                 <gml:Triangle><gml:exterior><gml:LinearRing>
                   <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                 </gml:LinearRing></gml:exterior></gml:Triangle>
               </gml:trianglePatches></gml:TriangulatedSurface></bldg:lod1MultiSurface>"#,
        );
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        assert!(matches!(geometry, Euclidean3DGeometry::TriangularMesh(_)));
    }

    #[test]
    fn geometry_tagged_with_enclosing_gml_ids() {
        // The wrapping feature is `b1`; a nested WallSurface `wall1` owns the geometry.
        let (geoms, _) = parse_one(&format!(
            r#"<core:boundary><con:WallSurface gml:id="wall1">
                 <core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>{POLYGON}</gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface>
               </con:WallSurface></core:boundary>"#
        ));
        assert_eq!(geoms.len(), 1);
        assert_eq!(
            geoms[0].owner_ids,
            vec!["wall1".to_string(), "b1".to_string()]
        );
    }

    #[test]
    fn gml_id_geometry_registers_and_xlink_resolves() {
        let inline = POLYGON.replacen("<gml:Polygon>", r#"<gml:Polygon gml:id="p1">"#, 1);
        let (geoms, registry) = parse_one(&format!(
            r##"<bldg:lod2MultiSurface><gml:MultiSurface>
                 <gml:surfaceMember>{inline}</gml:surfaceMember>
                 <gml:surfaceMember xlink:href="#p1"/>
               </gml:MultiSurface></bldg:lod2MultiSurface>"##
        ));
        assert!(registry.contains_key(&("file:///test.gml".to_string(), "p1".to_string())));
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        assert_eq!(collection_len(&geometry), 2);
    }

    // Single-geometry cases resolved via resolve_root. The two-face meshes are
    // built from TA and TB, two triangles sharing edge B–C.

    /// Two triangles sharing edge B–C.
    const TA: &str = "<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>1 2 0 3 2 0 1 4 5 1 2 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>";
    const TB: &str = "<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>3 2 0 3 4 5 1 4 5 3 2 0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>";

    #[test]
    fn case01_multicurve_linestring() {
        let (geoms, registry) = parse_one(
            r#"<core:lod0MultiCurve><gml:MultiCurve><gml:curveMember><gml:LineString>
                 <gml:posList>0 1 0 2 3 0 4 5 0</gml:posList>
               </gml:LineString></gml:curveMember></gml:MultiCurve></core:lod0MultiCurve>"#,
        );
        assert_eq!(geoms.len(), 1);
        assert_eq!(geoms[0].lod, Some(0));
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        match only_member(&geometry) {
            Euclidean3DGeometry::LineString(ls) => {
                assert_eq!(ls.coords().len(), 3);
                assert_eq!(ls.coords()[0], [0.0, 1.0, 0.0]);
            }
            other => panic!("expected LineString, got {other:?}"),
        }
    }

    #[test]
    fn case02_multisurface_polygon() {
        let (geoms, registry) = parse_one(&format!(
            "<core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>{TA}</gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface>"
        ));
        assert_eq!(geoms.len(), 1);
        assert_eq!(geoms[0].lod, Some(2));
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        match only_member(&geometry) {
            Euclidean3DGeometry::Polygon(p) => {
                assert_eq!(p.exterior().len(), 4);
                assert_eq!(p.exterior()[0], [1.0, 2.0, 0.0]);
                assert_eq!(p.interiors().count(), 0);
            }
            other => panic!("expected Polygon, got {other:?}"),
        }
    }

    #[test]
    fn case03_polygon_with_hole() {
        let (geoms, registry) = parse_one(
            r#"<core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                 <gml:Polygon>
                   <gml:exterior><gml:LinearRing><gml:posList>0 0 0 4 0 0 4 4 0 0 4 0 0 0 0</gml:posList></gml:LinearRing></gml:exterior>
                   <gml:interior><gml:LinearRing><gml:posList>1 1 0 3 1 0 3 3 0 1 1 0</gml:posList></gml:LinearRing></gml:interior>
                 </gml:Polygon>
               </gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface>"#,
        );
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        match only_member(&geometry) {
            Euclidean3DGeometry::Polygon(p) => {
                assert_eq!(p.exterior().len(), 5);
                let holes: Vec<_> = p.interiors().collect();
                assert_eq!(holes.len(), 1);
                assert_eq!(holes[0].len(), 4);
            }
            other => panic!("expected Polygon, got {other:?}"),
        }
    }

    #[test]
    fn case04_multisurface_compositesurface_xlink() {
        let (geoms, registry) = parse_one(&format!(
            r##"<core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                  <gml:CompositeSurface>
                    <gml:surfaceMember xlink:href="#cp1"/>
                    <gml:surfaceMember xlink:href="#cp2"/>
                  </gml:CompositeSurface>
                </gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface>
                <core:lod2MultiSurface><gml:MultiSurface>
                  <gml:surfaceMember>{cp1}</gml:surfaceMember>
                  <gml:surfaceMember>{cp2}</gml:surfaceMember>
                </gml:MultiSurface></core:lod2MultiSurface>"##,
            cp1 = TA.replacen("<gml:Polygon>", r#"<gml:Polygon gml:id="cp1">"#, 1),
            cp2 = TB.replacen("<gml:Polygon>", r#"<gml:Polygon gml:id="cp2">"#, 1),
        ));
        assert_eq!(geoms.len(), 2);
        // geoms[0] is the CompositeSurface-in-MultiSurface: the two xlinked polygons
        // resolved and welded into one two-face mesh.
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        match only_member(&geometry) {
            Euclidean3DGeometry::PolygonMesh(m) => assert_eq!(m.num_faces(), 2),
            other => panic!("expected PolygonMesh, got {other:?}"),
        }
    }

    #[test]
    fn case05_multisurface_surfacemember_xlink() {
        let (geoms, registry) = parse_one(&format!(
            r##"<core:lod2MultiSurface><gml:MultiSurface>
                  <gml:surfaceMember xlink:href="#tp1"/>
                </gml:MultiSurface></core:lod2MultiSurface>
                <core:lod2MultiSurface><gml:MultiSurface>
                  <gml:surfaceMember>{tp1}</gml:surfaceMember>
                </gml:MultiSurface></core:lod2MultiSurface>"##,
            tp1 = TA.replacen("<gml:Polygon>", r#"<gml:Polygon gml:id="tp1">"#, 1),
        ));
        assert_eq!(geoms.len(), 2);
        let geometry = resolve_root_bare(&geoms[0].node, &registry).unwrap();
        match only_member(&geometry) {
            Euclidean3DGeometry::Polygon(p) => assert_eq!(p.exterior().len(), 4),
            other => panic!("expected Polygon (xlink resolved), got {other:?}"),
        }
    }

    #[test]
    fn case06_solid_shell_polygons() {
        let (geoms, registry) = parse_one(&format!(
            "<core:lod1Solid><gml:Solid><gml:exterior><gml:Shell>
               <gml:surfaceMember>{TA}</gml:surfaceMember>
               <gml:surfaceMember>{TB}</gml:surfaceMember>
             </gml:Shell></gml:exterior></gml:Solid></core:lod1Solid>"
        ));
        assert_eq!(geoms.len(), 1);
        assert_eq!(geoms[0].lod, Some(1));
        match resolve_root_bare(&geoms[0].node, &registry).unwrap() {
            Euclidean3DGeometry::Solid(s) => {
                assert_eq!(s.exterior().num_faces(), 2);
                assert!(s.interiors().is_empty());
            }
            other => panic!("expected Solid, got {other:?}"),
        }
    }

    #[test]
    fn case07_solid_shell_compositesurface() {
        let (geoms, registry) = parse_one(&format!(
            "<core:lod2Solid><gml:Solid><gml:exterior><gml:Shell><gml:surfaceMember>
               <gml:CompositeSurface>
                 <gml:surfaceMember>{TA}</gml:surfaceMember>
                 <gml:surfaceMember>{TB}</gml:surfaceMember>
               </gml:CompositeSurface>
             </gml:surfaceMember></gml:Shell></gml:exterior></gml:Solid></core:lod2Solid>"
        ));
        assert_eq!(geoms[0].lod, Some(2));
        // The CompositeSurface shell body welds its two polygons into the solid's
        // one exterior shell.
        match resolve_root_bare(&geoms[0].node, &registry).unwrap() {
            Euclidean3DGeometry::Solid(s) => assert_eq!(s.exterior().num_faces(), 2),
            other => panic!("expected Solid, got {other:?}"),
        }
    }

    #[test]
    fn case08_triangulated_surface() {
        let (geoms, registry) = parse_one(
            r#"<dem:tin><gml:TriangulatedSurface><gml:patches>
                 <gml:Triangle><gml:exterior><gml:LinearRing><gml:posList>1 2 0 3 2 0 1 4 5 1 2 0</gml:posList></gml:LinearRing></gml:exterior></gml:Triangle>
                 <gml:Triangle><gml:exterior><gml:LinearRing><gml:posList>3 2 0 3 4 5 1 4 5 3 2 0</gml:posList></gml:LinearRing></gml:exterior></gml:Triangle>
               </gml:patches></gml:TriangulatedSurface></dem:tin>"#,
        );
        assert_eq!(geoms.len(), 1);
        assert_eq!(geoms[0].lod, None);
        match resolve_root_bare(&geoms[0].node, &registry).unwrap() {
            Euclidean3DGeometry::TriangularMesh(m) => {
                assert_eq!(m.num_triangles(), 2);
                // Edge B–C is shared, so the two triangles dedup to four vertices.
                assert_eq!(m.vertices().len(), 4);
            }
            other => panic!("expected TriangularMesh, got {other:?}"),
        }
    }

    #[test]
    fn case09_point() {
        let (geoms, registry) = parse_one(
            r#"<urc:lod0Geometry><gml:Point><gml:pos>1 2 5</gml:pos></gml:Point></urc:lod0Geometry>"#,
        );
        assert_eq!(geoms.len(), 1);
        assert_eq!(geoms[0].lod, Some(0));
        match resolve_root_bare(&geoms[0].node, &registry).unwrap() {
            Euclidean3DGeometry::Point(p) => assert_eq!(p.position(), [1.0, 2.0, 5.0]),
            other => panic!("expected Point, got {other:?}"),
        }
    }

    #[test]
    fn poslist_with_invalid_token_is_dropped_whole_not_shifted() {
        // A malformed ordinate must reject its whole ring rather than dropping the
        // token and misaligning every following coordinate. The valid member
        // survives; the one with a bad token is dropped entirely.
        let (geoms, registry) = parse_one(&format!(
            "<bldg:lod2MultiSurface><gml:MultiSurface>\
               <gml:surfaceMember>{POLYGON}</gml:surfaceMember>\
               <gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing>\
                 <gml:posList>0 0 0 1 0 0 0 1 bad 0 0 0</gml:posList>\
               </gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>\
             </gml:MultiSurface></bldg:lod2MultiSurface>"
        ));
        assert_eq!(
            collection_len(&resolve_root_bare(&geoms[0].node, &registry).unwrap()),
            1
        );
    }

    #[test]
    fn poslist_length_not_multiple_of_three_is_dropped_whole() {
        // An 8-ordinate list (e.g. a 2D posList, or a data error) is not a multiple
        // of 3; it must drop the whole ring rather than truncate into misaligned
        // coordinates. The valid member survives; the malformed one is dropped.
        let (geoms, registry) = parse_one(&format!(
            "<bldg:lod2MultiSurface><gml:MultiSurface>\
               <gml:surfaceMember>{POLYGON}</gml:surfaceMember>\
               <gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing>\
                 <gml:posList>0 0 1 0 1 1 0 0</gml:posList>\
               </gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>\
             </gml:MultiSurface></bldg:lod2MultiSurface>"
        ));
        assert_eq!(
            collection_len(&resolve_root_bare(&geoms[0].node, &registry).unwrap()),
            1
        );
    }

    #[test]
    fn pos_with_invalid_token_is_skipped() {
        // The bad pos yields no coordinate, so the Point is dropped at parse time
        // and no geometry is carved from the feature.
        let (geoms, _registry) = parse_one(
            r#"<urc:lod0Geometry><gml:Point><gml:pos>1 x 5</gml:pos></gml:Point></urc:lod0Geometry>"#,
        );
        assert!(geoms.is_empty());
    }

    #[test]
    fn lod_named_non_geometry_element_is_kept_as_attribute() {
        // An element whose name matches the lod<N> pattern but wraps no geometry
        // must stay in the attribute tree, not be stripped and silently lost.
        let (root, geoms, _registry) = parse_one_feature("<bldg:lod1Note>keep me</bldg:lod1Note>");
        assert!(geoms.is_empty(), "no geometry should be carved");
        assert!(
            root.children.iter().any(|c| matches!(
                c,
                RawChild::Element(e) if local_name(&e.name.0) == "lod1Note"
            )),
            "the lod-named non-geometry element should be preserved"
        );
    }

    #[test]
    fn linear_ring_gml_ids_are_captured() {
        // A Polygon surface with a gml:id whose exterior and interior LinearRings
        // each carry a gml:id: the surface id and both ring ids are captured, in
        // exterior-first, holes-next order. The polygon is registered under its own
        // gml:id, so its captured ids live on the registry entry.
        let (_geoms, registry) = parse_one(
            r#"<bldg:lod0FootPrint><gml:Polygon gml:id="surf1">
                 <gml:exterior><gml:LinearRing gml:id="ring_ext"><gml:posList>0 0 0 4 0 0 4 4 0 0 4 0 0 0 0</gml:posList></gml:LinearRing></gml:exterior>
                 <gml:interior><gml:LinearRing gml:id="ring_int"><gml:posList>1 1 0 3 1 0 3 3 0 1 1 0</gml:posList></gml:LinearRing></gml:interior>
               </gml:Polygon></bldg:lod0FootPrint>"#,
        );
        let node = registry
            .get(&("file:///test.gml".to_string(), "surf1".to_string()))
            .expect("polygon registered under its gml:id");
        let GeomNode::Resolved(_, ids) = node else {
            panic!("expected a resolved leaf");
        };
        assert_eq!(ids.faces.len(), 1);
        assert_eq!(ids.faces[0].surface.as_deref(), Some("surf1"));
        assert_eq!(
            ids.faces[0].rings,
            vec![Some("ring_ext".to_string()), Some("ring_int".to_string())]
        );
    }

    #[test]
    fn gml31_namespace_ids_are_captured() {
        // CityGML 2.0 uses the GML 3.1 namespace (`http://www.opengis.net/gml`);
        // gml:id capture must not be limited to GML 3.2.
        let xml = r#"<core:CityModel
             xmlns:core="http://www.opengis.net/citygml/2.0"
             xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
             xmlns:gml="http://www.opengis.net/gml"
             xmlns:xlink="http://www.w3.org/1999/xlink">
           <core:cityObjectMember><bldg:Building gml:id="b1">
             <bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
               <gml:Polygon gml:id="poly1"><gml:exterior><gml:LinearRing gml:id="ring1">
                 <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
               </gml:LinearRing></gml:exterior></gml:Polygon>
             </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>
           </bldg:Building></core:cityObjectMember>
         </core:CityModel>"#;
        let mut parser = Parser::new();
        parser
            .parse(xml.as_bytes(), &Url::parse("file:///test.gml").unwrap())
            .unwrap();
        let ParserOutput { geom_registry, .. } = parser.finish();
        let node = geom_registry
            .get(&("file:///test.gml".to_string(), "poly1".to_string()))
            .expect("polygon registered under its GML 3.1 gml:id");
        let GeomNode::Resolved(_, ids) = node else {
            panic!("expected a resolved leaf");
        };
        assert_eq!(ids.faces[0].surface.as_deref(), Some("poly1"));
        assert_eq!(ids.faces[0].rings, vec![Some("ring1".to_string())]);
    }

    #[test]
    fn ring_gml_ids_captured_on_multisurface_member() {
        // The common PLATEAU shape: a surfaceMember Polygon with no gml:id of its
        // own, whose exterior LinearRing carries one. The member leaf keeps the ring
        // id with no surface id.
        let (geoms, _registry) = parse_one(
            r#"<bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>
                 <gml:Polygon><gml:exterior><gml:LinearRing gml:id="ring1">
                   <gml:posList>0 0 0 1 0 0 0 1 0 0 0 0</gml:posList>
                 </gml:LinearRing></gml:exterior></gml:Polygon>
               </gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>"#,
        );
        let GeomNode::Unresolved(container) = &geoms[0].node else {
            panic!("expected an unresolved container");
        };
        let GeomNode::Resolved(_, ids) = &container.members[0].1 else {
            panic!("expected a resolved member leaf");
        };
        assert_eq!(ids.faces.len(), 1);
        assert_eq!(ids.faces[0].surface, None);
        assert_eq!(ids.faces[0].rings, vec![Some("ring1".to_string())]);
    }

    #[test]
    fn curve_is_deferred_and_skipped() {
        let (geoms, registry) = parse_one(
            r#"<bldg:lod0MultiSurface><gml:MultiCurve><gml:curveMember>
                 <gml:Curve><gml:segments><gml:LineStringSegment>
                   <gml:posList>0 0 0 1 0 0</gml:posList>
                 </gml:LineStringSegment></gml:segments></gml:Curve>
               </gml:curveMember></gml:MultiCurve></bldg:lod0MultiSurface>"#,
        );
        // The MultiCurve survives but its sole Curve member is dropped, leaving an
        // empty collection.
        assert_eq!(
            collection_len(&resolve_root_bare(&geoms[0].node, &registry).unwrap()),
            0
        );
    }
}
