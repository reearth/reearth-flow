use std::{fmt, str::FromStr};

use num_bigint::{BigInt, ParseBigIntError, ToBigInt};
use xml2_macro::UtilsDefaultSerde;

// https://www.w3.org/TR/xmlschema-2/#integer
#[derive(Default, Clone, PartialEq, PartialOrd, Debug, UtilsDefaultSerde)]
pub struct Integer(pub BigInt);

impl Integer {
    pub fn from_bigint(bigint: BigInt) -> Self {
        Integer(bigint)
    }
}

impl ToBigInt for Integer {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(self.0.clone())
    }
}

impl FromStr for Integer {
    type Err = ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Integer(BigInt::from_str(s)?))
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_str_radix(10))
    }
}
