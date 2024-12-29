use std::{fmt, str::FromStr};

use bigdecimal::{BigDecimal, ParseBigDecimalError};
use xml2_macro::UtilsDefaultSerde;

#[derive(Default, Clone, PartialEq, PartialOrd, Debug, UtilsDefaultSerde)]
pub struct Decimal(pub BigDecimal);

impl Decimal {
    pub fn from_bigdecimal(bigdecimal: BigDecimal) -> Self {
        Decimal(bigdecimal)
    }

    pub fn to_bigdecimal(&self) -> BigDecimal {
        self.0.clone()
    }
}

impl FromStr for Decimal {
    type Err = ParseBigDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Decimal(BigDecimal::from_str(s)?))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
