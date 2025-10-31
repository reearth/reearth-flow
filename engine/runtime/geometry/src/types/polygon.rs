use approx::{AbsDiffEq, RelativeEq};
use flatgeom::{
    LineString2 as NLineString2, LineString3 as NLineString3, Polygon2 as NPolygon2,
    Polygon3 as NPolygon3,
};
use geo_types::Polygon as GeoPolygon;
use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::Zero;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use crate::algorithm::contains::Contains;
use crate::algorithm::coords_iter::CoordsIter;
use crate::algorithm::line_intersection::{
    line_intersection, line_intersection3d, LineIntersection,
};
use crate::algorithm::GeoFloat;
use crate::types::coordinate::Coordinate2D;

use super::conversion::geojson::create_polygon_type;
use super::coordinate::Coordinate;
use super::coordnum::{CoordFloat, CoordNum};
use super::face::Face;
use super::line::Line;
use super::line_string::{from_line_string_5d, LineString, LineString2D, LineString3D};
use super::no_value::NoValue;
use super::point::{Point2D, Point3D};
use super::rect::Rect;
use super::solid::Solid;
use super::traits::{Elevation, Surface};
use super::triangle::Triangle;
use super::validation::Validation;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub struct Polygon<T: CoordNum = f64, Z: CoordNum = f64> {
    pub(crate) exterior: LineString<T, Z>,
    pub(crate) interiors: Vec<LineString<T, Z>>,
}

pub type Polygon2D<T> = Polygon<T, NoValue>;
pub type Polygon3D<T> = Polygon<T, T>;

impl From<Polygon<f64, f64>> for Polygon<f64, NoValue> {
    #[inline]
    fn from(polygons: Polygon<f64, f64>) -> Self {
        let new_exterior = polygons.exterior.into();
        let new_interiors = polygons
            .interiors
            .into_iter()
            .map(|interior| interior.into())
            .collect::<Vec<LineString<f64, NoValue>>>();
        Polygon {
            exterior: new_exterior,
            interiors: new_interiors,
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Polygon<T, Z> {
    pub fn new(mut exterior: LineString<T, Z>, mut interiors: Vec<LineString<T, Z>>) -> Self {
        exterior.close();
        for interior in &mut interiors {
            interior.close();
        }
        Self {
            exterior,
            interiors,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn into_inner(self) -> (LineString<T, Z>, Vec<LineString<T, Z>>) {
        (self.exterior, self.interiors)
    }

    pub fn exterior(&self) -> &LineString<T, Z> {
        &self.exterior
    }

    pub fn exterior_mut<F>(&mut self, f: F)
    where
        F: FnOnce(&mut LineString<T, Z>),
    {
        f(&mut self.exterior);
        self.exterior.close();
    }

    pub fn interiors(&self) -> &[LineString<T, Z>] {
        &self.interiors
    }

    pub fn rings(&self) -> Vec<LineString<T, Z>> {
        let mut result = vec![self.exterior.clone()];
        result.extend(self.interiors.iter().cloned());
        result
    }

    pub fn interiors_mut<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Vec<LineString<T, Z>>),
    {
        f(&mut self.interiors);
        for interior in &mut self.interiors {
            interior.close();
        }
    }

    pub fn interiors_push(&mut self, new_interior: impl Into<LineString<T, Z>>) {
        let mut new_interior = new_interior.into();
        new_interior.close();
        self.interiors.push(new_interior);
    }

    pub fn area(&self) -> f64 {
        let mut area = 0.0;
        area += self.exterior().ring_area();
        for interior in self.interiors() {
            area -= interior.ring_area();
        }
        area
    }

    /// Extrudes the polygon along the Z-axis by a specified distance.
    pub fn extrude(&self, height: Z) -> Solid<T, Z> {
        let mut top_exterior = self.exterior.clone();
        let mut top_interiors = self.interiors.clone();

        // Change the z-value of a vertex to generate a top surface.
        top_exterior.translate_z(height);
        for top_interior in &mut top_interiors {
            top_interior.translate_z(height);
        }

        let bottom_faces = to_faces(&self.exterior, &self.interiors);
        let top_faces = to_faces(&top_exterior, &top_interiors);

        let side_faces = to_side_faces(
            &self.exterior,
            &top_exterior,
            &self.interiors,
            &top_interiors,
        );
        let all_faces = bottom_faces
            .into_iter()
            .chain(top_faces)
            .chain(side_faces)
            .collect();
        Solid::new_with_faces(all_faces)
    }

    pub fn validate_rings_length(&self) -> Validation {
        let mut errors: Vec<String> = vec![];

        let exterior = self.exterior();
        if exterior.coords().count() < 3 {
            let error_message = format!("Exterior Ring {exterior:?} must contain 3 or more coords");
            errors.push(error_message);
        }
        for interior in self.interiors() {
            if interior.coords().count() < 3 {
                let error_message =
                    format!("Interior Ring {interior:?} must contain 3 or more coords");
                errors.push(error_message);
            }
        }
        Validation {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    pub fn validate_rings_closed(&self) -> Validation {
        let mut errors: Vec<String> = vec![];
        let exterior = self.exterior();
        if !exterior.is_closed() {
            let error_message = format!("Exterior ring {exterior:?} is not closed");
            errors.push(error_message);
        }
        for interior in self.interiors() {
            if !interior.is_closed() {
                let error_message = format!("Interior ring {interior:?} is not closed");
                errors.push(error_message);
            }
        }
        Validation {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    pub fn validate_polygon_rings_closed(&self) -> Validation {
        let mut errors: Vec<String> = vec![];
        let exterior = self.exterior();
        if !exterior.is_closed() {
            let error_message = format!("Exterior ring {exterior:?} is not closed");
            errors.push(error_message);
        }
        for interior in self.interiors() {
            if !interior.is_closed() {
                let error_message = format!("Interior ring {interior:?} is not closed");
                errors.push(error_message);
            }
        }
        Validation {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    pub fn bounding_box(&self) -> Option<Rect<T, Z>> {
        let coords = self
            .rings()
            .into_iter()
            .flat_map(|ring| ring.into_iter().map(|point| (point.x, point.y, point.z)))
            .collect::<Vec<_>>();

        if coords.is_empty() {
            return None;
        }

        let (mut min_x, mut min_y, mut min_z) = coords[0];
        let (mut max_x, mut max_y, mut max_z) = coords[0];

        for coord in coords.iter().skip(1) {
            let (x, y, z) = coord;
            if *x < min_x {
                min_x = *x;
            } else if *x > max_x {
                max_x = *x;
            }
            if *y < min_y {
                min_y = *y;
            } else if *y > max_y {
                max_y = *y;
            }

            if *z < min_z {
                min_z = *z;
            } else if *z > max_z {
                max_z = *z;
            }
        }
        Some(Rect::new(
            Coordinate::new__(min_x, min_y, min_z),
            Coordinate::new__(max_x, max_y, max_z),
        ))
    }
}

impl Polygon<f64> {
    // Merges all the rings (exterior and interiors) into a single closed LineString.
    pub fn into_merged_contour(self) -> Result<LineString<f64, f64>, String> {
        let mut exterior = self.exterior;
        for interior in self.interiors {
            exterior = Self::into_merged_contour_single_interior(exterior, interior)?;
        }
        Ok(exterior)
    }

    fn into_merged_contour_single_interior(
        mut exterior: LineString<f64, f64>,
        mut interior: LineString<f64, f64>,
    ) -> Result<LineString<f64, f64>, String> {
        if interior.is_empty() {
            return Ok(exterior);
        }

        let epsilon = 1e-5;

        let (mut x, mut y) = (usize::MAX, usize::MAX);
        'outer: for (i, &v) in exterior.iter().enumerate() {
            'inner: for (j, &w) in interior.iter().enumerate() {
                // check if the line segment vw intersects with any edge of the exterior ring and the interior ring
                let e1 = Line::new_(v, w);
                for e2 in exterior
                    .iter()
                    .copied()
                    .zip(exterior.iter().copied().skip(1))
                {
                    if (e2.0 - v).norm() < epsilon || (e2.1 - v).norm() < epsilon {
                        continue;
                    }
                    let e2 = Line::new_(e2.0, e2.1);
                    if line_intersection3d(e1, e2).is_some() {
                        continue 'inner;
                    }
                }
                for e2 in interior
                    .iter()
                    .copied()
                    .zip(interior.iter().copied().skip(1))
                {
                    if (e2.0 - w).norm() < epsilon || (e2.1 - w).norm() < epsilon {
                        continue;
                    }
                    let e2 = Line::new_(e2.0, e2.1);
                    if line_intersection3d(e1, e2).is_some() {
                        continue 'inner;
                    }
                }
                // check if `i` is not a vertex of adjacency greater than 2
                if exterior
                    .iter()
                    .skip(1)
                    .filter(|&&k| (k - v).norm() < epsilon)
                    .count()
                    > 1
                {
                    continue 'inner;
                }
                x = i;
                y = j;
                break 'outer;
            }
        }

        // The orientation of the interior ring must be opposite to that of the exterior ring.
        let are_orientations_opposite = {
            let n = exterior
                .0
                .windows(3)
                .map(|w| {
                    let a = w[0] - w[1];
                    let b = w[2] - w[1];
                    a.cross(&b)
                })
                .max_by(|a, b| a.norm().partial_cmp(&b.norm()).unwrap())
                .unwrap()
                .normalize();
            let inner = interior.exterior_angle_sum(Some(n));
            let outer = exterior.exterior_angle_sum(Some(n));
            if (inner.abs() - outer.abs()).abs() < epsilon {
                (inner + outer).abs() < epsilon
            } else {
                return Err("Failed to determine the orientation of the rings. Possible cases are: 1. degenerate rings, 2. non-planar rings, 3. rings not closed.".to_string());
            }
        };

        if !are_orientations_opposite {
            interior.0.reverse();
        }

        exterior.0.pop();
        exterior.0.rotate_left(x);
        exterior.0.push(exterior.0[0]);
        interior.0.pop();
        interior.0.rotate_left(y);
        interior.0.push(interior.0[0]);

        exterior.0.extend(interior.0);
        exterior.0.push(exterior.0[0]);

        Ok(exterior)
    }
}

impl Polygon3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.exterior.transform_inplace(jgd2wgs);
        for interior in &mut self.interiors {
            interior.transform_inplace(jgd2wgs);
        }
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.exterior.transform_offset(x, y, z);
        for interior in &mut self.interiors {
            interior.transform_offset(x, y, z);
        }
    }
}

pub fn validate_self_intersection<T: GeoFloat, Z: GeoFloat>(polygon: &Polygon<T, Z>) -> Validation {
    let mut errors: Vec<String> = vec![];
    let exterior = polygon.exterior();
    let mut lines: Vec<Line<T, Z>> = vec![];

    lines.extend(exterior.lines());
    for interior in polygon.interiors() {
        lines.extend(interior.lines())
    }
    // Use index of the line to determine which parts we havent compared to yet
    for (index, line) in lines.clone().iter().enumerate() {
        for line2 in &lines.clone()[index + 1..] {
            if let Some(intersection) = line_intersection(*line, *line2) {
                let intersection_message = match intersection {
                    LineIntersection::Collinear { intersection } => {
                        Some(format!("Found collinear at {intersection:?}"))
                    }

                    LineIntersection::SinglePoint {
                        intersection,
                        is_proper: true,
                    } => Some(format!("Found self intersection at {intersection:?}")),
                    _ => None,
                };
                if let Some(error_message) = intersection_message {
                    errors.push(error_message);
                }
            }
        }
    }
    Validation {
        is_valid: errors.is_empty(),
        errors,
    }
}

pub fn validate_interiors_are_not_within<T: GeoFloat, Z: GeoFloat>(
    polygon: &Polygon<T, Z>,
) -> Validation {
    let mut errors: Vec<String> = vec![];
    let interiors = polygon.interiors();
    for interior in interiors {
        let polygon = Polygon::<T, Z>::new(interior.clone(), vec![]);
        for interior2 in interiors {
            // dont compare exactly the same interiors
            if interior == interior2 {
                continue;
            }
            let polygon2 = Polygon::<T, Z>::new(interior2.clone(), vec![]);
            if polygon.contains(&polygon2) {
                let error_message = format!(
                    "Interior ring {interior:?} is contains another interior ring {interior2:?}"
                );
                errors.push(error_message);
            }
        }
    }
    Validation {
        is_valid: errors.is_empty(),
        errors,
    }
}

fn to_faces<T: CoordNum, Z: CoordNum>(
    exterior: &LineString<T, Z>,
    interiors: &[LineString<T, Z>],
) -> Vec<Face<T, Z>> {
    let mut faces = vec![Face::new(exterior.coords().cloned().collect::<Vec<_>>())];
    for interior in interiors.iter() {
        faces.push(Face::new(interior.coords().cloned().collect::<Vec<_>>()));
    }
    faces
}

fn create_side_walls<T: CoordNum, Z: CoordNum>(
    bottom: &LineString<T, Z>,
    top: &LineString<T, Z>,
) -> Vec<Face<T, Z>> {
    let bottom_coords = bottom.coords().cloned().collect::<Vec<_>>();
    let top_coords = top.coords().cloned().collect::<Vec<_>>();
    bottom_coords
        .iter()
        .zip(bottom_coords.iter().skip(1))
        .zip(top_coords.iter().zip(top_coords.iter().skip(1)))
        .map(|((bottom_start, bottom_end), (top_start, top_end))| {
            Face::new(vec![*bottom_start, *bottom_end, *top_end, *top_start])
        })
        .collect()
}

fn to_side_faces<T: CoordNum, Z: CoordNum>(
    bottom_exterior: &LineString<T, Z>,
    top_exterior: &LineString<T, Z>,
    bottom_interiors: &[LineString<T, Z>],
    top_interiors: &[LineString<T, Z>],
) -> Vec<Face<T, Z>> {
    let mut faces = Vec::new();
    // Outer perimeter wall
    faces.extend(create_side_walls(bottom_exterior, top_exterior));

    // Inner perimeter wall
    for (bottom, top) in bottom_interiors.iter().zip(top_interiors) {
        faces.extend(create_side_walls(bottom, top));
    }
    faces
}

impl From<Polygon2D<f64>> for Vec<NaPoint2<f64>> {
    #[inline]
    fn from(p: Polygon2D<f64>) -> Vec<NaPoint2<f64>> {
        let result = p
            .rings()
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<Vec<NaPoint2<f64>>>>();
        result.into_iter().flatten().collect()
    }
}

impl From<Polygon3D<f64>> for Vec<NaPoint3<f64>> {
    #[inline]
    fn from(p: Polygon3D<f64>) -> Vec<NaPoint3<f64>> {
        let result = p
            .rings()
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<Vec<NaPoint3<f64>>>>();
        result.into_iter().flatten().collect()
    }
}

impl<T: CoordNum> From<Rect<T>> for Polygon<T, NoValue> {
    fn from(r: Rect<T>) -> Self {
        Polygon::new(
            vec![
                (r.min().x, r.min().y),
                (r.max().x, r.min().y),
                (r.max().x, r.max().y),
                (r.min().x, r.max().y),
                (r.min().x, r.min().y),
            ]
            .into(),
            Vec::new(),
        )
    }
}

impl<T: CoordNum, Z: CoordNum> From<Triangle<T, Z>> for Polygon<T, Z> {
    fn from(t: Triangle<T, Z>) -> Self {
        Self::new(vec![t.0, t.1, t.2, t.0].into(), Vec::new())
    }
}

impl<'a> From<NPolygon2<'a>> for Polygon2D<f64> {
    #[inline]
    fn from(poly: NPolygon2<'a>) -> Self {
        let interiors = poly.interiors().map(|interior| interior.into()).collect();
        Polygon2D::new(poly.exterior().into(), interiors)
    }
}

impl From<Polygon2D<f64>> for NPolygon2<'_> {
    #[inline]
    fn from(poly: Polygon2D<f64>) -> Self {
        let interiors: Vec<NLineString2> = poly
            .interiors()
            .iter()
            .map(|interior| interior.clone().into())
            .collect();
        let mut npoly = NPolygon2::new();
        let exterior: NLineString2 = poly.exterior().clone().into();
        npoly.add_ring(&exterior);
        for interior in interiors.iter() {
            npoly.add_ring(interior);
        }
        npoly
    }
}

impl From<Polygon3D<f64>> for NPolygon2<'_> {
    #[inline]
    fn from(poly: Polygon3D<f64>) -> Self {
        let interiors: Vec<NLineString2> = poly
            .interiors()
            .iter()
            .map(|interior| interior.clone().into())
            .collect();
        let mut npoly = NPolygon2::new();
        let exterior: NLineString2 = poly.exterior().clone().into();
        npoly.add_ring(&exterior);
        for interior in interiors.iter() {
            npoly.add_ring(interior);
        }
        npoly
    }
}

impl<'a> From<NPolygon3<'a>> for Polygon3D<f64> {
    #[inline]
    fn from(poly: NPolygon3<'a>) -> Self {
        let interiors = poly.interiors().map(|interior| interior.into()).collect();
        Polygon3D::new(poly.exterior().into(), interiors)
    }
}

impl From<Polygon3D<f64>> for NPolygon3<'_> {
    #[inline]
    fn from(poly: Polygon3D<f64>) -> Self {
        let interiors: Vec<NLineString3> = poly
            .interiors()
            .iter()
            .map(|interior| interior.clone().into())
            .collect();
        let mut npoly = NPolygon3::new();
        let exterior: NLineString3 = poly.exterior().clone().into();
        npoly.add_ring(&exterior);
        for interior in interiors.iter() {
            npoly.add_ring(interior);
        }
        npoly
    }
}

pub fn from_polygon_5d(polygon: &flatgeom::Polygon<[f64; 5]>) -> (Polygon3D<f64>, Polygon2D<f64>) {
    let (exterior3d, exterior2d) = from_line_string_5d(polygon.exterior());
    let mut interiors3d: Vec<LineString3D<f64>> = Default::default();
    let mut interiors2d: Vec<LineString2D<f64>> = Default::default();
    for interior in polygon.interiors() {
        let (interior3d, interior2d) = from_line_string_5d(interior);
        interiors3d.push(interior3d);
        interiors2d.push(interior2d);
    }
    let polygon3d = Polygon3D::new(exterior3d, interiors3d);
    let polygon2d = Polygon2D::new(exterior2d, interiors2d);
    (polygon3d, polygon2d)
}

impl<T: CoordFloat, Z: CoordFloat> From<Polygon<T, Z>> for geojson::Value {
    fn from(polygon: Polygon<T, Z>) -> Self {
        let coords = create_polygon_type(&polygon);
        geojson::Value::Polygon(coords)
    }
}

impl<T: CoordNum, Z: CoordNum> Surface for Polygon<T, Z> {}

impl<T, Z> RelativeEq for Polygon<T, Z>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
    Z: AbsDiffEq<Epsilon = Z> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if !self
            .exterior
            .relative_eq(&other.exterior, epsilon, max_relative)
        {
            return false;
        }

        if self.interiors.len() != other.interiors.len() {
            return false;
        }
        let mut zipper = self.interiors.iter().zip(other.interiors.iter());
        zipper.all(|(lhs, rhs)| lhs.relative_eq(rhs, epsilon, max_relative))
    }
}

impl<T: AbsDiffEq<Epsilon = T> + CoordNum, Z: AbsDiffEq<Epsilon = Z> + CoordNum> AbsDiffEq
    for Polygon<T, Z>
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if !self.exterior.abs_diff_eq(&other.exterior, epsilon) {
            return false;
        }

        if self.interiors.len() != other.interiors.len() {
            return false;
        }
        let mut zipper = self.interiors.iter().zip(other.interiors.iter());
        zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
    }
}

#[allow(dead_code)]
pub struct Iter<'a, T: CoordNum> {
    poly: &'a Polygon<T, T>,
    pos: usize,
}

impl<T: CoordNum> Iterator for Iter<'_, T> {
    type Item = LineString<T, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == 0 {
            self.pos += 1;
            Some(self.poly.exterior.clone())
        } else if self.pos <= self.poly.interiors.len() {
            let pos = self.pos - 1;
            self.pos += 1;
            Some(self.poly.interiors[pos].clone())
        } else {
            None
        }
    }
}

impl rstar::RTreeObject for Polygon2D<f64> {
    type Envelope = ::rstar::AABB<Point2D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        self.exterior.envelope()
    }
}

impl rstar::RTreeObject for Polygon3D<f64> {
    type Envelope = ::rstar::AABB<Point3D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        self.exterior.envelope()
    }
}

impl<T, Z> Elevation for Polygon<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.exterior.is_elevation_zero()
            && self.interiors.iter().all(LineString::is_elevation_zero)
    }
}

impl<Z: CoordFloat> Polygon<f64, Z> {
    pub fn approx_eq(&self, other: &Polygon<f64, Z>, epsilon: f64) -> bool {
        self.exterior.approx_eq(&other.exterior, epsilon)
            && self
                .interiors
                .iter()
                .zip(other.interiors.iter())
                .all(|(lhs, rhs)| lhs.approx_eq(rhs, epsilon))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Polygon2DFloat(pub Polygon2D<f64>);

impl Eq for Polygon2DFloat {}

impl PartialEq for Polygon2DFloat {
    fn eq(&self, other: &Self) -> bool {
        let epsilon = 0.001;
        if self.0.interiors().len() != other.0.interiors().len() {
            return false;
        }
        self.0.exterior().approx_eq(other.0.exterior(), epsilon)
            && self
                .0
                .interiors()
                .iter()
                .zip(other.0.interiors())
                .all(|(lhs, rhs)| lhs.approx_eq(rhs, epsilon))
    }
}

impl Hash for Polygon2DFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let precision_inverse = 1000.0; // Inverse of epsilon used in PartialEq
        for coord in self.0.exterior_coords_iter() {
            let hashed_coord = (
                (coord.x * precision_inverse).round() as i64,
                (coord.y * precision_inverse).round() as i64,
            );
            hashed_coord.hash(state);
        }
        for interior in self.0.interiors() {
            for coord in interior.coords_iter() {
                let hashed_coord = (
                    (coord.x * precision_inverse).round() as i64,
                    (coord.y * precision_inverse).round() as i64,
                );
                hashed_coord.hash(state);
            }
        }
    }
}

impl<T: CoordNum> From<Polygon2D<T>> for GeoPolygon<T> {
    fn from(polygon: Polygon2D<T>) -> Self {
        let exterior = polygon.exterior().clone().into();
        let interiors = polygon
            .interiors()
            .iter()
            .map(|interior| interior.clone().into())
            .collect();
        GeoPolygon::new(exterior, interiors)
    }
}

impl<T: CoordNum> From<GeoPolygon<T>> for Polygon2D<T> {
    fn from(polygon: GeoPolygon<T>) -> Self {
        let exterior = polygon.exterior().clone().into();
        let interiors = polygon
            .interiors()
            .iter()
            .map(|interior| interior.clone().into())
            .collect();
        Polygon2D::new(exterior, interiors)
    }
}

impl<T: CoordFloat> Polygon2D<T> {
    pub fn denormalize_vertices_2d(&mut self, avg: Coordinate2D<T>, norm: Coordinate2D<T>) {
        self.exterior.denormalize_vertices_2d(avg, norm);
        for interior in &mut self.interiors {
            interior.denormalize_vertices_2d(avg, norm);
        }
    }
}

impl<T: CoordFloat + From<Z>, Z: CoordFloat> Polygon<T, Z> {
    pub fn get_vertices(&self) -> Vec<&Coordinate<T, Z>> {
        let mut vertices = self.exterior.get_vertices();
        for interior in &self.interiors {
            vertices.extend(interior.get_vertices());
        }
        vertices
    }

    pub fn get_vertices_mut(&mut self) -> Vec<&mut Coordinate<T, Z>> {
        let mut vertices = self.exterior.get_vertices_mut();
        for interior in &mut self.interiors {
            vertices.extend(interior.get_vertices_mut());
        }
        vertices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::line_string::LineString3D;
    use std::f64::consts::TAU;

    #[test]
    fn test_into_merged_contour1() {
        let exterior = LineString3D::new(vec![
            Coordinate::new__(0_f64, 0_f64, 0_f64),
            Coordinate::new__(4.0, 0.0, 0.0),
            Coordinate::new__(4.0, 4.0, 0.0),
            Coordinate::new__(0.0, 4.0, 0.0),
            Coordinate::new__(0.0, 0.0, 0.0),
        ]);
        let interior = LineString3D::new(vec![
            Coordinate::new__(1.0, 1.0, 0.0),
            Coordinate::new__(1.0, 2.0, 0.0),
            Coordinate::new__(2.0, 2.0, 0.0),
            Coordinate::new__(2.0, 1.0, 0.0),
            Coordinate::new__(1.0, 1.0, 0.0),
        ]);
        let polygon = Polygon3D::new(exterior, vec![interior]);
        let merged = polygon.into_merged_contour().unwrap();
        let expected_coords = vec![
            Coordinate::new__(0.0, 0.0, 0.0),
            Coordinate::new__(4.0, 0.0, 0.0),
            Coordinate::new__(4.0, 4.0, 0.0),
            Coordinate::new__(0.0, 4.0, 0.0),
            Coordinate::new__(0.0, 0.0, 0.0),
            Coordinate::new__(1.0, 1.0, 0.0),
            Coordinate::new__(1.0, 2.0, 0.0),
            Coordinate::new__(2.0, 2.0, 0.0),
            Coordinate::new__(2.0, 1.0, 0.0),
            Coordinate::new__(1.0, 1.0, 0.0),
            Coordinate::new__(0.0, 0.0, 0.0),
        ];
        let expected = LineString3D::new(expected_coords);
        assert_eq!(merged, expected);
        let exterior_angle_sum = merged.exterior_angle_sum(None);
        assert!((exterior_angle_sum - TAU).abs() < 1e-5);
    }

    #[test]
    fn test_into_merged_contour2() {
        let exterior = LineString3D::new(vec![
            Coordinate::new__(-2.0, 0.0, 0.0),
            Coordinate::new__(0.0, 3.0, 0.0),
            Coordinate::new__(2.0, 0.0, 0.0),
            Coordinate::new__(-2.0, 0.0, 0.0),
        ]);
        let interior = LineString3D::new(vec![
            Coordinate::new__(0.0, 1.0, 0.0),
            Coordinate::new__(0.0, 2.0, 0.0),
            Coordinate::new__(0.0, 1.0, 0.0),
        ]);
        let polygon = Polygon3D::new(exterior, vec![interior]);
        let merged = polygon.into_merged_contour().unwrap();
        assert_eq!(merged.len(), 8);
        let expected_coords = vec![
            Coordinate::new__(-2.0, 0.0, 0.0),
            Coordinate::new__(0.0, 3.0, 0.0),
            Coordinate::new__(2.0, 0.0, 0.0),
            Coordinate::new__(-2.0, 0.0, 0.0),
            Coordinate::new__(0.0, 1.0, 0.0),
            Coordinate::new__(0.0, 2.0, 0.0),
            Coordinate::new__(0.0, 1.0, 0.0),
            Coordinate::new__(-2.0, 0.0, 0.0),
        ];
        let expected = LineString3D::new(expected_coords);
        assert_eq!(merged, expected);
        let exterior_angle_sum = merged.exterior_angle_sum(None);
        assert!((exterior_angle_sum - TAU).abs() < 1e-5);
    }

    #[test]
    fn test_into_merged_contour3() {
        let exterior = LineString3D::new(vec![
            Coordinate::new__(-1.2194539968515465, 0.16933105043042337, 0.0),
            Coordinate::new__(-1.055857735333338, 0.4778411954004563, 0.0),
            Coordinate::new__(-0.6853019923873935, 0.24672195911322675, 0.0),
            Coordinate::new__(-0.5833398829901584, 0.43943710577733136, 0.0),
            Coordinate::new__(0.09143713416665612, 0.01846906540020424, 0.0),
            Coordinate::new__(0.03360770119237577, -0.09070861028635324, 0.0),
            Coordinate::new__(-0.033488920040556945, -0.048611415950504736, 0.0),
            Coordinate::new__(-0.09436194563997304, -0.1635788079200148, 0.0),
            Coordinate::new__(-1.0344747782448054, 0.4224735580414215, 0.0),
            Coordinate::new__(-1.2094860785722195, 0.09411205118207576, 0.0),
            Coordinate::new__(-1.4199251562078496, 0.22535458947396547, 0.0),
            Coordinate::new__(-1.8095119541428824, -0.50911502605386, 0.0),
            Coordinate::new__(-1.6021226584354356, -0.6387069655611979, 0.0),
            Coordinate::new__(-1.8349611545790832, -1.0778998728746179, 0.0),
            Coordinate::new__(-1.4781306577310172, -1.2999381710149365, 0.0),
            Coordinate::new__(-1.2612714401132272, -0.8905213629396685, 0.0),
            Coordinate::new__(-0.4553518611871161, -1.3940324579852619, 0.0),
            Coordinate::new__(-0.48654868467646323, -1.4535836170526768, 0.0),
            Coordinate::new__(0.35977956630138175, -1.9818579435999786, 0.0),
            Coordinate::new__(0.24412029417678788, -2.1993865707687323, 0.0),
            Coordinate::new__(2.0069177989502758, -3.2988597200846757, 0.0),
            Coordinate::new__(2.9961112587714194, -1.434570505224098, 0.0),
            Coordinate::new__(2.5279636768865688, -1.1431932733038, 0.0),
            Coordinate::new__(2.6177519190084935, -0.973637427949112, 0.0),
            Coordinate::new__(1.8766429631621764, -0.5105684032750967, 0.0),
            Coordinate::new__(1.7883776549743646, -0.6784695400510318, 0.0),
            Coordinate::new__(1.3301400359839846, -0.39204397002019326, 0.0),
            Coordinate::new__(0.8713081665710084, -1.2580207963071683, 0.0),
            Coordinate::new__(0.7607521844701187, -1.1895108170904136, 0.0),
            Coordinate::new__(0.9572256678649496, -0.06999017268013079, 0.0),
            Coordinate::new__(0.848957065611682, -0.0031313800969762005, 0.0),
            Coordinate::new__(1.2583308533118243, 0.7685561964332482, 0.0),
            Coordinate::new__(1.073053192369885, 0.8841164426087409, 0.0),
            Coordinate::new__(1.3020897669743488, 1.315864417446708, 0.0),
            Coordinate::new__(1.0321781807408184, 1.4850789517386835, 0.0),
            Coordinate::new__(1.0983773918902138, 1.6107982125647209, 0.0),
            Coordinate::new__(0.7476455114122035, 1.829537078051839, 0.0),
            Coordinate::new__(0.7141648859428431, 1.7666770671892256, 0.0),
            Coordinate::new__(-0.2076513132318949, 2.3420011628583612, 0.0),
            Coordinate::new__(-0.34765878002084466, 2.0765014300828795, 0.0),
            Coordinate::new__(-0.5489486854778642, 2.2019661703255338, 0.0),
            Coordinate::new__(-0.7954853956507596, 1.7371333518039644, 0.0),
            Coordinate::new__(-0.7299133806140475, 1.695861478434628, 0.0),
            Coordinate::new__(-0.81209244465817, 1.5411927534099812, 0.0),
            Coordinate::new__(-0.863939717982187, 1.5733841404529636, 0.0),
            Coordinate::new__(-1.0725030617437434, 1.1797282100502866, 0.0),
            Coordinate::new__(-1.3228410916347892, 0.7074499089428343, 0.0),
            Coordinate::new__(-1.5114766353620608, 0.35175035192546195, 0.0),
            Coordinate::new__(-1.2194539968515465, 0.16933105043042337, 0.0),
        ]);
        let interior = LineString3D::new(vec![
            Coordinate::new__(-0.2924963013937458, -1.1119771975084152, 0.0),
            Coordinate::new__(0.11234391452351394, -0.39072103819093074, 0.0),
            Coordinate::new__(0.4882480268812336, -0.6400230272020863, 0.0),
            Coordinate::new__(0.36247518786088223, -1.5453647114996691, 0.0),
            Coordinate::new__(-0.2924963013937458, -1.1119771975084152, 0.0),
        ]);
        let polygon = Polygon3D::new(exterior, vec![interior]);
        let merged = polygon.into_merged_contour().unwrap();
        let exterior_angle_sum = merged.exterior_angle_sum(None);
        assert!((exterior_angle_sum - TAU).abs() < 1e-5);
    }
}
