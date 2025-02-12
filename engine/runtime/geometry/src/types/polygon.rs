use std::hash::{Hash, Hasher};

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

use crate::algorithm::contains::Contains;
use crate::algorithm::coords_iter::CoordsIter;
use crate::algorithm::line_intersection::{line_intersection, LineIntersection};
use crate::algorithm::GeoFloat;

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

    pub fn exteriors_push(&mut self, new_exterior: impl Into<LineString<T, Z>>) {
        let mut new_exterior = new_exterior.into();
        new_exterior.close();
        self.exterior = new_exterior;
    }

    pub fn area(&self) -> f64 {
        let mut area = 0.0;
        area += self.exterior().ring_area();
        for interior in self.interiors() {
            area -= interior.ring_area();
        }
        area
    }

    pub fn add_ring(&mut self, linestring: LineString<T, Z>) {
        self.exteriors_push(linestring.clone());
        self.interiors_push(linestring.clone());
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
        Solid::new(bottom_faces, top_faces, side_faces)
    }

    pub fn validate_rings_length(&self) -> Validation {
        let mut errors: Vec<String> = vec![];

        let exterior = self.exterior();
        if exterior.coords().count() < 3 {
            let error_message =
                format!("Exterior Ring {:?} must contain 3 or more coords", exterior);
            errors.push(error_message);
        }
        for interior in self.interiors() {
            if interior.coords().count() < 3 {
                let error_message =
                    format!("Interior Ring {:?} must contain 3 or more coords", interior);
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
            let error_message = format!("Exterior ring {:?} is not closed", exterior);
            errors.push(error_message);
        }
        for interior in self.interiors() {
            if !interior.is_closed() {
                let error_message = format!("Interior ring {:?} is not closed", interior);
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
            let error_message = format!("Exterior ring {:?} is not closed", exterior);
            errors.push(error_message);
        }
        for interior in self.interiors() {
            if !interior.is_closed() {
                let error_message = format!("Interior ring {:?} is not closed", interior);
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
                        Some(format!("Found collinear at {:?}", intersection))
                    }

                    LineIntersection::SinglePoint {
                        intersection,
                        is_proper: true,
                    } => Some(format!("Found self intersection at {:?}", intersection)),
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
                    "Interior ring {:?} is contains another interior ring {:?}",
                    interior, interior2
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
