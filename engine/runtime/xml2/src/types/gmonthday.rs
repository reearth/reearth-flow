use std::{fmt, str::FromStr};

use chrono::FixedOffset;
use xml2_macro::UtilsDefaultSerde;

use crate::types::{gday::GDay, gmonth::GMonth, utils::parse_timezone};

#[derive(PartialEq, Debug, Clone, UtilsDefaultSerde)]
pub struct GMonthDay {
    pub month: i32,
    pub day: i32,
    pub timezone: Option<FixedOffset>,
}

impl GMonthDay {
    pub fn new(month: i32, day: i32, timezone: Option<FixedOffset>) -> Result<Self, String> {
        if !(1..=12).contains(&month) {
            return Err("Month value within GMonthDay should lie between 1 and 12".to_string());
        }

        if !(1..=31).contains(&day) {
            return Err("Day value within GMonthDay should lie between 1 and 31".to_string());
        }

        const MONTH_MAX_LEN: [i32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if day > MONTH_MAX_LEN[month as usize - 1] {
            return Err("Day value within GMonthDay is to big for specified month".to_string());
        }

        Ok(GMonthDay {
            month,
            day,
            timezone,
        })
    }

    pub fn gmonth(self) -> GMonth {
        GMonth {
            value: self.month,
            timezone: self.timezone,
        }
    }

    pub fn gday(self) -> GDay {
        GDay {
            value: self.day,
            timezone: self.timezone,
        }
    }
}

impl Default for GMonthDay {
    fn default() -> GMonthDay {
        Self {
            month: 1,
            day: 1,
            timezone: None,
        }
    }
}

impl FromStr for GMonthDay {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_value(s: &str) -> Result<(i32, i32), String> {
            if s.len() != 7 || &s[0..2] != "--" || &s[4..5] != "-" {
                return Err("bad gMonthDay format".to_string());
            }

            let month_token = &s[2..4];
            if !month_token.chars().all(|c| c.is_ascii_digit()) {
                return Err("bad month format within gMonthDay".to_string());
            }
            let month = month_token.parse::<i32>().map_err(|e| e.to_string())?;

            let day_token = &s[5..7];
            if !day_token.chars().all(|c| c.is_ascii_digit()) {
                return Err("bad day format within gMonthDay".to_string());
            }
            let day = day_token.parse::<i32>().map_err(|e| e.to_string())?;

            Ok((month, day))
        }

        if let Some(s) = s.strip_suffix('Z') {
            let (month, day) = parse_value(s)?;
            return GMonthDay::new(month, day, Some(FixedOffset::east_opt(0).unwrap()));
        }

        if s.contains('+') {
            if s.matches('+').count() > 1 {
                return Err("bad gMonthDay format".to_string());
            }

            let idx: usize = s.match_indices('+').collect::<Vec<_>>()[0].0;
            let value_token = &s[..idx];
            let tz_token = &s[idx..];
            let (month, day) = parse_value(value_token)?;
            return GMonthDay::new(month, day, Some(parse_timezone(tz_token)?));
        }

        if s.matches('-').count() == 4 {
            let idx: usize = s.match_indices('-').collect::<Vec<_>>()[3].0;
            let value_token = &s[..idx];
            let tz_token = &s[idx..];
            let (month, day) = parse_value(value_token)?;
            return GMonthDay::new(month, day, Some(parse_timezone(tz_token)?));
        }

        let (month, day) = parse_value(s)?;
        GMonthDay::new(month, day, None)
    }
}

impl fmt::Display for GMonthDay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.timezone {
            Some(tz) => write!(f, "--{:02}-{:02}{}", self.month, self.day, tz),
            None => write!(f, "--{:02}-{:02}", self.month, self.day),
        }
    }
}
