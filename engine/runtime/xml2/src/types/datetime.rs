use std::{fmt, str::FromStr};

use chrono::{format::ParseError, DateTime as CDateTime, FixedOffset};
use xml2_macro::UtilsDefaultSerde;

#[derive(PartialEq, PartialOrd, Debug, Clone, UtilsDefaultSerde)]
pub struct DateTime {
    pub value: CDateTime<FixedOffset>,
}

impl DateTime {
    pub fn from_chrono_datetime(datetime: CDateTime<FixedOffset>) -> Self {
        DateTime { value: datetime }
    }

    pub fn to_chrono_datetime(&self) -> CDateTime<FixedOffset> {
        self.value
    }
}

impl Default for DateTime {
    fn default() -> DateTime {
        Self {
            value: CDateTime::parse_from_rfc3339("0001-01-01T00:00:00Z").unwrap(),
        }
    }
}

impl FromStr for DateTime {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tz_provided = s.ends_with('Z') || s.contains('+') || s.matches('-').count() == 3;
        let s_with_timezone = if tz_provided {
            s.to_string()
        } else {
            format!("{}Z", s)
        };
        match CDateTime::parse_from_rfc3339(&s_with_timezone) {
            Ok(cdt) => Ok(DateTime { value: cdt }),
            Err(err) => Err(err),
        }
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value.to_rfc3339())
    }
}
