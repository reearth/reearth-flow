use crate::{
    algorithm::{line_intersection::LineIntersection, GeoFloat},
    types::line::Line,
};

pub(crate) trait LineIntersector<T: GeoFloat, Z: GeoFloat> {
    fn compute_intersection(
        &mut self,
        l1: Line<T, Z>,
        l2: Line<T, Z>,
    ) -> Option<LineIntersection<T, Z>>;
}
