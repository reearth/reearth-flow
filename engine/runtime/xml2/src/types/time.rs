use std::{fmt, str::FromStr};

use chrono::{format::strftime::StrftimeItems, FixedOffset, NaiveTime};
use xml2_macro::UtilsDefaultSerde;

use crate::types::utils::parse_timezone;

#[derive(PartialEq, Debug, Clone, UtilsDefaultSerde)]
pub struct Time {
    pub value: NaiveTime,
    pub timezone: Option<FixedOffset>,
}

impl Time {
    pub fn from_chrono_naive_time(time: NaiveTime) -> Self {
        Time {
            value: time,
            timezone: None,
        }
    }

    pub fn to_chrono_naive_time(&self) -> NaiveTime {
        self.value
    }
}

impl Default for Time {
    fn default() -> Time {
        Self {
            value: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            timezone: None,
        }
    }
}

impl FromStr for Time {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_naive_time(s: &str) -> Result<NaiveTime, String> {
            NaiveTime::parse_from_str(s, "%H:%M:%S").map_err(|e| e.to_string())
        }

        if let Some(s) = s.strip_suffix('Z') {
            return Ok(Time {
                value: parse_naive_time(s)?,
                timezone: Some(FixedOffset::east_opt(0).unwrap()),
            });
        }

        if s.contains('+') {
            if s.matches('+').count() > 1 {
                return Err("bad date format".to_string());
            }

            let idx: usize = s.match_indices('+').collect::<Vec<_>>()[0].0;
            let time_token = &s[..idx];
            let tz_token = &s[idx..];
            return Ok(Time {
                value: parse_naive_time(time_token)?,
                timezone: Some(parse_timezone(tz_token)?),
            });
        }

        if s.contains('-') {
            if s.matches('-').count() > 1 {
                return Err("bad date format".to_string());
            }

            let idx: usize = s.match_indices('-').collect::<Vec<_>>()[0].0;
            let time_token = &s[..idx];
            let tz_token = &s[idx..];
            return Ok(Time {
                value: parse_naive_time(time_token)?,
                timezone: Some(parse_timezone(tz_token)?),
            });
        }

        Ok(Time {
            value: parse_naive_time(s)?,
            timezone: None,
        })
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmt = StrftimeItems::new("%H:%M:%S");
        match self.timezone {
            Some(tz) => write!(f, "{}{}", self.value.format_with_items(fmt.clone()), tz),
            None => write!(f, "{}", self.value.format_with_items(fmt.clone())),
        }
    }
}
