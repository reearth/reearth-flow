pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

macro_rules! impl_contains_from_relate {
    ($for:ty,  [$($target:ty),*]) => {
        $(
            impl<T, Z> Contains<$target> for $for
            where
                T: $crate::algorithm::GeoFloat,
                Z: $crate::algorithm::GeoFloat,
            {
                fn contains(&self, target: &$target) -> bool {
                    use $crate::algorithm::Relate;
                    self.relate(target).is_contains()
                }
            }
        )*
    };
}
pub mod line;
pub mod line_string;
pub mod point;
pub mod polygon;
pub mod rect;
pub mod triangle;
