use reearth_flow_common::collection::ApproxHashSet;

use crate::{
    algorithm::GeoNum,
    types::{
        coordinate::Coordinate, face::Face, geometry::Geometry, line::Line,
        line_string::LineString, multi_line_string::MultiLineString, multi_point::MultiPoint,
        multi_polygon::MultiPolygon, point::Point, polygon::Polygon, rect::Rect, solid::Solid,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationType {
    DuplicatePoints,
}

pub trait Validator<
    T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
>
{
    fn validate(&self, valid_type: ValidationType) -> bool;
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Coordinate<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => true,
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Point<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => self.0.validate(valid_type),
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for MultiPoint<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => {
                let mut seen = ApproxHashSet::<Coordinate<T, Z>>::new();
                for pt in &self.0 {
                    if !seen.insert(pt.0) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => self.start == self.end,
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for LineString<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => {
                let mut seen = ApproxHashSet::<Coordinate<T, Z>>::new();
                for pt in &self.0 {
                    if !seen.insert(*pt) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for MultiLineString<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => {
                for line_string in &self.0 {
                    if !line_string.validate(valid_type.clone()) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Polygon<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => {
                for line_string in &self.rings() {
                    if !line_string.validate(valid_type.clone()) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for MultiPolygon<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => {
                for polygon in &self.0 {
                    if !polygon.validate(valid_type.clone()) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Face<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => {
                let mut seen = ApproxHashSet::<Coordinate<T, Z>>::new();
                for pt in &self.0 {
                    if !seen.insert(*pt) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Solid<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => {
                for face in &self.all_faces() {
                    if !face.validate(valid_type.clone()) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Rect<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match valid_type {
            ValidationType::DuplicatePoints => {
                let polygon = self.to_polygon();
                polygon.validate(valid_type)
            }
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Geometry<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> bool {
        match self {
            Geometry::Point(p) => p.validate(valid_type),
            Geometry::Line(l) => l.validate(valid_type),
            Geometry::LineString(ls) => ls.validate(valid_type),
            Geometry::Polygon(p) => p.validate(valid_type),
            Geometry::MultiPoint(mp) => mp.validate(valid_type),
            Geometry::MultiLineString(mls) => mls.validate(valid_type),
            Geometry::MultiPolygon(mp) => mp.validate(valid_type),
            Geometry::Rect(rect) => rect.validate(valid_type),
            Geometry::Triangle(_) => true,
            Geometry::Solid(s) => s.validate(valid_type),
            Geometry::GeometryCollection(gc) => {
                for geom in gc {
                    if !geom.validate(valid_type.clone()) {
                        return false;
                    }
                }
                true
            }
        }
    }
}
