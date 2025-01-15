use std::{fmt, str::FromStr};

use chrono::FixedOffset;
use xml2_macro::UtilsDefaultSerde;

use crate::types::utils::parse_timezone;

#[derive(PartialEq, Debug, Clone, UtilsDefaultSerde)]
pub struct GYear {
    pub value: i32,
    pub timezone: Option<FixedOffset>,
}

impl GYear {
    pub fn new(year: i32, timezone: Option<FixedOffset>) -> Result<Self, String> {
        if year == 0 {
            return Err("bad gYear format: year 0 occurred".to_string());
        }
        Ok(GYear {
            value: year,
            timezone,
        })
    }
}

impl Default for GYear {
    fn default() -> GYear {
        Self {
            value: 1,
            timezone: None,
        }
    }
}

impl FromStr for GYear {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('-') {
            let mut gyear = parse_str_positive(s)?;
            gyear.value *= -1;
            return Ok(gyear);
        }
        parse_str_positive(s)
    }
}

fn parse_str_positive(s: &str) -> Result<GYear, String> {
    fn parse_value(s: &str) -> Result<i32, String> {
        if s.len() < 4 {
            return Err("bad gYear format: to short".to_string());
        }
        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err("bad gYear format".to_string());
        }
        s.parse::<i32>().map_err(|e| e.to_string())
    }

    if let Some(s) = s.strip_suffix('Z') {
        return GYear::new(parse_value(s)?, Some(FixedOffset::east_opt(0).unwrap()));
    }

    if s.contains('+') {
        if s.matches('+').count() > 1 {
            return Err("bad gYear format".to_string());
        }

        let idx: usize = s.match_indices('+').collect::<Vec<_>>()[0].0;
        let value_token = &s[..idx];
        let tz_token = &s[idx..];
        return GYear::new(parse_value(value_token)?, Some(parse_timezone(tz_token)?));
    }

    if s.contains('-') {
        if s.matches('-').count() > 1 {
            return Err("bad gYear format".to_string());
        }

        let idx: usize = s.match_indices('-').collect::<Vec<_>>()[0].0;
        let value_token = &s[..idx];
        let tz_token = &s[idx..];
        return GYear::new(parse_value(value_token)?, Some(parse_timezone(tz_token)?));
    }

    GYear::new(parse_value(s)?, None)
}

impl fmt::Display for GYear {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.value > 0 {
            match self.timezone {
                Some(tz) => write!(f, "{:04}{}", self.value, tz),
                None => write!(f, "{:04}", self.value),
            }
        } else {
            match self.timezone {
                Some(tz) => write!(f, "-{:04}{}", -self.value, tz),
                None => write!(f, "-{:04}", -self.value),
            }
        }
    }
}
