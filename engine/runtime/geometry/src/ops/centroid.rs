//! The centroid of a geometry: the area-weighted center of its faces, or, when
//! it has no area, the length-weighted center of its lines, or, when it has no
//! length either, the mean of its points. Only the highest dimension present
//! counts (OGC / JTS semantics), so a polygon with a stray point keeps the
//! polygon's centroid.
//!
//! The centroid of a non-convex shape can lie outside it. A solid gets a
//! surface centroid (its boundary faces, voids included, weighted by area), not
//! a volume centroid.

use crate::coordinate::CoordinateFrame;
use crate::ops::vertex::first_vertex;
use crate::validation_next::newell_vector_3d;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

#[derive(Debug, Clone, PartialEq)]
pub enum CentroidError {
    /// Parts of the geometry are in different coordinate frames.
    MixedFrames,
    /// A part has no centroid to contribute (a point cloud, an unevaluated CSG tree).
    Unsupported(&'static str),
}

#[derive(Default)]
struct Bucket {
    w: f64,
    sum: [f64; 3],
}

impl Bucket {
    fn add(&mut self, w: f64, c: [f64; 3]) {
        self.w += w;
        for (s, x) in self.sum.iter_mut().zip(c) {
            *s += w * x;
        }
    }

    fn mean(&self) -> Option<[f64; 3]> {
        (self.w > 0.0).then(|| self.sum.map(|s| s / self.w))
    }
}

struct Acc {
    origin: [f64; 3],
    frame: Option<CoordinateFrame>,
    faces: Bucket,
    lines: Bucket,
    points: Bucket,
}

impl Acc {
    fn frame(&mut self, f: &CoordinateFrame) -> Result<(), CentroidError> {
        match &self.frame {
            None => {
                self.frame = Some(f.clone());
                Ok(())
            }
            Some(g) if g == f => Ok(()),
            Some(_) => Err(CentroidError::MixedFrames),
        }
    }

    fn local(&self, p: [f64; 3]) -> [f64; 3] {
        [
            p[0] - self.origin[0],
            p[1] - self.origin[1],
            p[2] - self.origin[2],
        ]
    }

    /// A planar ring's signed area and area-weighted centroid sum along the unit
    /// `normal`, in local coordinates.
    fn ring(&self, ring: &[[f64; 3]], normal: [f64; 3]) -> (f64, [f64; 3]) {
        if ring.len() < 3 {
            return (0.0, [0.0; 3]);
        }
        let p0 = self.local(ring[0]);
        let (mut a, mut c) = (0.0, [0.0; 3]);
        for w in ring[1..].windows(2) {
            let (p1, p2) = (self.local(w[0]), self.local(w[1]));
            let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cross = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let t = (cross[0] * normal[0] + cross[1] * normal[1] + cross[2] * normal[2]) / 2.0;
            a += t;
            for (i, ci) in c.iter_mut().enumerate() {
                *ci += t * (p0[i] + p1[i] + p2[i]) / 3.0;
            }
        }
        (a, c)
    }

    /// One face: exterior less holes, weighted by area.
    fn face(&mut self, exterior: &[[f64; 3]], holes: &[Vec<[f64; 3]>]) {
        let n = newell_vector_3d(exterior);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len == 0.0 {
            self.polyline(exterior);
            return;
        }
        let n = [n[0] / len, n[1] / len, n[2] / len];
        let (ea, ec) = self.ring(exterior, n);
        let (mut a, mut c) = (ea.abs(), ec.map(|x| x * ea.signum()));
        for h in holes {
            let (ha, hc) = self.ring(h, n);
            a -= ha.abs();
            for (ci, hi) in c.iter_mut().zip(hc) {
                *ci -= hi * ha.signum();
            }
        }
        if a > 0.0 {
            self.faces.add(a, c.map(|x| x / a));
        } else {
            self.polyline(exterior);
        }
    }

    /// A line's segments, weighted by length. A line with no length at all
    /// (every vertex the same) counts as a point, so it still has a centroid.
    fn polyline(&mut self, coords: &[[f64; 3]]) {
        let mut length = 0.0;
        for w in coords.windows(2) {
            let (a, b) = (self.local(w[0]), self.local(w[1]));
            let len =
                ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
            length += len;
            self.lines.add(
                len,
                [
                    (a[0] + b[0]) / 2.0,
                    (a[1] + b[1]) / 2.0,
                    (a[2] + b[2]) / 2.0,
                ],
            );
        }
        if length <= 0.0 {
            if let Some(&first) = coords.first() {
                self.point(first);
            }
        }
    }

    fn point(&mut self, p: [f64; 3]) {
        let l = self.local(p);
        self.points.add(1.0, l);
    }
}

fn lift(c: &[[f64; 2]], z: f64) -> Vec<[f64; 3]> {
    c.iter().map(|p| [p[0], p[1], z]).collect()
}

fn walk_2d(g: &Euclidean2DGeometry, acc: &mut Acc) -> Result<(), CentroidError> {
    use Euclidean2DGeometry as G;
    match g {
        G::Point(p) => {
            acc.frame(p.frame())?;
            let [x, y] = p.position();
            acc.point([x, y, 0.0]);
        }
        G::LineString(l) => {
            acc.frame(l.frame())?;
            acc.polyline(&lift(l.coords(), l.elevation().unwrap_or(0.0)));
        }
        G::Polygon(p) => {
            acc.frame(p.frame())?;
            let z = p.elevation().unwrap_or(0.0);
            let holes: Vec<_> = p.interiors().map(|h| lift(h, z)).collect();
            acc.face(&lift(p.exterior(), z), &holes);
        }
        G::PolygonMesh(m) => {
            acc.frame(m.frame())?;
            let mut faces = Vec::new();
            m.for_each_face_polygon(|f| faces.push(f));
            for f in faces {
                let z = f.elevation().unwrap_or(0.0);
                let holes: Vec<_> = f.interiors().map(|h| lift(h, z)).collect();
                acc.face(&lift(f.exterior(), z), &holes);
            }
        }
        G::TriangularMesh(m) => {
            acc.frame(m.frame())?;
            let z = m.elevation().unwrap_or(0.0);
            let v = m.vertices();
            for t in m.triangles() {
                let [a, b, c] = t.map(|i| v[i as usize]);
                acc.face(&lift(&[a, b, c, a], z), &[]);
            }
        }
        G::Collection(c) => {
            for m in c.members() {
                walk_2d(m, acc)?;
            }
        }
    }
    Ok(())
}

fn walk_3d(g: &Euclidean3DGeometry, acc: &mut Acc) -> Result<(), CentroidError> {
    use Euclidean3DGeometry as G;
    match g {
        G::Point(p) => {
            acc.frame(p.frame())?;
            acc.point(p.position());
        }
        G::LineString(l) => {
            acc.frame(l.frame())?;
            acc.polyline(l.coords());
        }
        G::Polygon(p) => {
            acc.frame(p.frame())?;
            let holes: Vec<_> = p.interiors().map(|h| h.to_vec()).collect();
            acc.face(p.exterior(), &holes);
        }
        G::PolygonMesh(m) => {
            acc.frame(m.frame())?;
            let mut faces = Vec::new();
            m.for_each_face_polygon(|f| faces.push(f));
            for f in faces {
                let holes: Vec<_> = f.interiors().map(|h| h.to_vec()).collect();
                acc.face(f.exterior(), &holes);
            }
        }
        G::TriangularMesh(m) => {
            acc.frame(m.frame())?;
            let v = m.vertices();
            for t in m.triangles() {
                let [a, b, c] = t.map(|i| v[i as usize]);
                acc.face(&[a, b, c, a], &[]);
            }
        }
        G::Solid(s) => {
            acc.frame(s.frame())?;
            let mut faces = Vec::new();
            s.for_each_boundary_face(|f| faces.push(f));
            for f in faces {
                let holes: Vec<_> = f.interiors().map(|h| h.to_vec()).collect();
                acc.face(f.exterior(), &holes);
            }
        }
        G::PointCloud(_) => return Err(CentroidError::Unsupported("PointCloud")),
        G::Csg(_) => return Err(CentroidError::Unsupported("Csg")),
        G::Collection(c) => {
            for m in c.members() {
                walk_3d(m, acc)?;
            }
        }
    }
    Ok(())
}

fn walk(g: &Geometry, acc: &mut Acc) -> Result<(), CentroidError> {
    match g {
        Geometry::None => Ok(()),
        Geometry::Euclidean2D(g) => walk_2d(g, acc),
        Geometry::Euclidean3D(g) => walk_3d(g, acc),
        Geometry::GeometryCollection(c) => {
            for m in c.members() {
                walk(m, acc)?;
            }
            Ok(())
        }
    }
}

/// The centroid of `geometry` with the frame it is expressed in, in the
/// geometry's own coordinate order. A 2D leaf lifts to its elevation, `0.0`
/// when it has none.
pub fn centroid(geometry: &Geometry) -> Result<Option<([f64; 3], CoordinateFrame)>, CentroidError> {
    // A point cloud or CSG tree has no first vertex but must still be refused,
    // so the origin falls back to zero rather than ending the walk early.
    let origin = first_vertex(geometry).map_or([0.0; 3], |(o, _)| o);
    let mut acc = Acc {
        origin,
        frame: None,
        faces: Bucket::default(),
        lines: Bucket::default(),
        points: Bucket::default(),
    };
    walk(geometry, &mut acc)?;
    let local = acc
        .faces
        .mean()
        .or_else(|| acc.lines.mean())
        .or_else(|| acc.points.mean());
    Ok(local
        .zip(acc.frame)
        .map(|(l, f)| ([l[0] + origin[0], l[1] + origin[1], l[2] + origin[2]], f)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::{Collection2D, Collection3D};
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::csg::Csg;
    use crate::line_string::LineString2D;
    use crate::point::Point2D;
    use crate::point_cloud::PointCloud;
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::solid::Solid;
    use crate::triangular_mesh::TriangularMesh3DData;
    use crate::{Euclidean2DGeometry as G2, Euclidean3DGeometry as G3, Geometry};

    fn frame() -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(6677))
    }

    fn poly(ext: &[[f64; 2]], holes: &[&[[f64; 2]]]) -> Geometry {
        Geometry::Euclidean2D(G2::Polygon(Box::new(Polygon2D::from_rings(
            frame(),
            ext.iter().copied(),
            holes.iter().map(|h| h.to_vec()).collect::<Vec<_>>(),
        ))))
    }

    fn xy(g: &Geometry) -> [f64; 2] {
        let (p, _) = centroid(g).unwrap().unwrap();
        [p[0], p[1]]
    }

    fn close(a: [f64; 2], b: [f64; 2]) {
        assert!(
            (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
            "{a:?} != {b:?}"
        );
    }

    const SQUARE: [[f64; 2]; 5] = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [0.0, 0.0]];

    #[test]
    fn square_centroid_is_its_middle() {
        close(xy(&poly(&SQUARE, &[])), [1.0, 1.0]);
    }

    #[test]
    fn a_hole_pulls_the_centroid_away() {
        // 4x4 square with a 2x2 hole in the right half: area 12, the centroid
        // moves left of 2.0.
        let ext = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole: &[[f64; 2]] = &[[2.0, 1.0], [4.0, 1.0], [4.0, 3.0], [2.0, 3.0], [2.0, 1.0]];
        // (16*2 - 4*3) / 12 = 20/12
        close(xy(&poly(&ext, &[hole])), [20.0 / 12.0, 2.0]);
    }

    #[test]
    fn a_c_shape_centroid_lies_outside_the_shape() {
        let c = [
            [0.0, 0.0],
            [3.0, 0.0],
            [3.0, 1.0],
            [1.0, 1.0],
            [1.0, 2.0],
            [3.0, 2.0],
            [3.0, 3.0],
            [0.0, 3.0],
            [0.0, 0.0],
        ];
        let p = xy(&poly(&c, &[]));
        // Symmetric about y = 1.5; x lies in the notch (1 < x < 3 at y = 1.5).
        assert!((p[1] - 1.5).abs() < 1e-9);
        assert!(p[0] > 1.0 && p[0] < 3.0, "{p:?}");
    }

    #[test]
    fn centroid_is_precise_at_projected_magnitudes() {
        let off = [-8_620.0, 72_580.0];
        let shifted: Vec<[f64; 2]> = SQUARE
            .iter()
            .map(|c| [c[0] + off[0], c[1] + off[1]])
            .collect();
        close(xy(&poly(&shifted, &[])), [1.0 + off[0], 1.0 + off[1]]);
    }

    #[test]
    fn a_zero_area_polygon_falls_back_to_its_ring_length() {
        let flat = [[0.0, 0.0], [2.0, 0.0], [4.0, 0.0], [0.0, 0.0]];
        // segments: (0..2) w2 mid 1, (2..4) w2 mid 3, (4..0) w4 mid 2
        // (2*1 + 2*3 + 4*2) / 8 = 2
        close(xy(&poly(&flat, &[])), [2.0, 0.0]);
    }

    #[test]
    fn lines_are_length_weighted() {
        // A long segment and a short one: the centroid sits nearer the long one.
        let g = Geometry::Euclidean2D(G2::LineString(LineString2D::from_coords(
            frame(),
            vec![[0.0, 0.0], [6.0, 0.0], [6.0, 2.0]],
        )));
        // midpoints (3,0) w6 and (6,1) w2 -> (30/8, 2/8)
        close(xy(&g), [30.0 / 8.0, 2.0 / 8.0]);
    }

    #[test]
    fn a_zero_length_line_falls_back_to_its_point() {
        let g = Geometry::Euclidean2D(G2::LineString(LineString2D::from_coords(
            frame(),
            vec![[3.0, 4.0], [3.0, 4.0]],
        )));
        close(xy(&g), [3.0, 4.0]);
    }

    #[test]
    fn points_average() {
        let pts = Collection2D::new(vec![
            G2::Point(Point2D::new(frame(), [0.0, 0.0])),
            G2::Point(Point2D::new(frame(), [2.0, 4.0])),
        ]);
        close(xy(&Geometry::Euclidean2D(G2::Collection(pts))), [1.0, 2.0]);
    }

    #[test]
    fn faces_outrank_lines_and_points_in_a_collection() {
        let g = Collection2D::new(vec![
            G2::Polygon(Box::new(Polygon2D::from_rings(
                frame(),
                SQUARE.iter().copied(),
                Vec::<Vec<[f64; 2]>>::new(),
            ))),
            G2::Point(Point2D::new(frame(), [100.0, 100.0])),
        ]);
        close(xy(&Geometry::Euclidean2D(G2::Collection(g))), [1.0, 1.0]);
    }

    #[test]
    fn mixed_frames_are_refused() {
        let g = Collection2D::new(vec![
            G2::Point(Point2D::new(frame(), [0.0, 0.0])),
            G2::Point(Point2D::new(CoordinateFrame::Euclidean, [1.0, 1.0])),
        ]);
        assert!(matches!(
            centroid(&Geometry::Euclidean2D(G2::Collection(g))),
            Err(CentroidError::MixedFrames)
        ));
    }

    #[test]
    fn a_tilted_3d_face_is_weighted_by_true_area() {
        // A tilted 2x2 face rising to z = 2 has true area 4*sqrt(2) and centre
        // (1, 1, 1); its projection has area 4. A flat unit face at x 10..11
        // has area 1 and centre (10.5, 0.5, 0). With s = 4*sqrt(2):
        //   x = (s*1 + 10.5) / (s + 1) = 2.4271 (projected weights give 2.9)
        //   y = (s*1 + 0.5) / (s + 1)
        //   z = (s*1 + 0.0) / (s + 1)
        let tilted = Polygon3D::from_rings(
            frame(),
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 2.0],
                [0.0, 2.0, 2.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let flat = Polygon3D::from_rings(
            frame(),
            vec![
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [11.0, 1.0, 0.0],
                [10.0, 1.0, 0.0],
                [10.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let g = Collection3D::new(vec![
            G3::Polygon(Box::new(tilted)),
            G3::Polygon(Box::new(flat)),
        ]);
        let (p, _) = centroid(&Geometry::Euclidean3D(G3::Collection(g)))
            .unwrap()
            .unwrap();
        let s = 4.0 * 2.0_f64.sqrt();
        let expected = [(s + 10.5) / (s + 1.0), (s + 0.5) / (s + 1.0), s / (s + 1.0)];
        for (got, want) in p.iter().zip(expected) {
            assert!((got - want).abs() < 1e-9, "{p:?} != {expected:?}");
        }
    }

    #[test]
    fn a_point_cloud_is_unsupported() {
        let cloud = PointCloud::from_positions(frame(), [[0.0, 0.0, 0.0]]);
        let g = Geometry::Euclidean3D(G3::PointCloud(Box::new(cloud)));
        assert!(matches!(centroid(&g), Err(CentroidError::Unsupported(_))));
    }

    #[test]
    fn a_csg_tree_is_unsupported() {
        let solid = || {
            Solid::from_exterior(
                CoordinateFrame::Euclidean,
                TriangularMesh3DData::from_parts(
                    vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    [0u32, 1, 2],
                )
                .unwrap(),
            )
        };
        let csg = Csg::Union(Box::new(solid().into()), Box::new(solid().into()));
        let g = Geometry::Euclidean3D(G3::Csg(csg));
        assert!(matches!(centroid(&g), Err(CentroidError::Unsupported(_))));
    }

    #[test]
    fn an_empty_geometry_has_no_centroid() {
        assert!(matches!(centroid(&Geometry::None), Ok(None)));
    }
}
