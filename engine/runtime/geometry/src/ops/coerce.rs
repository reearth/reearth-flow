//! Geometry-type coercion.
//!
//! Coercion re-labels a geometry as another type without moving a coordinate: a
//! face becomes the polylines of its boundary rings, a chain that closes becomes
//! the face it bounds, a polygonal surface becomes triangles. What a geometry
//! cannot become, it simply stays — see [`Coerce`].

use crate::collection::{Collection2D, Collection3D};
use crate::coordinate::CoordinateFrame;
use crate::line_string::{LineString2D, LineString3D};
use crate::ops::triangulation::Cache;
use crate::ops::UnsupportedOperation;
use crate::polygon::{Polygon2D, Polygon3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// The geometry type a coercion re-represents its input as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoercionTarget {
    /// Curves: every boundary ring of every face, each as a polyline of its own.
    LineString,
    /// Faces: the area a closed chain bounds, or the faces a surface is built from.
    Polygon,
    /// Triangles: a polygonal surface tessellated. A volume stays a volume, with
    /// its boundary triangulated.
    TriangularMesh,
}

/// Re-represent a geometry as [`CoercionTarget`].
///
/// Coordinates are carried over verbatim: a ring is never re-wound and never
/// closed, so a chain that was not a ring in the source does not become one here.
///
/// [`UnsupportedOperation`] means **nothing changed** — the geometry already is
/// the target type, or its coordinates do not satisfy what the target needs (an
/// open chain bounds no face). It is not a failure to report: a caller leaves the
/// geometry as it arrived, and `self` is untouched. On success `self` may be left
/// moved-from, as [`Triangulate`](crate::ops::Triangulate) does.
///
/// A container coerces its members one at a time and best-effort: a member that
/// cannot be coerced stays as it is, and the container reports "nothing changed"
/// only when no member did.
///
/// A curve carries no appearance, so coercing a face to
/// [`CoercionTarget::LineString`] drops its materials and UV.
#[enum_dispatch::enum_dispatch]
pub trait Coerce {
    /// Re-represent this geometry as `target`. The default body reports nothing
    /// changed; a leaf opts in by overriding it.
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        let _ = (target, cache);
        Err(unchanged::<Self>())
    }
}

// `enum_dispatch` forwards by UFCS, not auto-deref, so the boxed variants need
// the trait on the `Box` itself.
impl<T: Coerce + ?Sized> Coerce for Box<T> {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        (**self).coerce(target, cache)
    }
}

pub(crate) fn unchanged<T: ?Sized>() -> UnsupportedOperation {
    UnsupportedOperation {
        geometry: core::any::type_name::<T>(),
        operation: "coerce",
    }
}

/// Gather coerced parts into one geometry. `None` when there is no part, which
/// is the caller's "nothing was coerced".
pub(crate) fn wrap_2d(mut parts: Vec<Euclidean2DGeometry>) -> Option<Geometry> {
    match parts.len() {
        0 => None,
        1 => parts.pop().map(Geometry::Euclidean2D),
        _ => Some(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            Collection2D::new(parts),
        ))),
    }
}

/// The 3D counterpart of [`wrap_2d`].
pub(crate) fn wrap_3d(mut parts: Vec<Euclidean3DGeometry>) -> Option<Geometry> {
    match parts.len() {
        0 => None,
        1 => parts.pop().map(Geometry::Euclidean3D),
        _ => Some(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            Collection3D::new(parts),
        ))),
    }
}

fn line_2d(
    frame: &CoordinateFrame,
    ring: impl IntoIterator<Item = [f64; 2]>,
    elevation: Option<f64>,
) -> Euclidean2DGeometry {
    let line = match elevation {
        None => LineString2D::from_coords(frame.clone(), ring),
        Some(elevation) => LineString2D::from_coords_at_elevation(frame.clone(), ring, elevation),
    };
    Euclidean2DGeometry::LineString(line)
}

fn line_3d(
    frame: &CoordinateFrame,
    ring: impl IntoIterator<Item = [f64; 3]>,
) -> Euclidean3DGeometry {
    Euclidean3DGeometry::LineString(LineString3D::from_coords(frame.clone(), ring))
}

/// Append every ring of `face`, exterior first. A face with no exterior ring
/// bounds nothing and contributes none.
pub(crate) fn push_face_lines_2d(face: &Polygon2D, out: &mut Vec<Euclidean2DGeometry>) {
    let exterior = face.exterior();
    if exterior.is_empty() {
        return;
    }
    let frame = face.frame();
    let elevation = face.elevation();
    out.push(line_2d(frame, exterior.iter().copied(), elevation));
    for hole in face.interiors() {
        out.push(line_2d(frame, hole.iter().copied(), elevation));
    }
}

/// The 3D counterpart of [`push_face_lines_2d`].
pub(crate) fn push_face_lines_3d(face: &Polygon3D, out: &mut Vec<Euclidean3DGeometry>) {
    let exterior = face.exterior();
    if exterior.is_empty() {
        return;
    }
    let frame = face.frame();
    out.push(line_3d(frame, exterior.iter().copied()));
    for hole in face.interiors() {
        out.push(line_3d(frame, hole.iter().copied()));
    }
}

/// Whether a chain closes a ring. A chain of three or fewer encloses no area
/// even when its ends meet.
pub(crate) fn closes_a_ring<const N: usize>(coords: &[[f64; N]]) -> bool {
    coords.len() >= 4 && coords.first() == coords.last()
}

/// One triangle as a ring, closed by repeating its first vertex — the form the
/// polygon constructors expect.
pub(crate) fn triangle_ring<const N: usize>(
    vertices: &[[f64; N]],
    [i, j, k]: [u32; 3],
) -> [[f64; N]; 4] {
    [
        vertices[i as usize],
        vertices[j as usize],
        vertices[k as usize],
        vertices[i as usize],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::EpsgCode;
    use crate::csg::Csg;
    use crate::point::Point3D;
    use crate::point_cloud::PointCloud;
    use crate::polygon_mesh::{PolygonMesh3D, PolygonMesh3DData};
    use crate::solid::Solid;
    use crate::triangular_mesh::TriangularMesh3D;
    use crate::GeometryCollection;
    use reearth_flow_common::attribute::{Attribute, AttributeValue, Attributes};

    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    const SQUARE_2D: [[f64; 2]; 5] = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];

    fn hole() -> Vec<[f64; 3]> {
        vec![
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    fn face(holes: usize) -> Polygon3D {
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            SQUARE,
            (0..holes).map(|_| hole()),
        )
    }

    fn g3(g: Euclidean3DGeometry) -> Geometry {
        Geometry::Euclidean3D(g)
    }

    fn face_geometry(holes: usize) -> Geometry {
        g3(Euclidean3DGeometry::Polygon(Box::new(face(holes))))
    }

    fn chain(coords: [[f64; 3]; 5]) -> Geometry {
        g3(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            coords,
        )))
    }

    /// A two-face surface: one holed, one not.
    fn surface() -> Geometry {
        let mesh =
            PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&face(1), &face(0)]).unwrap();
        g3(Euclidean3DGeometry::PolygonMesh(Box::new(mesh)))
    }

    /// A one-face volume with one void shell.
    fn solid() -> Geometry {
        g3(Euclidean3DGeometry::Solid(Box::new(Solid::new(
            CoordinateFrame::Euclidean,
            PolygonMesh3DData::from_polygons([&face(0)]),
            vec![PolygonMesh3DData::from_polygons([&face(0)]).into()],
        ))))
    }

    fn point() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [0.0; 3]))
    }

    fn coerce(geometry: &mut Geometry, target: CoercionTarget) -> Option<Geometry> {
        geometry.coerce(target, &mut Cache::new()).ok()
    }

    /// The ring of every part a coerced geometry is made of: a polyline's chain,
    /// or a face's exterior.
    fn rings(geometry: &Geometry) -> Vec<Vec<[f64; 3]>> {
        fn parts(g: &Euclidean3DGeometry) -> Vec<Vec<[f64; 3]>> {
            match g {
                Euclidean3DGeometry::LineString(l) => vec![l.coords().to_vec()],
                Euclidean3DGeometry::Polygon(p) => vec![p.exterior().to_vec()],
                Euclidean3DGeometry::Collection(c) => c.members().iter().flat_map(parts).collect(),
                other => panic!("unexpected part {other:?}"),
            }
        }
        match geometry {
            Geometry::Euclidean3D(g) => parts(g),
            other => panic!("expected a 3D geometry, got {other:?}"),
        }
    }

    #[test]
    fn coercion_rebuilds_the_geometry_from_the_same_rings() {
        let cases = vec![
            (
                "face -> curves",
                face_geometry(0),
                CoercionTarget::LineString,
                vec![SQUARE.to_vec()],
            ),
            (
                "donut -> curves, holes included",
                face_geometry(2),
                CoercionTarget::LineString,
                vec![SQUARE.to_vec(), hole(), hole()],
            ),
            (
                "surface -> curves",
                surface(),
                CoercionTarget::LineString,
                vec![SQUARE.to_vec(), hole(), SQUARE.to_vec()],
            ),
            (
                "surface -> faces",
                surface(),
                CoercionTarget::Polygon,
                vec![SQUARE.to_vec(), SQUARE.to_vec()],
            ),
            (
                "solid -> faces of every shell",
                solid(),
                CoercionTarget::Polygon,
                vec![SQUARE.to_vec(), SQUARE.to_vec()],
            ),
            (
                "closed chain -> face",
                chain(SQUARE),
                CoercionTarget::Polygon,
                vec![SQUARE.to_vec()],
            ),
        ];
        for (name, mut input, target, expected) in cases {
            let out = coerce(&mut input, target).unwrap_or_else(|| panic!("{name}: not coerced"));
            assert_eq!(rings(&out), expected, "{name}");
        }
    }

    #[test]
    fn a_2d_geometry_carries_its_elevation_across() {
        let mut g = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings_at_elevation(
                CoordinateFrame::Euclidean,
                SQUARE_2D,
                Vec::<Vec<[f64; 2]>>::new(),
                7.5,
            ),
        )));
        let Some(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line))) =
            coerce(&mut g, CoercionTarget::LineString)
        else {
            panic!("expected a polyline");
        };
        assert_eq!(line.elevation(), Some(7.5));

        let mut g = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords_at_elevation(CoordinateFrame::Euclidean, SQUARE_2D, 3.0),
        ));
        let Some(Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(face))) =
            coerce(&mut g, CoercionTarget::Polygon)
        else {
            panic!("expected a face");
        };
        assert_eq!(face.elevation(), Some(3.0));
    }

    #[test]
    fn the_triangular_mesh_target_tessellates_but_keeps_a_volume_a_volume() {
        let mut g = face_geometry(1);
        assert!(matches!(
            coerce(&mut g, CoercionTarget::TriangularMesh),
            Some(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
                _
            )))
        ));
        let mut g = solid();
        assert!(matches!(
            coerce(&mut g, CoercionTarget::TriangularMesh),
            Some(Geometry::Euclidean3D(Euclidean3DGeometry::Solid(_)))
        ));
    }

    #[test]
    fn input_the_target_does_not_apply_to_is_left_untouched() {
        let triangles = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        let empty_face = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            Vec::<[f64; 3]>::new(),
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let csg = Csg::union(
            Solid::from_exterior(
                CoordinateFrame::Euclidean,
                PolygonMesh3DData::from_polygons([&face(0)]),
            ),
            Solid::from_exterior(
                CoordinateFrame::Euclidean,
                PolygonMesh3DData::from_polygons([&face(0)]),
            ),
        );
        let cases = vec![
            ("already a face", face_geometry(0), CoercionTarget::Polygon),
            ("already a curve", chain(SQUARE), CoercionTarget::LineString),
            (
                "already triangles",
                g3(Euclidean3DGeometry::TriangularMesh(Box::new(triangles))),
                CoercionTarget::TriangularMesh,
            ),
            (
                "an open chain",
                chain([
                    [0.0, 0.0, 0.0],
                    [4.0, 0.0, 0.0],
                    [4.0, 4.0, 0.0],
                    [0.0, 4.0, 0.0],
                    [2.0, 9.0, 0.0],
                ]),
                CoercionTarget::Polygon,
            ),
            (
                "a chain too short to enclose",
                g3(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                    CoordinateFrame::Euclidean,
                    [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
                ))),
                CoercionTarget::Polygon,
            ),
            (
                "a face with no ring",
                g3(Euclidean3DGeometry::Polygon(Box::new(empty_face))),
                CoercionTarget::LineString,
            ),
            ("a point", g3(point()), CoercionTarget::LineString),
            (
                "a point cloud",
                g3(Euclidean3DGeometry::PointCloud(Box::new(
                    PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]),
                ))),
                CoercionTarget::Polygon,
            ),
            (
                "a boolean tree",
                g3(Euclidean3DGeometry::Csg(csg)),
                CoercionTarget::TriangularMesh,
            ),
            ("no geometry", Geometry::None, CoercionTarget::LineString),
            (
                "a collection with no coercible member",
                g3(Euclidean3DGeometry::Collection(Collection3D::new(
                    [point()],
                ))),
                CoercionTarget::LineString,
            ),
        ];
        for (name, before, target) in cases {
            let mut g = before.clone();
            assert!(coerce(&mut g, target).is_none(), "{name}");
            assert_eq!(g, before, "{name}: the input must stay untouched");
        }
    }

    #[test]
    fn coercion_is_idempotent() {
        for target in [
            CoercionTarget::LineString,
            CoercionTarget::Polygon,
            CoercionTarget::TriangularMesh,
        ] {
            let mut g = face_geometry(1);
            let Some(mut once) = coerce(&mut g, target) else {
                continue;
            };
            let expected = once.clone();
            assert!(coerce(&mut once, target).is_none(), "{target:?}");
            assert_eq!(once, expected, "{target:?}");
        }
    }

    // Members carry their own frame, so coercion must not fold them into one.
    #[test]
    fn a_collection_coerces_each_member_keeping_its_frame() {
        let crs = CoordinateFrame::Crs(EpsgCode::new(6697));
        let mut g = g3(Euclidean3DGeometry::Collection(Collection3D::new([
            Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                crs.clone(),
                SQUARE,
                Vec::<Vec<[f64; 3]>>::new(),
            ))),
            Euclidean3DGeometry::Polygon(Box::new(face(0))),
        ])));
        let Some(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(out))) =
            coerce(&mut g, CoercionTarget::LineString)
        else {
            panic!("expected a 3D collection");
        };
        let frames: Vec<_> = out
            .members()
            .iter()
            .map(|m| match m {
                Euclidean3DGeometry::LineString(l) => l.frame().clone(),
                other => panic!("expected a polyline member, got {other:?}"),
            })
            .collect();
        assert_eq!(frames, vec![crs, CoordinateFrame::Euclidean]);
    }

    // One member that cannot be coerced must not discard the ones that can.
    #[test]
    fn a_container_coerces_what_it_can_and_leaves_the_rest() {
        let mut g = g3(Euclidean3DGeometry::Collection(Collection3D::new([
            Euclidean3DGeometry::Polygon(Box::new(face(0))),
            point(),
        ])));
        let Some(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(out))) =
            coerce(&mut g, CoercionTarget::LineString)
        else {
            panic!("expected a 3D collection");
        };
        assert!(matches!(
            out.members(),
            [Euclidean3DGeometry::LineString(_), _]
        ));
        assert_eq!(out.members()[1], point());

        let mut g = Geometry::GeometryCollection(GeometryCollection::new(vec![
            face_geometry(0),
            Geometry::None,
        ]));
        let Some(Geometry::GeometryCollection(out)) = coerce(&mut g, CoercionTarget::LineString)
        else {
            panic!("expected a geometry collection");
        };
        assert!(matches!(
            out.members(),
            [
                Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_)),
                Geometry::None
            ]
        ));
    }

    // A CityGML feature arrives as a collection whose members carry the LOD they
    // came from; the pairing must survive.
    #[test]
    fn a_geometry_collection_keeps_its_per_member_attributes() {
        let attrs = vec![
            Attributes::from([(Attribute::new("lod"), AttributeValue::Number(1.into()))]),
            Attributes::from([(Attribute::new("lod"), AttributeValue::Number(2.into()))]),
        ];
        let members = vec![face_geometry(0), face_geometry(0)];
        let mut g = Geometry::GeometryCollection(
            GeometryCollection::with_attributes(members, attrs.clone()).unwrap(),
        );
        let Some(Geometry::GeometryCollection(out)) = coerce(&mut g, CoercionTarget::LineString)
        else {
            panic!("expected a geometry collection");
        };
        assert_eq!(out.members().len(), 2);
        assert_eq!(out.member_attributes(), attrs.as_slice());
    }
}
