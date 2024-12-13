use std::{fmt, str::FromStr};

use chrono::FixedOffset;
use xml2_macro::UtilsDefaultSerde;

use crate::types::utils::parse_timezone;

#[derive(PartialEq, Debug, Clone, UtilsDefaultSerde)]
pub struct GDay {
    pub value: i32,
    pub timezone: Option<FixedOffset>,
}

impl GDay {
    pub fn new(day: i32, timezone: Option<FixedOffset>) -> Result<Self, String> {
        if !(1..=31).contains(&day) {
            return Err("gDay value should lie between 1 and 31".to_string());
        }
        Ok(GDay {
            value: day,
            timezone,
        })
    }
}

impl Default for GDay {
    fn default() -> GDay {
        Self {
            value: 1,
            timezone: None,
        }
    }
}

impl FromStr for GDay {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_value(s: &str) -> Result<i32, String> {
            if s.len() != 5 || &s[0..3] != "---" {
                return Err("bad gDay format".to_string());
            }
            let token = &s[3..5];
            if !token.chars().all(|c| c.is_ascii_digit()) {
                return Err("bad gDay format".to_string());
            }
            token.parse::<i32>().map_err(|e| e.to_string())
        }

        if let Some(s) = s.strip_suffix('Z') {
            return GDay::new(parse_value(s)?, Some(FixedOffset::east_opt(0).unwrap()));
        }

        if s.contains('+') {
            if s.matches('+').count() > 1 {
                return Err("bad gDay format".to_string());
            }

            let idx: usize = s.match_indices('+').collect::<Vec<_>>()[0].0;
            let value_token = &s[..idx];
            let tz_token = &s[idx..];
            return GDay::new(parse_value(value_token)?, Some(parse_timezone(tz_token)?));
        }

        if s.matches('-').count() == 4 {
            let idx: usize = s.match_indices('-').collect::<Vec<_>>()[3].0;
            let value_token = &s[..idx];
            let tz_token = &s[idx..];
            return GDay::new(parse_value(value_token)?, Some(parse_timezone(tz_token)?));
        }

        GDay::new(parse_value(s)?, None)
    }
}

impl fmt::Display for GDay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.timezone {
            Some(tz) => write!(f, "---{:02}{}", self.value, tz),
            None => write!(f, "---{:02}", self.value),
        }
    }
}
