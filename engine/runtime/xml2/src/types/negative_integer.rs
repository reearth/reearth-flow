use std::{fmt, str::FromStr};

use num_bigint::{BigInt, ToBigInt};
use xml2_macro::UtilsDefaultSerde;

// https://www.w3.org/TR/xmlschema-2/#negativeInteger
#[derive(Default, Clone, PartialEq, PartialOrd, Debug, UtilsDefaultSerde)]
pub struct NegativeInteger(pub BigInt);

impl NegativeInteger {
    pub fn from_bigint(bigint: BigInt) -> Self {
        NegativeInteger(bigint)
    }
}

impl ToBigInt for NegativeInteger {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(self.0.clone())
    }
}

impl FromStr for NegativeInteger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = BigInt::from_str(s).map_err(|e| e.to_string())?;
        if value >= 0.to_bigint().unwrap() {
            Err("Bad value for NegativeInteger".to_string())
        } else {
            Ok(NegativeInteger(value))
        }
    }
}

impl fmt::Display for NegativeInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_str_radix(10))
    }
}
