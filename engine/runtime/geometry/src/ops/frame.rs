//! The one coordinate frame a whole geometry is expressed in.
//!
//! Every leaf is visited, including the members of a `GeometryCollection` and
//! of the per-dimension collections. Point clouds and solids contribute their
//! own frame, and a CSG tree contributes the frames of its operand solids, so a
//! part with no centroid or vertex of its own still takes part in the check.

use crate::coordinate::CoordinateFrame;
use crate::csg::{Csg, ThreeDimensional};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Parts of the geometry are in different coordinate frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MixedFrames;

impl core::fmt::Display for MixedFrames {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("parts of the geometry are in different coordinate frames")
    }
}

impl std::error::Error for MixedFrames {}

/// The frame every part of `geometry` shares: `Ok(None)` when it has no part
/// carrying a frame, `Err(MixedFrames)` on the first part that disagrees.
pub fn common_frame(geometry: &Geometry) -> Result<Option<CoordinateFrame>, MixedFrames> {
    let mut found = None;
    visit(geometry, &mut found)?;
    Ok(found.cloned())
}

fn note<'a>(
    frame: &'a CoordinateFrame,
    found: &mut Option<&'a CoordinateFrame>,
) -> Result<(), MixedFrames> {
    match found {
        None => {
            *found = Some(frame);
            Ok(())
        }
        Some(f) if *f == frame => Ok(()),
        Some(_) => Err(MixedFrames),
    }
}

fn visit<'a>(g: &'a Geometry, found: &mut Option<&'a CoordinateFrame>) -> Result<(), MixedFrames> {
    match g {
        Geometry::None => Ok(()),
        Geometry::Euclidean2D(g) => visit_2d(g, found),
        Geometry::Euclidean3D(g) => visit_3d(g, found),
        Geometry::GeometryCollection(c) => c.members().iter().try_for_each(|m| visit(m, found)),
    }
}

fn visit_2d<'a>(
    g: &'a Euclidean2DGeometry,
    found: &mut Option<&'a CoordinateFrame>,
) -> Result<(), MixedFrames> {
    use Euclidean2DGeometry as G;
    match g {
        G::Point(p) => note(p.frame(), found),
        G::LineString(l) => note(l.frame(), found),
        G::Polygon(p) => note(p.frame(), found),
        G::PolygonMesh(m) => note(m.frame(), found),
        G::TriangularMesh(m) => note(m.frame(), found),
        G::Collection(c) => c.members().iter().try_for_each(|m| visit_2d(m, found)),
    }
}

fn visit_3d<'a>(
    g: &'a Euclidean3DGeometry,
    found: &mut Option<&'a CoordinateFrame>,
) -> Result<(), MixedFrames> {
    use Euclidean3DGeometry as G;
    match g {
        G::Point(p) => note(p.frame(), found),
        G::PointCloud(c) => note(c.frame(), found),
        G::LineString(l) => note(l.frame(), found),
        G::Polygon(p) => note(p.frame(), found),
        G::PolygonMesh(m) => note(m.frame(), found),
        G::TriangularMesh(m) => note(m.frame(), found),
        G::Solid(s) => note(s.frame(), found),
        G::Csg(c) => visit_csg(c, found),
        G::Collection(c) => c.members().iter().try_for_each(|m| visit_3d(m, found)),
    }
}

fn visit_csg<'a>(c: &'a Csg, found: &mut Option<&'a CoordinateFrame>) -> Result<(), MixedFrames> {
    let (Csg::Union(a, b) | Csg::Intersection(a, b) | Csg::Difference(a, b)) = c;
    for operand in [a, b] {
        match operand.as_ref() {
            ThreeDimensional::Solid(s) => note(s.frame(), found)?,
            ThreeDimensional::Csg(c) => visit_csg(c, found)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::EpsgCode;
    use crate::point::Point3D;
    use crate::point_cloud::PointCloud;
    use crate::solid::Solid;
    use crate::triangular_mesh::TriangularMesh3DData;

    fn crs(code: u16) -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(code))
    }

    fn point(code: u16) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(crs(code), [1.0, 2.0, 3.0]))
    }

    fn solid(code: u16) -> Solid {
        Solid::from_exterior(
            crs(code),
            TriangularMesh3DData::from_parts(
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                [0u32, 1, 2],
            )
            .unwrap(),
        )
    }

    #[test]
    fn a_single_frame_is_returned() {
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(vec![
            point(6675),
            point(6675),
        ])));
        assert_eq!(common_frame(&g), Ok(Some(crs(6675))));
    }

    #[test]
    fn mixed_frames_are_reported_even_after_a_point_cloud() {
        let cloud = PointCloud::from_positions(crs(6675), [[0.0, 0.0, 0.0]]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(vec![
            Euclidean3DGeometry::PointCloud(Box::new(cloud)),
            point(6675),
            point(6668),
        ])));
        assert_eq!(common_frame(&g), Err(MixedFrames));
    }

    #[test]
    fn a_csg_tree_contributes_its_operand_frames() {
        let csg = Csg::Union(Box::new(solid(6675).into()), Box::new(solid(6668).into()));
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Csg(csg));
        assert_eq!(common_frame(&g), Err(MixedFrames));
    }

    #[test]
    fn no_geometry_has_no_frame() {
        assert_eq!(common_frame(&Geometry::None), Ok(None));
    }
}
