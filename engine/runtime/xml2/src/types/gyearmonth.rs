use std::{fmt, str::FromStr};

use chrono::FixedOffset;
use xml2_macro::UtilsDefaultSerde;

use crate::types::{gmonth::GMonth, gyear::GYear, utils::parse_timezone};

#[derive(PartialEq, Debug, Clone, UtilsDefaultSerde)]
pub struct GYearMonth {
    pub year: i32,
    pub month: i32,
    pub timezone: Option<FixedOffset>,
}

impl GYearMonth {
    pub fn new(year: i32, month: i32, timezone: Option<FixedOffset>) -> Result<Self, String> {
        if year == 0 {
            return Err("bad gYear format: year 0 occurred".to_string());
        }

        if !(1..=12).contains(&month) {
            return Err("Month value within GYearMonth should lie between 1 and 12".to_string());
        }

        Ok(GYearMonth {
            year,
            month,
            timezone,
        })
    }

    pub fn gyear(self) -> GYear {
        GYear {
            value: self.year,
            timezone: self.timezone,
        }
    }

    pub fn gmonth(self) -> GMonth {
        GMonth {
            value: self.month,
            timezone: self.timezone,
        }
    }
}

impl Default for GYearMonth {
    fn default() -> GYearMonth {
        Self {
            year: 1,
            month: 1,
            timezone: None,
        }
    }
}

impl FromStr for GYearMonth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('-') {
            let mut gyearmonth = parse_str_positive(s)?;
            gyearmonth.year *= -1;
            return Ok(gyearmonth);
        }
        parse_str_positive(s)
    }
}

fn parse_str_positive(s: &str) -> Result<GYearMonth, String> {
    fn parse_value(s: &str) -> Result<(i32, i32), String> {
        if s.matches('-').count() != 1 {
            return Err("bad gYearMonth format".to_string());
        }

        let idx: usize = s.match_indices('-').collect::<Vec<_>>()[0].0;
        let year_token = &s[..idx];
        let month_token = &s[idx + 1..];
        if year_token.len() < 4 || month_token.len() != 2 {
            return Err("bad gYearMonth format".to_string());
        }

        if !year_token.chars().all(|c| c.is_ascii_digit()) {
            return Err("bad year format within gYearMonth".to_string());
        }
        let year = year_token.parse::<i32>().map_err(|e| e.to_string())?;

        if !month_token.chars().all(|c| c.is_ascii_digit()) {
            return Err("bad month format within gYearMonth".to_string());
        }
        let month = month_token.parse::<i32>().map_err(|e| e.to_string())?;

        Ok((year, month))
    }

    if let Some(s) = s.strip_suffix('Z') {
        let (year, month) = parse_value(s)?;
        return GYearMonth::new(year, month, Some(FixedOffset::east_opt(0).unwrap()));
    }

    if s.contains('+') {
        if s.matches('+').count() > 1 {
            return Err("bad gMonthDay format".to_string());
        }

        let idx: usize = s.match_indices('+').collect::<Vec<_>>()[0].0;
        let value_token = &s[..idx];
        let tz_token = &s[idx..];
        let (year, month) = parse_value(value_token)?;
        return GYearMonth::new(year, month, Some(parse_timezone(tz_token)?));
    }

    if s.matches('-').count() == 2 {
        let idx: usize = s.match_indices('-').collect::<Vec<_>>()[1].0;
        let value_token = &s[..idx];
        let tz_token = &s[idx..];
        let (year, month) = parse_value(value_token)?;
        return GYearMonth::new(year, month, Some(parse_timezone(tz_token)?));
    }

    let (year, month) = parse_value(s)?;
    GYearMonth::new(year, month, None)
}

impl fmt::Display for GYearMonth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.year > 0 {
            match self.timezone {
                Some(tz) => write!(f, "{:04}-{:02}{}", self.year, self.month, tz),
                None => write!(f, "{:04}-{:02}", self.year, self.month),
            }
        } else {
            match self.timezone {
                Some(tz) => write!(f, "-{:04}-{:02}{}", -self.year, self.month, tz),
                None => write!(f, "-{:04}-{:02}", -self.year, self.month),
            }
        }
    }
}
