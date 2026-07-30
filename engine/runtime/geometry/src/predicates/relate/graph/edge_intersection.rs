/// A point on an [`Edge`](super::Edge) where another edge crosses it, keyed by
/// its position along the edge (segment index, then distance within the
/// segment) so the edge's intersections sort in traversal order.
#[derive(Debug)]
pub(crate) struct EdgeIntersection {
    coord: [f64; 2],
    segment_index: usize,
    dist: f64,
}

impl EdgeIntersection {
    pub fn new(coord: [f64; 2], segment_index: usize, dist: f64) -> EdgeIntersection {
        EdgeIntersection {
            coord,
            segment_index,
            dist,
        }
    }

    pub fn coordinate(&self) -> [f64; 2] {
        self.coord
    }

    pub fn segment_index(&self) -> usize {
        self.segment_index
    }

    pub fn distance(&self) -> f64 {
        self.dist
    }
}

impl std::cmp::PartialEq for EdgeIntersection {
    fn eq(&self, other: &EdgeIntersection) -> bool {
        self.segment_index == other.segment_index && self.dist == other.dist
    }
}

impl std::cmp::Eq for EdgeIntersection {}

impl std::cmp::PartialOrd for EdgeIntersection {
    fn partial_cmp(&self, other: &EdgeIntersection) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for EdgeIntersection {
    fn cmp(&self, other: &EdgeIntersection) -> std::cmp::Ordering {
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

        // BTreeSet requires nodes to be fully `Ord`, but we're comparing floats, so we require
        // non-NaN for valid results.
        debug_assert!(!self.dist.is_nan() && !other.dist.is_nan());

        std::cmp::Ordering::Equal
    }
}
