use crate::{algorithm::GeoFloat, types::coordinate::Coordinate};

#[derive(Debug)]
pub(crate) struct EdgeIntersection<T: GeoFloat, Z: GeoFloat> {
    coord: Coordinate<T, Z>,
    segment_index: usize,
    dist: T,
}

impl<T: GeoFloat, Z: GeoFloat> EdgeIntersection<T, Z> {
    pub fn new(coord: Coordinate<T, Z>, segment_index: usize, dist: T) -> EdgeIntersection<T, Z> {
        EdgeIntersection {
            coord,
            segment_index,
            dist,
        }
    }

    pub fn coordinate(&self) -> Coordinate<T, Z> {
        self.coord
    }

    pub fn segment_index(&self) -> usize {
        self.segment_index
    }

    pub fn distance(&self) -> T {
        self.dist
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::PartialEq for EdgeIntersection<T, Z> {
    fn eq(&self, other: &EdgeIntersection<T, Z>) -> bool {
        self.segment_index == other.segment_index && self.dist == other.dist
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::Eq for EdgeIntersection<T, Z> {}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::PartialOrd for EdgeIntersection<T, Z> {
    fn partial_cmp(&self, other: &EdgeIntersection<T, Z>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: GeoFloat, Z: GeoFloat> std::cmp::Ord for EdgeIntersection<T, Z> {
    fn cmp(&self, other: &EdgeIntersection<T, Z>) -> std::cmp::Ordering {
        if self.segment_index < other.segment_index {
            return std::cmp::Ordering::Less;
        }
        if self.segment_index > other.segment_index {
            return std::cmp::Ordering::Greater;
        }
        if self.dist < other.dist {
            return std::cmp::Ordering::Less;
        }
        if self.dist > other.dist {
            return std::cmp::Ordering::Greater;
        }

        // BTreeMap requires nodes to be fully `Ord`, but we're comparing floats, so we require
        // non-NaN for valid results.
        debug_assert!(!self.dist.is_nan() && !other.dist.is_nan());

        std::cmp::Ordering::Equal
    }
}

impl<T: GeoFloat, Z: GeoFloat> EdgeIntersection<T, Z> {}
