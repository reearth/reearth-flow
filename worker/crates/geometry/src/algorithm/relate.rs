pub(crate) use edge_end_builder::EdgeEndBuilder;
pub use geomgraph::intersection_matrix::IntersectionMatrix;

use crate::types::geometry::Geometry;
use crate::types::{
    geometry_collection::GeometryCollection, line::Line, line_string::LineString,
    multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon,
    point::Point, polygon::Polygon, rect::Rect, triangle::Triangle,
};

use super::{geometry_cow::GeometryCow, GeoFloat};

mod edge_end_builder;
mod geomgraph;
mod relate_operation;

pub trait Relate<T, F, Z> {
    fn relate(&self, other: &F) -> IntersectionMatrix;
}

impl<T: GeoFloat, Z: GeoFloat> Relate<T, GeometryCow<'_, T, Z>, Z> for GeometryCow<'_, T, Z> {
    fn relate(&self, other: &GeometryCow<T, Z>) -> IntersectionMatrix {
        let mut relate_computer = relate_operation::RelateOperation::new(self, other);
        relate_computer.compute_intersection_matrix()
    }
}

macro_rules! relate_impl {
    ($k:ty, $t:ty) => {
        relate_impl![($k, $t),];
    };
    ($(($k:ty, $t:ty),)*) => {
        $(
            impl<T: GeoFloat, Z: GeoFloat> Relate<T, $t, Z> for $k {
                fn relate(&self, other: &$t) -> IntersectionMatrix {
                    GeometryCow::from(self).relate(&GeometryCow::from(other))
                }
            }
        )*
    };
}

/// Call the given macro with every pair of inputs
///
/// # Examples
///
/// ```ignore
/// cartesian_pairs!(foo, [Bar, Baz, Qux]);
/// ```
/// Is akin to calling:
/// ```ignore
/// foo![(Bar, Bar), (Bar, Baz), (Bar, Qux), (Baz, Bar), (Baz, Baz), (Baz, Qux), (Qux, Bar), (Qux, Baz), (Qux, Qux)];
/// ```
macro_rules! cartesian_pairs {
    ($macro_name:ident, [$($a:ty),*]) => {
        cartesian_pairs_helper! { [] [$($a,)*] [$($a,)*] [$($a,)*] $macro_name}
    };
}

macro_rules! cartesian_pairs_helper {
    // popped all a's - we're done. Use the accumulated output as the input to relate macro.
    ([$($out_pairs:tt)*] [] [$($b:ty,)*] $init_b:tt $macro_name:ident) => {
        $macro_name!{$($out_pairs)*}
    };
    // finished one loop of b, pop next a and reset b
    ($out_pairs:tt [$a_car:ty, $($a_cdr:ty,)*] [] $init_b:tt $macro_name:ident) => {
        cartesian_pairs_helper!{$out_pairs [$($a_cdr,)*] $init_b $init_b $macro_name}
    };
    // pop b through all of b with head of a
    ([$($out_pairs:tt)*] [$a_car:ty, $($a_cdr:ty,)*] [$b_car:ty, $($b_cdr:ty,)*] $init_b:tt $macro_name:ident) => {
        cartesian_pairs_helper!{[$($out_pairs)* ($a_car, $b_car),] [$a_car, $($a_cdr,)*] [$($b_cdr,)*] $init_b $macro_name}
    };
}

// Implement Relate for every combination of Geometry. Alternatively we could do something like
// `impl Relate<Into<GeometryCow>> for Into<GeometryCow> { }`
// but I don't know that we want to make GeometryCow public (yet?).
cartesian_pairs!(relate_impl, [Point<T, Z>, Line<T, Z>, LineString<T, Z>, Polygon<T, Z>, MultiPoint<T, Z>, MultiLineString<T, Z>, MultiPolygon<T, Z>, Rect<T, Z>, Triangle<T, Z>, GeometryCollection<T, Z>]);
relate_impl!(Geometry<T, Z>, Geometry<T, Z>);
