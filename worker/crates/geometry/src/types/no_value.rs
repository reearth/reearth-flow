use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use num_traits::{Num, NumCast, One, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, PartialOrd, Clone, Copy, Debug, Hash, Default)]
pub struct NoValue;

impl Add for NoValue {
    type Output = Self;

    #[inline]
    fn add(self, _: Self) -> Self::Output {
        NoValue
    }
}

impl<T> Div<T> for NoValue {
    type Output = Self;

    #[inline]
    fn div(self, _: T) -> Self::Output {
        NoValue
    }
}

impl<T> Mul<T> for NoValue {
    type Output = Self;

    #[inline]
    fn mul(self, _: T) -> Self::Output {
        NoValue
    }
}

impl Neg for NoValue {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        NoValue
    }
}

impl<T> Rem<T> for NoValue {
    type Output = Self;

    #[inline]
    fn rem(self, _: T) -> Self::Output {
        NoValue
    }
}

impl Sub for NoValue {
    type Output = Self;

    #[inline]
    fn sub(self, _: Self) -> Self::Output {
        NoValue
    }
}

/// This hack allows mathematical operations that result in noop due to above ops
impl Zero for NoValue {
    #[inline]
    fn zero() -> Self {
        NoValue
    }

    #[inline]
    fn is_zero(&self) -> bool {
        true
    }
}

/// These hacks allows mathematical operations that result in noop due to above ops
impl One for NoValue {
    #[inline]
    fn one() -> Self {
        NoValue
    }
}

impl ToPrimitive for NoValue {
    fn to_i64(&self) -> Option<i64> {
        None
    }

    fn to_u64(&self) -> Option<u64> {
        None
    }
}

impl NumCast for NoValue {
    fn from<T: ToPrimitive>(_: T) -> Option<Self> {
        None
    }
}

impl Num for NoValue {
    type FromStrRadixErr = ();

    fn from_str_radix(_str: &str, _radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Err(())
    }
}
