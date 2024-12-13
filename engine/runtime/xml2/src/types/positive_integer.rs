use std::{fmt, str::FromStr};

use num_bigint::{BigUint, ToBigUint};
use xml2_macro::UtilsDefaultSerde;

// https://www.w3.org/TR/xmlschema-2/#positiveInteger
#[derive(Default, Clone, PartialEq, PartialOrd, Debug, UtilsDefaultSerde)]
pub struct PositiveInteger(pub BigUint);

impl PositiveInteger {
    pub fn from_biguint(bigint: BigUint) -> Self {
        PositiveInteger(bigint)
    }
}

impl ToBigUint for PositiveInteger {
    fn to_biguint(&self) -> Option<BigUint> {
        Some(self.0.clone())
    }
}

impl FromStr for PositiveInteger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = BigUint::from_str(s).map_err(|e| e.to_string())?;
        if value <= 0.to_biguint().unwrap() {
            Err("Bad value for PositiveInteger".to_string())
        } else {
            Ok(PositiveInteger(value))
        }
    }
}

impl fmt::Display for PositiveInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_str_radix(10))
    }
}
