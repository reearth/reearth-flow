use super::PointCloud;
use crate::ops::validation::{Validate, ValidationReport};
use crate::ops::ValidationType;

impl Validate for PointCloud {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        // TODO(new-geometry validation): a point cloud stores its positions in a
        // packed little-endian byte stream (see `point_cloud`), so finiteness
        // has to decode each segment's XYZ through the position accessors rather
        // than scan a flat `[f64; 3]` buffer. Until those accessors are wired
        // through, a point cloud is treated as valid.
        None
    }
}
