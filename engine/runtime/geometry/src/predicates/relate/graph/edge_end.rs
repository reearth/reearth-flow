use crate::predicates::kernel::{orient2d, Orientation};

use super::{Label, Quadrant};

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
pub(crate) struct EdgeEnd {
    label: Label,
    key: EdgeEndKey,
}

#[derive(Clone)]
pub(crate) struct EdgeEndKey {
    coord_0: [f64; 2],
    coord_1: [f64; 2],
    delta: [f64; 2],
    quadrant: Option<Quadrant>,
}

impl fmt::Debug for EdgeEndKey {
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

impl EdgeEnd {
    pub fn new(coord_0: [f64; 2], coord_1: [f64; 2], label: Label) -> EdgeEnd {
        let delta = [coord_1[0] - coord_0[0], coord_1[1] - coord_0[1]];
        let quadrant = Quadrant::new(delta[0], delta[1]);
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

    pub fn coordinate(&self) -> &[f64; 2] {
        &self.key.coord_0
    }

    pub fn key(&self) -> &EdgeEndKey {
        &self.key
    }
}

impl std::cmp::Eq for EdgeEndKey {}

impl std::cmp::PartialEq for EdgeEndKey {
    /// Consistent with [`Ord`]: two keys are equal exactly when they point in
    /// the same direction, so the `Ord`/`Eq` contract holds.
    fn eq(&self, other: &EdgeEndKey) -> bool {
        self.compare_direction(other) == std::cmp::Ordering::Equal
    }
}

impl std::cmp::PartialOrd for EdgeEndKey {
    fn partial_cmp(&self, other: &EdgeEndKey) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for EdgeEndKey {
    fn cmp(&self, other: &EdgeEndKey) -> std::cmp::Ordering {
        self.compare_direction(other)
    }
}

impl EdgeEndKey {
    /// Compares the directions of two edge ends (JTS `EdgeEnd::compareDirection`):
    /// same-quadrant ties break on the robust orientation of this end's
    /// direction point against the other end's ray.
    pub(crate) fn compare_direction(&self, other: &EdgeEndKey) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if self.delta == other.delta {
            return Ordering::Equal;
        }

        match (self.quadrant, other.quadrant) {
            (Some(q1), Some(q2)) if q1 > q2 => Ordering::Greater,
            (Some(q1), Some(q2)) if q1 < q2 => Ordering::Less,
            _ => match orient2d(other.coord_0, other.coord_1, self.coord_1) {
                Orientation::Clockwise => Ordering::Less,
                Orientation::CounterClockwise => Ordering::Greater,
                Orientation::Collinear => Ordering::Equal,
            },
        }
    }
}
