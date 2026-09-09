//! Pass-2 geometry resolution for the CityGML reader.
//!
//! Pass 1 parses every geometry that is guaranteed to be fully inline directly
//! into [`Euclidean3DGeometry`]; the remaining reference-bearing containers are
//! left as [`Unresolved`] nodes and assembled here once every `gml:id` is known.
//! See `GEOMETRY_MAPPING.md` for the CityGML-to-Flow type mapping.

use std::collections::{HashMap, HashSet};

use reearth_flow_geometry::collection::Collection3D;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::line_string::LineString3D;
use reearth_flow_geometry::polygon::Polygon3D;
use reearth_flow_geometry::polygon_mesh::PolygonMesh3D;
use reearth_flow_geometry::solid::{Shell, Solid};
use reearth_flow_geometry::Euclidean3DGeometry;

use super::appearance::AppearanceIndex;
use super::malformation::Malformation;
use super::parser::RawNodeKey;
use super::utils::frame_for;

/// A node in a CityGML geometry tree during pass-2 resolution: a geometry already
/// built in pass 1, a reference still to be looked up, or a container still to be
/// assembled from its members.
#[derive(Debug)]
pub enum GeomNode {
    /// A geometry fully parsed in pass 1: an inline leaf, or a container that
    /// turned out to be reference-free and was assembled eagerly. Carries the
    /// per-face gml:ids captured from the source, for binding appearance later.
    Resolved(Euclidean3DGeometry, LeafIds),
    /// An `xlink:href`, resolved against the geometry registry in pass 2.
    Ref(RawNodeKey),
    /// A container awaiting assembly from its resolved members.
    Unresolved(Unresolved),
}

/// The gml:ids captured for one face of a built geometry, aligned with the face's
/// stored ring layout, used to bind CityGML appearance by gml:id in a later pass.
#[derive(Debug)]
pub(super) struct FaceIds {
    /// The enclosing surface element's gml:id (e.g. a `gml:Polygon`), if any; the
    /// target of a material or a texture.
    pub(super) surface: Option<String>,
    /// Ring gml:ids, exterior first then holes, each `None` when the ring carried
    /// no gml:id; the target of texture coordinates.
    pub(super) rings: Vec<Option<String>>,
}

/// Per-face gml:ids for a built leaf geometry, in the leaf's face order, scoped to
/// the source file every id belongs to. An appearance targeting these ids must be
/// declared in the same file.
#[derive(Debug)]
pub struct LeafIds {
    /// The source file URL the face ids are scoped to.
    pub(super) file: String,
    /// One entry per face, in the leaf's face order; empty for leaves with no faces
    /// (`Point`, `LineString`).
    pub(super) faces: Vec<FaceIds>,
}

/// A geometry container held until pass 2, when its members are resolved and
/// folded into a single [`Euclidean3DGeometry`].
#[derive(Debug)]
pub struct Unresolved {
    /// The CityGML type, selecting which construction site assembles this node.
    pub(super) ty: GmlGeometryType,
    /// The `gml:id`, if any, under which this node is registered as a reference
    /// target.
    pub(super) id: Option<String>,
    /// The source file URL this container was parsed from, scoping its `id` for
    /// appearance binding.
    pub(super) file: String,
    /// The child geometries, each tagged with the GML property that owns it.
    pub(super) members: Vec<(Role, GeomNode)>,
}

/// The GML property a member fills within its parent container.
///
/// Only `Solid` distinguishes its roles; every other container has a single member
/// role, for which [`Role::Member`] stands in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Role {
    /// The sole member role of aggregates and single-base wrappers (`pointMember`,
    /// `curveMember`, `surfaceMember`, `solidMember`, `geometryMember`, `element`,
    /// `baseCurve`, `baseSurface`).
    Member,
    /// A `Solid`'s outer boundary shell (`exterior`).
    Exterior,
    /// A `Solid`'s inner void boundary shell (`interior`).
    Interior,
}

/// Registry of geometries reachable by `xlink:href`, keyed by `(file_url, gml:id)`.
pub(super) type GeomRegistry = HashMap<RawNodeKey, GeomNode>;

/// The CityGML / GML geometry types the reader recognizes.
///
/// The types split into ones that are always fully inline, parsed whole in pass 1,
/// and reference-bearing containers assembled in pass 2; see
/// [`GmlGeometryType::is_inline`].
#[derive(Debug)]
pub(super) enum GmlGeometryType {
    /// A single position. Inline.
    Point,
    /// A chain of positions. Inline.
    LineString,
    /// A chain of inline curve segments. Inline. Parsing is deferred; see
    /// `build_leaf`.
    Curve,
    /// A single closed ring. Inline.
    LinearRing,
    /// An exterior ring with optional interior holes. Inline.
    Polygon,
    /// Inline polygon patches. Inline.
    Surface,
    /// Inline polygon patches forming a connected surface. Inline.
    PolyhedralSurface,
    /// Inline triangle patches. Inline.
    TriangulatedSurface,
    /// Inline triangle patches with control points. Inline.
    Tin,
    /// A collection of points.
    MultiPoint,
    /// A single base curve, possibly reversed.
    OrientableCurve,
    /// Curve members forming one connected curve.
    CompositeCurve,
    /// A collection of curves.
    MultiCurve,
    /// Curve members concatenated into one closed ring.
    Ring,
    /// A single base surface, possibly flipped.
    OrientableSurface,
    /// Surface members forming one connected surface.
    CompositeSurface,
    /// A collection of surfaces.
    MultiSurface,
    /// Surface members bounding a solid.
    Shell,
    /// An exterior shell with optional interior void shells.
    Solid,
    /// Solid members forming one connected solid.
    CompositeSolid,
    /// A collection of solids.
    MultiSolid,
    /// A heterogeneous collection of geometries.
    MultiGeometry,
    /// A heterogeneous collection of geometric primitives.
    GeometricComplex,
}

impl GmlGeometryType {
    /// Whether this type is always fully buildable while streaming, i.e. it can
    /// carry no `xlink:href` in any child property and every child type is itself
    /// inline. These are parsed in pass 1; every other type defers to pass 2.
    pub(super) fn is_inline(&self) -> bool {
        matches!(
            self,
            GmlGeometryType::Point
                | GmlGeometryType::LineString
                | GmlGeometryType::Curve
                | GmlGeometryType::LinearRing
                | GmlGeometryType::Polygon
                | GmlGeometryType::TriangulatedSurface
                | GmlGeometryType::Tin
        )
    }
}

/// The read-only lookup tables threaded through pass-2 resolution (registry,
/// appearance index, per-file CRS), bundled with the mutable malformation sink
/// so `resolve` and its callees carry one context argument instead of four
/// separate ones that always travel together.
struct ResolveCtx<'a> {
    registry: &'a GeomRegistry,
    appearance: &'a AppearanceIndex,
    srs_by_file: &'a HashMap<String, EpsgCode>,
    malformations: &'a mut Vec<Malformation>,
}

/// Resolve a top-level geometry node into a single [`Euclidean3DGeometry`],
/// following `xlink:href` references through `registry` and attaching the
/// appearance targeting each surface from `appearance` (an empty index attaches
/// nothing). Attachment is at the polygon leaf, so mesh welding carries it into
/// `PolygonMesh` / `Solid`. Returns `None` when the node, or every one of its
/// members, cannot be resolved. Malformed input encountered while resolving
/// (Action Standard §4.3) is appended to `malformations`.
pub(super) fn resolve_root(
    node: &GeomNode,
    registry: &GeomRegistry,
    appearance: &AppearanceIndex,
    srs_by_file: &HashMap<String, EpsgCode>,
    malformations: &mut Vec<Malformation>,
) -> Option<Euclidean3DGeometry> {
    let mut ctx = ResolveCtx {
        registry,
        appearance,
        srs_by_file,
        malformations,
    };
    resolve(node, &mut ctx, &[], &mut HashSet::new())
}

/// Resolve a top-level geometry node with no appearance attached, no known CRS,
/// and no interest in the malformations collected along the way.
#[cfg(test)]
pub(super) fn resolve_root_bare(
    node: &GeomNode,
    registry: &GeomRegistry,
) -> Option<Euclidean3DGeometry> {
    let mut malformations = Vec::new();
    resolve_root(
        node,
        registry,
        &AppearanceIndex::default(),
        &HashMap::new(),
        &mut malformations,
    )
}

/// Resolve one node. `enclosing` is the chain of enclosing container ids, each
/// file-qualified (nearest first), so a material bound to an aggregate reaches its
/// member polygons even across an `xlink`ed file boundary. `in_progress` holds the
/// `xlink` keys on the current resolution path, so a reference cycle is caught
/// instead of recursing forever.
fn resolve(
    node: &GeomNode,
    ctx: &mut ResolveCtx,
    enclosing: &[(&str, &str)],
    in_progress: &mut HashSet<RawNodeKey>,
) -> Option<Euclidean3DGeometry> {
    match node {
        GeomNode::Resolved(geometry, ids) => Some(with_appearance(
            geometry.clone(),
            ids,
            ctx.appearance,
            enclosing,
        )),
        GeomNode::Ref(key) => resolve_ref(key, ctx, enclosing, in_progress),
        GeomNode::Unresolved(unresolved) => construct(unresolved, ctx, enclosing, in_progress),
    }
}

/// Attach the appearance for a leaf's captured ids. A `Polygon` binds per face and
/// a `TriangulatedSurface`'s `TriangularMesh` binds one draped texture over its
/// triangles; a `Polygon`'s appearance is welded into any enclosing mesh
/// downstream. The leaf's own surface id takes precedence over the `enclosing`
/// container ids. Every candidate is qualified by the file its id belongs to: the
/// leaf's own ids by the leaf's file, the enclosing ids by their containers' files.
fn with_appearance(
    mut geometry: Euclidean3DGeometry,
    ids: &LeafIds,
    appearance: &AppearanceIndex,
    enclosing: &[(&str, &str)],
) -> Euclidean3DGeometry {
    if appearance.is_empty() {
        return geometry;
    }
    let Some(first) = ids.faces.first() else {
        return geometry;
    };
    let mut candidates: Vec<(&str, &str)> = Vec::with_capacity(1 + enclosing.len());
    candidates.extend(first.surface.as_deref().map(|id| (ids.file.as_str(), id)));
    candidates.extend_from_slice(enclosing);
    match &mut geometry {
        Euclidean3DGeometry::Polygon(polygon) => {
            appearance.apply_to_polygon(polygon, &candidates, &ids.file, &first.rings);
        }
        Euclidean3DGeometry::TriangularMesh(mesh) => {
            appearance.apply_to_triangular_mesh(mesh, &candidates, &ids.file, &ids.faces);
        }
        _ => {}
    }
    geometry
}

/// Resolve an `xlink:href` target through `registry`. `in_progress` holds the keys
/// on the current path, so a reference cycle is caught instead of recursing forever.
fn resolve_ref(
    key: &RawNodeKey,
    ctx: &mut ResolveCtx,
    enclosing: &[(&str, &str)],
    in_progress: &mut HashSet<RawNodeKey>,
) -> Option<Euclidean3DGeometry> {
    if !in_progress.insert(key.clone()) {
        tracing::warn!(id = key.1, "citygml geometry: cyclic xlink:href, skipped");
        ctx.malformations.push(Malformation {
            file: key.0.clone(),
            location: key.1.clone(),
            reason: "citygml geometry: cyclic xlink:href, skipped".to_string(),
        });
        return None;
    }
    // `ctx.registry` is a `&'a GeomRegistry` copied out of `ctx`, so the `&'a
    // GeomNode` this yields borrows independently of `ctx` itself and doesn't
    // keep `ctx` borrowed across the recursive `resolve` call below.
    let target = ctx.registry.get(key);
    let resolved = match target {
        Some(target) => resolve(target, ctx, enclosing, in_progress),
        None => {
            // Not collected: a dangling `xlink:href` is indistinguishable here
            // from an ordinary cross-file reference, which these single-document
            // sources hit routinely (the target simply hasn't been registered
            // from this file). Collecting here would reject valid split
            // datasets, not just malformed ones.
            tracing::warn!(
                id = key.1,
                "citygml geometry: unresolved xlink:href, skipped"
            );
            None
        }
    };
    in_progress.remove(key);
    resolved
}

/// Assemble a container from its resolved members, dispatching on its CityGML
/// type. Members that fail to resolve are dropped, so a container survives with
/// whatever members did resolve. This container's own gml:id is prepended to
/// `enclosing` for its members, so an appearance bound to the container reaches
/// them. The container's frame is `node.file`'s CRS; a member from a file in a
/// different CRS is dropped with error rather than folded in.
fn construct(
    node: &Unresolved,
    ctx: &mut ResolveCtx,
    enclosing: &[(&str, &str)],
    in_progress: &mut HashSet<RawNodeKey>,
) -> Option<Euclidean3DGeometry> {
    let mut member_enclosing: Vec<(&str, &str)> = Vec::with_capacity(1 + enclosing.len());
    member_enclosing.extend(node.id.as_deref().map(|id| (node.file.as_str(), id)));
    member_enclosing.extend_from_slice(enclosing);

    let frame = frame_for(&node.file, ctx.srs_by_file);
    let members: Vec<(Role, Euclidean3DGeometry)> = node
        .members
        .iter()
        .filter_map(|(role, child)| {
            if frame_for(source_file(child), ctx.srs_by_file) != frame {
                // Not collected, for the same reason as the unresolved-href
                // site above: `frame_for` keys on file URL, so within one
                // document this can never fire on same-file members. The only
                // way to trip it in a single-document read is an ordinary
                // cross-file reference, so collecting here would reject valid
                // split datasets.
                tracing::error!(
                    "citygml geometry: member srsName disagrees with its parent, skipped"
                );
                return None;
            }
            resolve(child, ctx, &member_enclosing, in_progress).map(|g| (*role, g))
        })
        .collect();

    // Pass-2 leaf functions (`ring`/`surface_mesh`/`solid` and their helpers)
    // push bare (file/location empty) Malformations, since they only see
    // resolved members and have neither this node's file nor its gml:id.
    // Backfill both here, at the owning `construct` call, mirroring
    // `geometry_node`'s pass-1 backfill in geometry_next.rs. Only empty
    // fields are filled, so a site that already named its own location keeps
    // it.
    let before = ctx.malformations.len();
    let built = match node.ty {
        GmlGeometryType::MultiPoint
        | GmlGeometryType::MultiCurve
        | GmlGeometryType::CompositeCurve
        | GmlGeometryType::MultiSurface
        | GmlGeometryType::CompositeSolid
        | GmlGeometryType::MultiSolid
        | GmlGeometryType::MultiGeometry
        | GmlGeometryType::GeometricComplex => Some(collection(members)),
        // A single base curve/surface passes through unchanged.
        // TODO: honor orientation="-" (reverse curve / flip surface normals).
        GmlGeometryType::OrientableCurve | GmlGeometryType::OrientableSurface => {
            members.into_iter().next().map(|(_, geometry)| geometry)
        }
        GmlGeometryType::Ring => ring(members, &frame, ctx.malformations),
        GmlGeometryType::CompositeSurface
        | GmlGeometryType::Shell
        | GmlGeometryType::Surface
        | GmlGeometryType::PolyhedralSurface => surface_mesh(members, &frame, ctx.malformations),
        GmlGeometryType::Solid => solid(members, &frame, ctx.malformations),
        _ => {
            tracing::warn!("citygml geometry: inline type deferred to pass 2, skipped");
            None
        }
    };
    for m in &mut ctx.malformations[before..] {
        if m.file.is_empty() {
            m.file = node.file.clone();
        }
        if m.location.is_empty() {
            if let Some(id) = &node.id {
                m.location = id.clone();
            }
        }
    }
    built
}

/// The source file URL a child node's coordinates belong to.
fn source_file(node: &GeomNode) -> &str {
    match node {
        GeomNode::Resolved(_, ids) => &ids.file,
        GeomNode::Ref(key) => &key.0,
        GeomNode::Unresolved(unresolved) => &unresolved.file,
    }
}

/// Gather members into a `Collection`, discarding their roles.
fn collection(members: Vec<(Role, Euclidean3DGeometry)>) -> Euclidean3DGeometry {
    Euclidean3DGeometry::Collection(Collection3D::new(
        members.into_iter().map(|(_, geometry)| geometry),
    ))
}

/// Concatenate curve members into one open exterior ring, yielding a single-ring
/// `Polygon`.
fn ring(
    members: Vec<(Role, Euclidean3DGeometry)>,
    frame: &CoordinateFrame,
    malformations: &mut Vec<Malformation>,
) -> Option<Euclidean3DGeometry> {
    let mut exterior: Vec<[f64; 3]> = Vec::new();
    for (_, geometry) in members {
        match into_line_string(geometry, malformations) {
            Some(line) => exterior.extend_from_slice(line.coords()),
            None => {
                tracing::warn!("citygml geometry: non-curve ring member, skipped");
                malformations.push(Malformation {
                    file: String::new(),
                    location: String::new(),
                    reason: "citygml geometry: non-curve ring member, skipped".to_string(),
                });
            }
        }
    }
    if exterior.is_empty() {
        return None;
    }
    let polygon = Polygon3D::from_rings(frame.clone(), exterior, Vec::<Vec<[f64; 3]>>::new());
    Some(Euclidean3DGeometry::Polygon(Box::new(polygon)))
}

/// Assemble surface members into one mesh, for an inline `Surface` /
/// `PolyhedralSurface`, a `CompositeSurface`, or a solid's `Shell`. A single
/// already-built mesh passes through as-is; otherwise bare polygon members are
/// welded into a single mesh, carrying each member polygon's appearance across.
/// Mesh members mixed with other members are not yet supported and are skipped
/// with a warning.
fn surface_mesh(
    members: Vec<(Role, Euclidean3DGeometry)>,
    frame: &CoordinateFrame,
    malformations: &mut Vec<Malformation>,
) -> Option<Euclidean3DGeometry> {
    if members.len() == 1
        && matches!(
            members[0].1,
            Euclidean3DGeometry::PolygonMesh(_) | Euclidean3DGeometry::TriangularMesh(_)
        )
    {
        return members.into_iter().next().map(|(_, geometry)| geometry);
    }

    let mut faces: Vec<Polygon3D> = Vec::new();
    for (_, geometry) in members {
        match geometry {
            Euclidean3DGeometry::Polygon(polygon) => faces.push(*polygon),
            Euclidean3DGeometry::PolygonMesh(_) | Euclidean3DGeometry::TriangularMesh(_) => {
                // Not present-but-malformed input: our resolver simply can't
                // merge a mesh member into a composite surface yet.
                tracing::warn!(
                    "citygml geometry: merging a mesh into a composite surface is not yet supported, member skipped"
                )
            }
            _ => {
                tracing::warn!("citygml geometry: expected a surface member, skipped");
                malformations.push(Malformation {
                    file: String::new(),
                    location: String::new(),
                    reason: "citygml geometry: expected a surface member, skipped".to_string(),
                });
            }
        }
    }
    if faces.is_empty() {
        return None;
    }
    build_mesh(faces, frame, malformations)
        .map(|mesh| Euclidean3DGeometry::PolygonMesh(Box::new(mesh)))
}

/// Pair an exterior boundary with any interior void boundaries into a `Solid`.
fn solid(
    members: Vec<(Role, Euclidean3DGeometry)>,
    frame: &CoordinateFrame,
    malformations: &mut Vec<Malformation>,
) -> Option<Euclidean3DGeometry> {
    let mut exterior: Option<Shell> = None;
    let mut interiors: Vec<Shell> = Vec::new();
    for (role, geometry) in members {
        match role {
            Role::Exterior if exterior.is_none() => {
                exterior = into_shell(geometry, frame, malformations)
            }
            Role::Exterior => {
                tracing::warn!("citygml geometry: solid with multiple exteriors, extra skipped");
                malformations.push(Malformation {
                    file: String::new(),
                    location: String::new(),
                    reason: "citygml geometry: solid with multiple exteriors, extra skipped"
                        .to_string(),
                });
            }
            Role::Interior => interiors.extend(into_shell(geometry, frame, malformations)),
            Role::Member => {
                tracing::warn!("citygml geometry: unexpected solid member role, skipped");
                malformations.push(Malformation {
                    file: String::new(),
                    location: String::new(),
                    reason: "citygml geometry: unexpected solid member role, skipped".to_string(),
                });
            }
        }
    }
    let exterior = exterior?;
    Some(Euclidean3DGeometry::Solid(Box::new(Solid::new(
        frame.clone(),
        exterior,
        interiors,
    ))))
}

/// Weld independent faces into one polygon mesh in `frame`.
fn build_mesh(
    faces: Vec<Polygon3D>,
    frame: &CoordinateFrame,
    malformations: &mut Vec<Malformation>,
) -> Option<PolygonMesh3D> {
    match PolygonMesh3D::from_polygons(frame.clone(), &faces) {
        Ok(mesh) => Some(mesh),
        Err(e) => {
            tracing::error!("citygml geometry: failed to weld mesh: {e}");
            malformations.push(Malformation {
                file: String::new(),
                location: String::new(),
                reason: format!("citygml geometry: failed to weld mesh: {e}"),
            });
            None
        }
    }
}

/// Coerce a resolved member to a curve.
fn into_line_string(
    geometry: Euclidean3DGeometry,
    malformations: &mut Vec<Malformation>,
) -> Option<LineString3D> {
    match geometry {
        Euclidean3DGeometry::LineString(line) => Some(line),
        _ => {
            tracing::warn!("citygml geometry: expected a curve, skipped");
            malformations.push(Malformation {
                file: String::new(),
                location: String::new(),
                reason: "citygml geometry: expected a curve, skipped".to_string(),
            });
            None
        }
    }
}

/// Coerce a resolved member to a solid boundary shell. A `CompositeSurface` or
/// `Shell` boundary has already been assembled into a mesh by [`surface_mesh`], so
/// this just unwraps it; a bare `Polygon` boundary is welded into a single-face
/// mesh.
fn into_shell(
    geometry: Euclidean3DGeometry,
    frame: &CoordinateFrame,
    malformations: &mut Vec<Malformation>,
) -> Option<Shell> {
    match geometry {
        Euclidean3DGeometry::PolygonMesh(mesh) => Some(Shell::PolygonMesh(mesh.into_data())),
        Euclidean3DGeometry::TriangularMesh(mesh) => Some(Shell::TriangularMesh(mesh.into_data())),
        Euclidean3DGeometry::Polygon(polygon) => build_mesh(vec![*polygon], frame, malformations)
            .map(|mesh| Shell::PolygonMesh(mesh.into_data())),
        _ => {
            tracing::warn!("citygml geometry: cannot use as a solid boundary, skipped");
            malformations.push(Malformation {
                file: String::new(),
                location: String::new(),
                reason: "citygml geometry: cannot use as a solid boundary, skipped".to_string(),
            });
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::point::Point3D;

    const FRAME: CoordinateFrame = CoordinateFrame::Euclidean;

    fn key(id: &str) -> RawNodeKey {
        ("file:///test.gml".to_string(), id.to_string())
    }

    fn point() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(FRAME, [1.0, 2.0, 3.0]))
    }

    fn triangle() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            FRAME,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        )))
    }

    fn line() -> Euclidean3DGeometry {
        Euclidean3DGeometry::LineString(LineString3D::from_coords(
            FRAME,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
        ))
    }

    fn resolved(geometry: Euclidean3DGeometry) -> GeomNode {
        GeomNode::Resolved(
            geometry,
            LeafIds {
                file: String::new(),
                faces: Vec::new(),
            },
        )
    }

    fn node(ty: GmlGeometryType, members: Vec<(Role, GeomNode)>) -> GeomNode {
        GeomNode::Unresolved(Unresolved {
            ty,
            id: None,
            file: String::new(),
            members,
        })
    }

    fn crs(code: u16) -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(code))
    }

    fn triangle_in(frame: CoordinateFrame) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            frame,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        )))
    }

    fn point_in(frame: CoordinateFrame) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(frame, [1.0, 2.0, 3.0]))
    }

    fn resolved_in(file: &str, geometry: Euclidean3DGeometry) -> GeomNode {
        GeomNode::Resolved(
            geometry,
            LeafIds {
                file: file.to_string(),
                faces: Vec::new(),
            },
        )
    }

    fn node_in(file: &str, ty: GmlGeometryType, members: Vec<(Role, GeomNode)>) -> GeomNode {
        GeomNode::Unresolved(Unresolved {
            ty,
            id: None,
            file: file.to_string(),
            members,
        })
    }

    fn srs(pairs: &[(&str, u16)]) -> HashMap<String, EpsgCode> {
        pairs
            .iter()
            .map(|(file, code)| (file.to_string(), EpsgCode::new(*code)))
            .collect()
    }

    fn collection_len(geometry: &Euclidean3DGeometry) -> usize {
        match geometry {
            Euclidean3DGeometry::Collection(collection) => collection.len(),
            other => panic!("expected a collection, got {other:?}"),
        }
    }

    #[test]
    fn multipoint_becomes_collection() {
        let n = node(
            GmlGeometryType::MultiPoint,
            vec![
                (Role::Member, resolved(point())),
                (Role::Member, resolved(point())),
            ],
        );
        let geometry = resolve_root_bare(&n, &GeomRegistry::new()).unwrap();
        assert_eq!(collection_len(&geometry), 2);
    }

    #[test]
    fn shell_welds_polygons_into_mesh() {
        let n = node(
            GmlGeometryType::Shell,
            vec![
                (Role::Member, resolved(triangle())),
                (Role::Member, resolved(triangle())),
            ],
        );
        let geometry = resolve_root_bare(&n, &GeomRegistry::new()).unwrap();
        assert!(matches!(geometry, Euclidean3DGeometry::PolygonMesh(_)));
    }

    #[test]
    fn solid_from_exterior_shell() {
        let shell = node(
            GmlGeometryType::Shell,
            vec![(Role::Member, resolved(triangle()))],
        );
        let n = node(GmlGeometryType::Solid, vec![(Role::Exterior, shell)]);
        let geometry = resolve_root_bare(&n, &GeomRegistry::new()).unwrap();
        assert!(matches!(geometry, Euclidean3DGeometry::Solid(_)));
    }

    #[test]
    fn composite_surface_welds_polygons_into_mesh() {
        let n = node(
            GmlGeometryType::CompositeSurface,
            vec![
                (Role::Member, resolved(triangle())),
                (Role::Member, resolved(triangle())),
            ],
        );
        let geometry = resolve_root_bare(&n, &GeomRegistry::new()).unwrap();
        assert!(matches!(geometry, Euclidean3DGeometry::PolygonMesh(_)));
    }

    fn two_face_mesh() -> Euclidean3DGeometry {
        let a = Polygon3D::from_rings(
            FRAME,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let b = Polygon3D::from_rings(
            FRAME,
            [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        Euclidean3DGeometry::PolygonMesh(Box::new(
            PolygonMesh3D::from_polygons(FRAME, &[a, b]).unwrap(),
        ))
    }

    #[test]
    fn composite_surface_welds_polygons_and_skips_mesh_members() {
        let n = node(
            GmlGeometryType::CompositeSurface,
            vec![
                (Role::Member, resolved(two_face_mesh())),
                (Role::Member, resolved(triangle())),
                (Role::Member, resolved(triangle())),
            ],
        );
        match resolve_root_bare(&n, &GeomRegistry::new()).unwrap() {
            Euclidean3DGeometry::PolygonMesh(mesh) => assert_eq!(mesh.num_faces(), 2),
            other => panic!("expected PolygonMesh, got {other:?}"),
        }
    }

    #[test]
    fn solid_from_composite_surface_exterior() {
        // PLATEAU's common shape: a Solid whose boundary is a CompositeSurface.
        let boundary = node(
            GmlGeometryType::CompositeSurface,
            vec![(Role::Member, resolved(triangle()))],
        );
        let n = node(GmlGeometryType::Solid, vec![(Role::Exterior, boundary)]);
        let geometry = resolve_root_bare(&n, &GeomRegistry::new()).unwrap();
        assert!(matches!(geometry, Euclidean3DGeometry::Solid(_)));
    }

    #[test]
    fn ring_concatenates_curves_into_polygon() {
        let n = node(
            GmlGeometryType::Ring,
            vec![(Role::Member, resolved(line()))],
        );
        let geometry = resolve_root_bare(&n, &GeomRegistry::new()).unwrap();
        assert!(matches!(geometry, Euclidean3DGeometry::Polygon(_)));
    }

    #[test]
    fn ref_resolves_via_registry() {
        let mut registry = GeomRegistry::new();
        registry.insert(key("p1"), resolved(point()));
        let n = node(
            GmlGeometryType::MultiPoint,
            vec![(Role::Member, GeomNode::Ref(key("p1")))],
        );
        let geometry = resolve_root_bare(&n, &registry).unwrap();
        assert_eq!(collection_len(&geometry), 1);
    }

    #[test]
    fn unresolved_ref_is_dropped() {
        let n = node(
            GmlGeometryType::MultiPoint,
            vec![
                (Role::Member, GeomNode::Ref(key("missing"))),
                (Role::Member, resolved(point())),
            ],
        );
        let geometry = resolve_root_bare(&n, &GeomRegistry::new()).unwrap();
        assert_eq!(collection_len(&geometry), 1);
    }

    #[test]
    fn cyclic_ref_terminates() {
        let mut registry = GeomRegistry::new();
        registry.insert(
            key("a"),
            node(
                GmlGeometryType::MultiGeometry,
                vec![(Role::Member, GeomNode::Ref(key("a")))],
            ),
        );
        let geometry = resolve_root_bare(&GeomNode::Ref(key("a")), &registry).unwrap();
        assert_eq!(collection_len(&geometry), 0);
    }

    /// Action Standard §4.3: a cyclic `xlink:href` is present-but-malformed
    /// input, so it must be collected (not just warned about) with the file
    /// and `gml:id` that name the cycle, alongside the existing lenient
    /// (drop-and-continue) behavior asserted above.
    #[test]
    fn cyclic_ref_is_collected_as_a_malformation() {
        let mut registry = GeomRegistry::new();
        registry.insert(
            key("a"),
            node(
                GmlGeometryType::MultiGeometry,
                vec![(Role::Member, GeomNode::Ref(key("a")))],
            ),
        );
        let mut malformations = Vec::new();
        let geometry = resolve_root(
            &GeomNode::Ref(key("a")),
            &registry,
            &AppearanceIndex::default(),
            &HashMap::new(),
            &mut malformations,
        )
        .unwrap();
        assert_eq!(collection_len(&geometry), 0);
        assert_eq!(
            malformations,
            vec![Malformation {
                file: "file:///test.gml".to_string(),
                location: "a".to_string(),
                reason: "citygml geometry: cyclic xlink:href, skipped".to_string(),
            }]
        );
    }

    fn resolve_root_srs(node: &GeomNode, srs: &HashMap<String, EpsgCode>) -> Euclidean3DGeometry {
        let mut malformations = Vec::new();
        resolve_root(
            node,
            &GeomRegistry::new(),
            &AppearanceIndex::default(),
            srs,
            &mut malformations,
        )
        .unwrap()
    }

    #[test]
    fn member_in_disagreeing_crs_is_dropped_from_weld() {
        // The face from a different-CRS file is dropped; the rest still weld.
        let n = node_in(
            "a.gml",
            GmlGeometryType::CompositeSurface,
            vec![
                (Role::Member, resolved_in("a.gml", triangle_in(crs(6668)))),
                (Role::Member, resolved_in("a.gml", triangle_in(crs(6668)))),
                (Role::Member, resolved_in("b.gml", triangle_in(crs(6697)))),
            ],
        );
        match resolve_root_srs(&n, &srs(&[("a.gml", 6668), ("b.gml", 6697)])) {
            Euclidean3DGeometry::PolygonMesh(mesh) => assert_eq!(mesh.num_faces(), 2),
            other => panic!("expected PolygonMesh, got {other:?}"),
        }
    }

    #[test]
    fn member_in_same_crs_from_another_file_is_kept() {
        // Same CRS from a different file still welds.
        let n = node_in(
            "a.gml",
            GmlGeometryType::CompositeSurface,
            vec![
                (Role::Member, resolved_in("a.gml", triangle_in(crs(6668)))),
                (Role::Member, resolved_in("c.gml", triangle_in(crs(6668)))),
            ],
        );
        match resolve_root_srs(&n, &srs(&[("a.gml", 6668), ("c.gml", 6668)])) {
            Euclidean3DGeometry::PolygonMesh(mesh) => assert_eq!(mesh.num_faces(), 2),
            other => panic!("expected PolygonMesh, got {other:?}"),
        }
    }

    #[test]
    fn disagreeing_member_is_gated_for_non_welded_types_too() {
        // The gate is uniform: a non-welded MultiPoint member is gated too.
        let n = node_in(
            "a.gml",
            GmlGeometryType::MultiPoint,
            vec![
                (Role::Member, resolved_in("a.gml", point_in(crs(6668)))),
                (Role::Member, resolved_in("b.gml", point_in(crs(6697)))),
            ],
        );
        let geometry = resolve_root_srs(&n, &srs(&[("a.gml", 6668), ("b.gml", 6697)]));
        assert_eq!(collection_len(&geometry), 1);
    }
}
