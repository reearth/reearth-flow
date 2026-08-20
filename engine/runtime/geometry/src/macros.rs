/// Stamp the mandatory empty `impl Trait for Type {}` blocks for operations a
/// leaf leaves to the trait default, so that default body fires through
/// `enum_dispatch`. The default is an `UnsupportedOperation` error for most
/// operations and a no-op for the ones that are vacuous on a leaf.
///
/// ```ignore
/// unsupported!(Csg: Reproject, WriteGltf);
/// ```
#[macro_export]
macro_rules! unsupported {
    ($ty:ty : $($tr:ident),+ $(,)?) => {
        $( impl $crate::ops::$tr for $ty {} )+
    };
}

/// Stamps [`Area`](crate::ops::Area) on a type that encloses no area — a point,
/// a curve, a point cloud — measuring `0.0` rather than refusing. Unlike
/// [`unsupported!`](crate::unsupported), which leaves the refusing default in
/// place, this records a real answer.
#[macro_export]
macro_rules! no_area {
    ($($ty:ty),+ $(,)?) => { $(
        impl $crate::ops::Area for $ty {
            fn projected_area(&self) -> Result<f64, $crate::ops::UnsupportedOperation> {
                Ok(0.0)
            }

            fn surface_area(&self) -> Result<f64, $crate::ops::UnsupportedOperation> {
                Ok(0.0)
            }
        }
    )+ };
}

#[macro_export]
macro_rules! point {
    ( $($tag:tt : $val:expr),* $(,)? ) => {
        $crate::point! ( $crate::coord! { $( $tag: $val , )* } )
    };
    ( $coord:expr $(,)? ) => {
        $crate::types::point::Point::from($coord)
    };
}

#[macro_export]
macro_rules! coord {
    (x: $x:expr, y: $y:expr $(,)* ) => {
        $crate::types::coordinate::Coordinate::new_($x, $y)
    };
    (x: $x:expr, y: $y:expr, z: $z:expr $(,)* ) => {
        $crate::types::coordinate::Coordinate::new__($x, $y, $z)
    };
}

#[macro_export]
macro_rules! line_string {
    () => { $crate::types::line_string::LineString::new(vec![]) };
    (
        $(( $($tag:tt : $val:expr),* $(,)? )),*
        $(,)?
    ) => {
        line_string![
            $(
                $crate::coord! { $( $tag: $val , )* },
            )*
        ]
    };
    (
        $($coord:expr),*
        $(,)?
    ) => {
        $crate::types::line_string::LineString::new(
            <[_]>::into_vec(
                ::std::boxed::Box::new(
                    [$($coord), *]
                )
            )
        )
    };
}

#[macro_export]
macro_rules! polygon {
    () => { $crate::types::polygon::Polygon::new(line_string![], vec![]) };
    (
        exterior: [
            $(( $($exterior_tag:tt : $exterior_val:expr),* $(,)? )),*
            $(,)?
        ],
        interiors: [
            $([
                $(( $($interior_tag:tt : $interior_val:expr),* $(,)? )),*
                $(,)?
            ]),*
            $(,)?
        ]
        $(,)?
    ) => {
        polygon!(
            exterior: [
                $(
                    $crate::coord! { $( $exterior_tag: $exterior_val , )* },
                )*
            ],
            interiors: [
                $([
                    $($crate::coord! { $( $interior_tag: $interior_val , )* }),*
                ]),*
            ],
        )
    };
    (
        exterior: [
            $($exterior_coord:expr),*
            $(,)?
        ],
        interiors: [
            $([
                $($interior_coord:expr),*
                $(,)?
            ]),*
            $(,)?
        ]
        $(,)?
    ) => {
        $crate::types::polygon::Polygon::new(
            $crate::line_string![
                $($exterior_coord), *
            ],
            <[_]>::into_vec(
                ::std::boxed::Box::new(
                    [
                        $(
                            $crate::line_string![$($interior_coord),*]
                        ), *
                    ]
                )
            )
        )
    };
    (
        $(( $($tag:tt : $val:expr),* $(,)? )),*
        $(,)?
    ) => {
        polygon![
            $($crate::coord! { $( $tag: $val , )* }),*
        ]
    };
    (
        $($coord:expr),*
        $(,)?
    ) => {
        $crate::types::polygon::Polygon::new(
            $crate::line_string![$($coord,)*],
            vec![],
        )
    };
}
