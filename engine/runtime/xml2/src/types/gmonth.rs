use std::{fmt, str::FromStr};

use chrono::FixedOffset;
use xml2_macro::UtilsDefaultSerde;

use crate::types::utils::parse_timezone;

#[derive(PartialEq, Debug, Clone, UtilsDefaultSerde)]
pub struct GMonth {
    pub value: i32,
    pub timezone: Option<FixedOffset>,
}

impl GMonth {
    pub fn new(month: i32, timezone: Option<FixedOffset>) -> Result<Self, String> {
        if !(1..=12).contains(&month) {
            return Err("GMonth value should lie between 1 and 12".to_string());
        }
        Ok(GMonth {
            value: month,
            timezone,
        })
    }
}

impl Default for GMonth {
    fn default() -> GMonth {
        Self {
            value: 1,
            timezone: None,
        }
    }
}

impl FromStr for GMonth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_value(s: &str) -> Result<i32, String> {
            if s.len() != 4 || &s[0..2] != "--" {
                return Err("bad gMonth format".to_string());
            }
            let token = &s[2..4];
            if !token.chars().all(|c| c.is_ascii_digit()) {
                return Err("bad gMonth format".to_string());
            }
            token.parse::<i32>().map_err(|e| e.to_string())
        }

        if let Some(s) = s.strip_suffix('Z') {
            return GMonth::new(parse_value(s)?, Some(FixedOffset::east_opt(0).unwrap()));
        }

        if s.contains('+') {
            if s.matches('+').count() > 1 {
                return Err("bad gMonth format".to_string());
            }

            let idx: usize = s.match_indices('+').collect::<Vec<_>>()[0].0;
            let value_token = &s[..idx];
            let tz_token = &s[idx..];
            return GMonth::new(parse_value(value_token)?, Some(parse_timezone(tz_token)?));
        }

        if s.matches('-').count() == 3 {
            let idx: usize = s.match_indices('-').collect::<Vec<_>>()[2].0;
            let value_token = &s[..idx];
            let tz_token = &s[idx..];
            return GMonth::new(parse_value(value_token)?, Some(parse_timezone(tz_token)?));
        }

        GMonth::new(parse_value(s)?, None)
    }
}

impl fmt::Display for GMonth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.timezone {
            Some(tz) => write!(f, "--{:02}{}", self.value, tz),
            None => write!(f, "--{:02}", self.value),
        }
    }
}
