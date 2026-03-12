use num_traits::NumCast;
use reearth_flow_geometry::types::coordnum::CoordNum;

pub(super) fn finite_z<Z: CoordNum>(z: Z) -> Option<f64> {
    NumCast::from(z).filter(|z: &f64| z.is_finite())
}
