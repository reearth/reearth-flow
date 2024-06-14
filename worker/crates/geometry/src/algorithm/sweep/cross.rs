use std::{fmt::Debug, rc::Rc, sync::Arc};

use crate::{algorithm::GeoFloat, types::line::Line};

use super::LineOrPoint;

pub trait Cross: Sized + Debug {
    /// Scalar used the coordinates.
    type ScalarXY: GeoFloat;
    type ScalarZ: GeoFloat;

    fn line(&self) -> LineOrPoint<Self::ScalarXY, Self::ScalarZ>;
}

impl<'a, T: Cross> Cross for &'a T {
    type ScalarXY = T::ScalarXY;
    type ScalarZ = T::ScalarZ;

    fn line(&self) -> LineOrPoint<Self::ScalarXY, Self::ScalarZ> {
        T::line(*self)
    }
}

impl<T: GeoFloat, Z: GeoFloat> Cross for LineOrPoint<T, Z> {
    type ScalarXY = T;
    type ScalarZ = Z;

    fn line(&self) -> LineOrPoint<Self::ScalarXY, Self::ScalarZ> {
        *self
    }
}

impl<T: GeoFloat, Z: GeoFloat> Cross for Line<T, Z> {
    type ScalarXY = T;
    type ScalarZ = Z;

    fn line(&self) -> LineOrPoint<Self::ScalarXY, Self::ScalarZ> {
        (*self).into()
    }
}

macro_rules! blanket_impl_smart_pointer {
    ($ty:ty) => {
        impl<T: Cross> Cross for $ty {
            type ScalarXY = T::ScalarXY;
            type ScalarZ = T::ScalarZ;

            fn line(&self) -> LineOrPoint<Self::ScalarXY, Self::ScalarZ> {
                T::line(self)
            }
        }
    };
}
blanket_impl_smart_pointer!(Box<T>);
blanket_impl_smart_pointer!(Rc<T>);
blanket_impl_smart_pointer!(Arc<T>);
