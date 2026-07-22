use super::Csg;
use crate::validation_next::Validate;

// A `Csg` holds no coordinates of its own; the driver validates it by recursing
// into its operands (see `validation_next::validate`), so it declares no direct
// checks and inherits every `Validate` default.
impl Validate for Csg {}
