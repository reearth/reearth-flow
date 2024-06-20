use std::fmt::Display;

use reearth_flow_common::collection::ApproxHashSet;
use serde::{Deserialize, Serialize};

use crate::{
    algorithm::GeoNum,
    types::{
        coordinate::Coordinate, face::Face, geometry::Geometry, line::Line,
        line_string::LineString, multi_line_string::MultiLineString, multi_point::MultiPoint,
        multi_polygon::MultiPolygon, point::Point, polygon::Polygon, rect::Rect, solid::Solid,
    },
    utils,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// The role of a ring in a polygon.
pub enum RingRole {
    Exterior,
    Interior(isize),
}

impl std::fmt::Display for RingRole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RingRole::Exterior => write!(f, "exterior ring"),
            RingRole::Interior(i) => write!(f, "interior ring n°{}", i),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// The position of the problem in a multi-geometry, starting at 0.
pub struct GeometryPosition(isize);

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// The coordinate position of the problem in the geometry.
/// If the value is 0 or more, it is the index of the coordinate.
/// If the value is -1 it indicates that the coordinate position is not relevant or unknown.
pub struct CoordinatePosition(isize);

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// The position of the problem in the geometry.
pub enum ValidationProblemPosition {
    Point,
    Line(CoordinatePosition),
    Triangle(CoordinatePosition),
    Rect(RingRole, CoordinatePosition),
    MultiPoint(GeometryPosition),
    LineString(CoordinatePosition),
    MultiLineString(GeometryPosition, CoordinatePosition),
    Polygon(RingRole, CoordinatePosition),
    MultiPolygon(GeometryPosition, RingRole, CoordinatePosition),
    Face(GeometryPosition),
    Solid(GeometryPosition),
    GeometryCollection(GeometryPosition, Box<ValidationProblemPosition>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// The type of problem encountered.
pub enum ValidationProblem {
    /// A coordinate is not finite (NaN or infinite)
    NotFinite,
    /// A LineString or a Polygon ring has too few points
    TooFewPoints,
    /// Identical coords
    IdenticalCoords,
    /// Collinear coords
    CollinearCoords,
    /// A ring has a self-intersection
    SelfIntersection,
    /// Two interior rings of a Polygon share a common line
    IntersectingRingsOnALine,
    /// Two interior rings of a Polygon share a common area
    IntersectingRingsOnAnArea,
    /// The interior ring of a Polygon is not contained in the exterior ring
    InteriorRingNotContainedInExteriorRing,
    /// Two Polygons of a MultiPolygon overlap partially
    ElementsOverlaps,
    /// Two Polygons of a MultiPolygon touch on a line
    ElementsTouchOnALine,
    /// Two Polygons of a MultiPolygon are identical
    ElementsAreIdentical,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// A problem, at a given position, encountered when checking the validity of a geometry.
pub struct ValidationProblemAtPosition(pub ValidationProblem, pub ValidationProblemPosition);

impl Display for ValidationProblemAtPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {:?}", self.0, self.1)
    }
}

/// All the problems encountered when checking the validity of a geometry.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ValidationProblemReport(pub Vec<ValidationProblemAtPosition>);

impl ValidationProblemReport {
    /// The number of problems encountered.
    pub fn error_count(&self) -> usize {
        self.0.len()
    }

    pub fn reports(&self) -> Vec<ValidationProblemAtPosition> {
        self.0.clone()
    }
}

impl Display for ValidationProblemPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_buffer: Vec<String> = Vec::new();
        match self {
            ValidationProblemPosition::Point => str_buffer.push(String::new()),
            ValidationProblemPosition::LineString(coord) => {
                if coord.0 == -1 {
                    str_buffer.push(String::new())
                } else {
                    str_buffer.push(format!(" at coordinate {} of the LineString", coord.0))
                }
            }
            ValidationProblemPosition::Triangle(coord) => {
                if coord.0 == -1 {
                    str_buffer.push(String::new())
                } else {
                    str_buffer.push(format!(" at coordinate {} of the Triangle", coord.0))
                }
            }
            ValidationProblemPosition::Polygon(ring_role, coord) => {
                if coord.0 == -1 {
                    str_buffer.push(format!(" on the {}", ring_role))
                } else {
                    str_buffer.push(format!(" at coordinate {} of the {}", coord.0, ring_role))
                }
            }
            ValidationProblemPosition::MultiPolygon(geom_number, ring_role, coord) => {
                if coord.0 == -1 {
                    str_buffer.push(format!(
                        " on the {} of the Polygon n°{} of the MultiPolygon",
                        ring_role, geom_number.0
                    ))
                } else {
                    str_buffer.push(format!(
                        " at coordinate {} of the {} of the Polygon n°{} of the MultiPolygon",
                        coord.0, ring_role, geom_number.0
                    ))
                }
            }
            ValidationProblemPosition::MultiLineString(geom_number, coord) => {
                if coord.0 == -1 {
                    str_buffer.push(format!(
                        " on the LineString n°{} of the MultiLineString",
                        geom_number.0
                    ))
                } else {
                    str_buffer.push(format!(
                        " at coordinate {} of the LineString n°{} of the MultiLineString",
                        coord.0, geom_number.0
                    ))
                }
            }
            ValidationProblemPosition::MultiPoint(geom_number) => str_buffer.push(format!(
                " on the Point n°{} of the MultiPoint",
                geom_number.0
            )),
            ValidationProblemPosition::GeometryCollection(geom_number, problem_position) => {
                str_buffer.push(format!(
                    "{} of the geometry n°{} of the GeometryCollection",
                    *problem_position, geom_number.0
                ));
            }
            ValidationProblemPosition::Rect(ring_role, coord) => {
                if coord.0 == -1 {
                    str_buffer.push(format!(
                        " on the {} of the Polygon n°{} of the MultiPolygon",
                        ring_role, coord.0
                    ))
                } else {
                    str_buffer.push(format!(
                        " at coordinate {} of the {} of the Polygon n°{} of the MultiPolygon",
                        coord.0, ring_role, coord.0
                    ))
                }
            }
            ValidationProblemPosition::Line(coord) => {
                if coord.0 == -1 {
                    str_buffer.push(String::new())
                } else {
                    str_buffer.push(format!(" at coordinate {} of the Line", coord.0))
                }
            }
            ValidationProblemPosition::Face(geom_number) => {
                str_buffer.push(format!(" on the Face n°{} of the Solid", geom_number.0))
            }
            ValidationProblemPosition::Solid(geom_number) => {
                str_buffer.push(format!(" on the Solid n°{}", geom_number.0))
            }
        }
        write!(f, "{}", str_buffer.join(""))
    }
}

impl Display for ValidationProblemReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buffer = self
            .0
            .iter()
            .map(|p| {
                let (problem, position) = (&p.0, &p.1);
                let mut str_buffer: Vec<String> = Vec::new();
                let is_polygon = matches!(
                    position,
                    ValidationProblemPosition::Polygon(_, _)
                        | ValidationProblemPosition::MultiPolygon(_, _, _)
                );

                str_buffer.push(format!("{}", position));

                match *problem {
                    ValidationProblem::NotFinite => {
                        str_buffer.push("Coordinate is not finite (NaN or infinite)".to_string())
                    }
                    ValidationProblem::TooFewPoints => {
                        if is_polygon {
                            str_buffer.push("Polygon ring has too few points".to_string())
                        } else {
                            str_buffer.push("LineString has too few points".to_string())
                        }
                    }
                    ValidationProblem::IdenticalCoords => {
                        str_buffer.push("Identical coords".to_string())
                    }
                    ValidationProblem::CollinearCoords => {
                        str_buffer.push("Collinear coords".to_string())
                    }
                    ValidationProblem::SelfIntersection => {
                        str_buffer.push("Ring has a self-intersection".to_string())
                    }
                    ValidationProblem::IntersectingRingsOnALine => str_buffer
                        .push("Two interior rings of a Polygon share a common line".to_string()),
                    ValidationProblem::IntersectingRingsOnAnArea => str_buffer
                        .push("Two interior rings of a Polygon share a common area".to_string()),
                    ValidationProblem::InteriorRingNotContainedInExteriorRing => str_buffer.push(
                        "The interior ring of a Polygon is not contained in the exterior ring"
                            .to_string(),
                    ),
                    ValidationProblem::ElementsOverlaps => str_buffer
                        .push("Two Polygons of MultiPolygons overlap partially".to_string()),
                    ValidationProblem::ElementsTouchOnALine => {
                        str_buffer.push("Two Polygons of MultiPolygons touch on a line".to_string())
                    }
                    ValidationProblem::ElementsAreIdentical => {
                        str_buffer.push("Two Polygons of MultiPolygons are identical".to_string())
                    }
                };
                str_buffer.into_iter().rev().collect::<Vec<_>>().join("")
            })
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}", buffer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationType {
    DuplicatePoints,
    CorruptGeometry,
    SelfIntersection,
}

pub trait Validator<
    T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport>;
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Coordinate<T, Z>
{
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationProblemReport> {
        if utils::check_coord_is_not_finite(self) {
            return Some(ValidationProblemReport(vec![ValidationProblemAtPosition(
                ValidationProblem::NotFinite,
                ValidationProblemPosition::Point,
            )]));
        }
        None
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Point<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        self.0.validate(valid_type)
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for MultiPoint<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                let mut seen = ApproxHashSet::<Coordinate<T, Z>>::new();
                for (idx, pt) in self.0.iter().enumerate() {
                    if let Some(result) = pt.validate(valid_type.clone()) {
                        for problem in &result.0 {
                            reason.push(ValidationProblemAtPosition(
                                problem.0.clone(),
                                ValidationProblemPosition::MultiPoint(GeometryPosition(
                                    idx as isize,
                                )),
                            ));
                        }
                    }
                    if !seen.insert(pt.0) {
                        reason.push(ValidationProblemAtPosition(
                            ValidationProblem::IdenticalCoords,
                            ValidationProblemPosition::MultiPoint(GeometryPosition(idx as isize)),
                        ));
                    }
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
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
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                if let Some(result) = self.start.validate(valid_type.clone()) {
                    for problem in &result.0 {
                        reason.push(ValidationProblemAtPosition(
                            problem.0.clone(),
                            ValidationProblemPosition::Line(CoordinatePosition(0)),
                        ));
                    }
                }
                if let Some(result) = self.end.validate(valid_type.clone()) {
                    for problem in &result.0 {
                        reason.push(ValidationProblemAtPosition(
                            problem.0.clone(),
                            ValidationProblemPosition::Line(CoordinatePosition(1)),
                        ));
                    }
                }
                if self.start == self.end {
                    reason.push(ValidationProblemAtPosition(
                        ValidationProblem::IdenticalCoords,
                        ValidationProblemPosition::Line(CoordinatePosition(-1)),
                    ));
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for LineString<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                let mut seen = ApproxHashSet::<Coordinate<T, Z>>::new();
                for (idx, pt) in self.0.iter().enumerate() {
                    if let Some(result) = pt.validate(valid_type.clone()) {
                        for problem in &result.0 {
                            reason.push(ValidationProblemAtPosition(
                                problem.0.clone(),
                                ValidationProblemPosition::Line(CoordinatePosition(1)),
                            ));
                        }
                    }
                    if !seen.insert(*pt) {
                        reason.push(ValidationProblemAtPosition(
                            ValidationProblem::IdenticalCoords,
                            ValidationProblemPosition::LineString(CoordinatePosition(idx as isize)),
                        ));
                    }
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for MultiLineString<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                for line_string in &self.0 {
                    if let Some(result) = line_string.validate(valid_type.clone()) {
                        for (idx, problem) in result.0.iter().enumerate() {
                            reason.push(ValidationProblemAtPosition(
                                problem.0.clone(),
                                ValidationProblemPosition::MultiLineString(
                                    GeometryPosition(0),
                                    CoordinatePosition(idx as isize),
                                ),
                            ));
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Polygon<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                for line_string in &self.rings() {
                    if let Some(result) = line_string.validate(valid_type.clone()) {
                        for (idx, problem) in result.0.iter().enumerate() {
                            reason.push(ValidationProblemAtPosition(
                                problem.0.clone(),
                                ValidationProblemPosition::Polygon(
                                    RingRole::Exterior,
                                    CoordinatePosition(idx as isize),
                                ),
                            ));
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for MultiPolygon<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                for (idx, polygon) in self.0.iter().enumerate() {
                    if let Some(result) = polygon.validate(valid_type.clone()) {
                        for (idx2, problem) in result.0.iter().enumerate() {
                            reason.push(ValidationProblemAtPosition(
                                problem.0.clone(),
                                ValidationProblemPosition::MultiPolygon(
                                    GeometryPosition(idx as isize),
                                    RingRole::Exterior,
                                    CoordinatePosition(idx2 as isize),
                                ),
                            ));
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Face<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                let mut seen = ApproxHashSet::<Coordinate<T, Z>>::new();
                for (idx, pt) in self.0.iter().enumerate() {
                    if let Some(result) = pt.validate(valid_type.clone()) {
                        for problem in &result.0 {
                            reason.push(ValidationProblemAtPosition(
                                problem.0.clone(),
                                ValidationProblemPosition::Face(GeometryPosition(idx as isize)),
                            ));
                        }
                    }
                    if !seen.insert(*pt) {
                        reason.push(ValidationProblemAtPosition(
                            ValidationProblem::IdenticalCoords,
                            ValidationProblemPosition::Face(GeometryPosition(idx as isize)),
                        ));
                    }
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Solid<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                for (idx, face) in self.all_faces().iter().enumerate() {
                    if let Some(result) = face.validate(valid_type.clone()) {
                        for problem in result.0.iter() {
                            reason.push(ValidationProblemAtPosition(
                                problem.0.clone(),
                                ValidationProblemPosition::Solid(GeometryPosition(idx as isize)),
                            ));
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Rect<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        let mut reason = Vec::new();
        match valid_type {
            ValidationType::DuplicatePoints => {
                let polygon = self.to_polygon();
                if let Some(result) = polygon.validate(valid_type.clone()) {
                    for problem in result.0.iter() {
                        reason.push(ValidationProblemAtPosition(
                            problem.0.clone(),
                            ValidationProblemPosition::Rect(
                                RingRole::Exterior,
                                CoordinatePosition(-1),
                            ),
                        ));
                    }
                }
            }
            _ => unimplemented!(),
        }
        if reason.is_empty() {
            None
        } else {
            Some(ValidationProblemReport(reason))
        }
    }
}

impl<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64>,
    > Validator<T, Z> for Geometry<T, Z>
{
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationProblemReport> {
        match self {
            Geometry::Point(p) => p.validate(valid_type),
            Geometry::Line(l) => l.validate(valid_type),
            Geometry::LineString(ls) => ls.validate(valid_type),
            Geometry::Polygon(p) => p.validate(valid_type),
            Geometry::MultiPoint(mp) => mp.validate(valid_type),
            Geometry::MultiLineString(mls) => mls.validate(valid_type),
            Geometry::MultiPolygon(mp) => mp.validate(valid_type),
            Geometry::Rect(rect) => rect.validate(valid_type),
            Geometry::Triangle(_) => unimplemented!(),
            Geometry::Solid(s) => s.validate(valid_type),
            Geometry::GeometryCollection(gc) => {
                let mut reason = Vec::new();
                for geom in gc {
                    if let Some(result) = geom.validate(valid_type.clone()) {
                        for problem in result.0.iter() {
                            reason.push(ValidationProblemAtPosition(
                                problem.0.clone(),
                                ValidationProblemPosition::GeometryCollection(
                                    GeometryPosition(0),
                                    Box::new(problem.1.clone()),
                                ),
                            ));
                        }
                    }
                }
                if reason.is_empty() {
                    None
                } else {
                    Some(ValidationProblemReport(reason))
                }
            }
        }
    }
}
