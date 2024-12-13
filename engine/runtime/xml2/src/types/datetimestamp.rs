use std::{fmt, str::FromStr};

use chrono::{format::ParseError, DateTime as CDateTime, FixedOffset};
use xml2_macro::UtilsDefaultSerde;

use crate::types::datetime::DateTime;

#[derive(Default, Clone, PartialEq, PartialOrd, Debug, UtilsDefaultSerde)]
pub struct DateTimeStamp {
    pub value: DateTime,
}

impl DateTimeStamp {
    pub fn from_chrono_datetime(datetime: CDateTime<FixedOffset>) -> Self {
        DateTimeStamp {
            value: DateTime::from_chrono_datetime(datetime),
        }
    }

    pub fn to_chrono_datetime(&self) -> CDateTime<FixedOffset> {
        self.value.to_chrono_datetime()
    }
}

impl FromStr for DateTimeStamp {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match CDateTime::parse_from_rfc3339(s) {
            Ok(cdt) => Ok(DateTimeStamp::from_chrono_datetime(cdt)),
            Err(err) => Err(err),
        }
    }
}

impl fmt::Display for DateTimeStamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
