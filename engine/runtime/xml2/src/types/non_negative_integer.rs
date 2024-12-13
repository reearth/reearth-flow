use std::{fmt, str::FromStr};

use num_bigint::{BigUint, ToBigUint};
use xml2_macro::UtilsDefaultSerde;

// https://www.w3.org/TR/xmlschema-2/#nonNegativeInteger
#[derive(Default, Clone, PartialEq, PartialOrd, Debug, UtilsDefaultSerde)]
pub struct NonNegativeInteger(pub BigUint);

impl NonNegativeInteger {
    pub fn from_biguint(bigint: BigUint) -> Self {
        NonNegativeInteger(bigint)
    }
}

impl ToBigUint for NonNegativeInteger {
    fn to_biguint(&self) -> Option<BigUint> {
        Some(self.0.clone())
    }
}

impl FromStr for NonNegativeInteger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = BigUint::from_str(s).map_err(|e| e.to_string())?;
        if value < 0.to_biguint().unwrap() {
            Err("Bad value for NonNegativeInteger".to_string())
        } else {
            Ok(NonNegativeInteger(value))
        }
    }
}

impl fmt::Display for NonNegativeInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_str_radix(10))
    }
}
