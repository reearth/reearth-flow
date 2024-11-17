use std::{fmt, str::FromStr};

use chrono::{format::strftime::StrftimeItems, FixedOffset, NaiveDate};
use xml2_macro::UtilsDefaultSerde;

use crate::types::utils::parse_timezone;

#[derive(PartialEq, Debug, Clone, UtilsDefaultSerde)]
pub struct Date {
    pub value: NaiveDate,
    pub timezone: Option<FixedOffset>,
}

impl Date {
    pub fn from_chrono_naive_date(date: NaiveDate) -> Self {
        Date {
            value: date,
            timezone: None,
        }
    }

    pub fn to_chrono_naive_date(&self) -> NaiveDate {
        self.value
    }
}

impl Default for Date {
    fn default() -> Date {
        Self {
            value: NaiveDate::from_ymd_opt(1, 1, 1).unwrap(),
            timezone: None,
        }
    }
}

impl FromStr for Date {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_naive_date(s: &str) -> Result<NaiveDate, String> {
            NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| e.to_string())
        }

        if let Some(s) = s.strip_suffix('Z') {
            return Ok(Date {
                value: parse_naive_date(s)?,
                timezone: Some(FixedOffset::east_opt(0).unwrap()),
            });
        }

        if s.contains('+') {
            if s.matches('+').count() > 1 {
                return Err("bad date format".to_string());
            }

            let idx: usize = s.match_indices('+').collect::<Vec<_>>()[0].0;
            let date_token = &s[..idx];
            let tz_token = &s[idx..];
            return Ok(Date {
                value: parse_naive_date(date_token)?,
                timezone: Some(parse_timezone(tz_token)?),
            });
        }

        if s.matches('-').count() == 3 {
            let idx: usize = s.match_indices('-').collect::<Vec<_>>()[2].0;
            let date_token = &s[..idx];
            let tz_token = &s[idx..];
            return Ok(Date {
                value: parse_naive_date(date_token)?,
                timezone: Some(parse_timezone(tz_token)?),
            });
        }

        Ok(Date {
            value: parse_naive_date(s)?,
            timezone: None,
        })
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmt = StrftimeItems::new("%Y-%m-%d");
        match self.timezone {
            Some(tz) => write!(f, "{}{}", self.value.format_with_items(fmt.clone()), tz),
            None => write!(f, "{}", self.value.format_with_items(fmt.clone())),
        }
    }
}
