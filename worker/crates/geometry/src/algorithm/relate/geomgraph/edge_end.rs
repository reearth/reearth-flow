use crate::algorithm::kernels::{Orientation, RobustKernel};
use crate::algorithm::GeoFloat;
use crate::types::coordinate::Coordinate;

use super::{CoordNode, Edge, Label, Quadrant};

use std::cell::RefCell;
use std::fmt;

/// Models the end of an edge incident on a node.
///
/// EdgeEnds have a direction determined by the direction of the ray from the initial
/// point to the next point.
///
/// EdgeEnds are comparable by their EdgeEndKey, under the ordering
/// "a has a greater angle with the x-axis than b".
///
/// This ordering is used to sort EdgeEnds around a node.
///
/// This is based on [JTS's EdgeEnd as of 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/EdgeEnd.java)
#[derive(Clone, Debug)]
pub(crate) struct EdgeEnd<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    label: Label,
    key: EdgeEndKey<T, Z>,
}

#[derive(Clone)]
pub(crate) struct EdgeEndKey<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    coord_0: Coordinate<T, Z>,
    coord_1: Coordinate<T, Z>,
    delta: Coordinate<T, Z>,
    quadrant: Option<Quadrant>,
}

impl<T: GeoFloat, Z: GeoFloat> fmt::Debug for EdgeEndKey<T, Z> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EdgeEndKey")
            .field(
                "coords",
                &format!("{:?} -> {:?}", &self.coord_0, &self.coord_1),
            )
            .field("quadrant", &self.quadrant)
            .finish()
    }
}

impl<T, Z> EdgeEnd<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    pub fn new(
        coord_0: Coordinate<T, Z>,
        coord_1: Coordinate<T, Z>,
        label: Label,
    ) -> EdgeEnd<T, Z> {
        let delta = coord_1 - coord_0;
        let quadrant = Quadrant::new(delta.x, delta.y);
        EdgeEnd {
            label,
            key: EdgeEndKey {
                coord_0,
                coord_1,
                delta,
                quadrant,
            },
        }
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    pub fn coordinate(&self) -> &Coordinate<T, Z> {
        &self.key.coord_0
    }

    pub fn key(&self) -> &EdgeEndKey<T, Z> {
        &self.key
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::Eq for EdgeEndKey<T, Z> {}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::PartialEq for EdgeEndKey<T, Z> {
    fn eq(&self, other: &EdgeEndKey<T, Z>) -> bool {
        self.delta == other.delta
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::PartialOrd for EdgeEndKey<T, Z> {
    fn partial_cmp(&self, other: &EdgeEndKey<T, Z>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::Ord for EdgeEndKey<T, Z> {
    fn cmp(&self, other: &EdgeEndKey<T, Z>) -> std::cmp::Ordering {
        self.compare_direction(other)
    }
}

impl<T: GeoFloat, Z: GeoFloat> EdgeEndKey<T, Z> {
    pub(crate) fn compare_direction(&self, other: &EdgeEndKey<T, Z>) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if self.delta == other.delta {
            return Ordering::Equal;
        }

        match (self.quadrant, other.quadrant) {
            (Some(q1), Some(q2)) if q1 > q2 => Ordering::Greater,
            (Some(q1), Some(q2)) if q1 < q2 => Ordering::Less,
            _ => {
                match RobustKernel::orient(
                    other.coord_0,
                    other.coord_1,
                    self.coord_0,
                    Some(self.coord_1),
                ) {
                    Orientation::Clockwise => Ordering::Less,
                    Orientation::CounterClockwise => Ordering::Greater,
                    Orientation::Collinear => Ordering::Equal,
                }
            }
        }
    }
}
