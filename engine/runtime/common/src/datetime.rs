use chrono::{DateTime, Utc};

pub fn try_from(s: &str) -> crate::Result<chrono::DateTime<Utc>> {
    if let Ok(v) = chrono::DateTime::parse_from_rfc3339(s).map_err(crate::Error::datetime) {
        Ok(v.into())
    } else if let Ok(v) = chrono::DateTime::parse_from_str(s, "%Y/%m/%d %H:%M:%S") {
        Ok(v.into())
    } else if let Ok(v) = chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        Ok(v.into())
    } else if let Ok(v) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        if let Some(v) = v.and_hms_opt(0, 0, 0) {
            Ok(DateTime::<Utc>::from_naive_utc_and_offset(v, Utc))
        } else {
            Err(crate::Error::datetime(format!("Invalid datetime: {s}")))
        }
    } else {
        Err(crate::Error::datetime(format!("Invalid datetime: {s}")))
    }
}

/// Parse datetime from Unix timestamp in seconds
pub fn try_from_unix_s(ts: i64) -> crate::Result<chrono::DateTime<Utc>> {
    chrono::DateTime::from_timestamp(ts, 0)
        .ok_or_else(|| crate::Error::datetime(format!("Invalid Unix timestamp (seconds): {ts}")))
}

/// Parse datetime from Unix timestamp in milliseconds
pub fn try_from_unix_ms(ts: i64) -> crate::Result<chrono::DateTime<Utc>> {
    let secs = ts.div_euclid(1000);
    let nanos = (ts.rem_euclid(1000) * 1_000_000) as u32;
    chrono::DateTime::from_timestamp(secs, nanos).ok_or_else(|| {
        crate::Error::datetime(format!("Invalid Unix timestamp (milliseconds): {ts}"))
    })
}

/// Format a DateTime<Utc> according to the specified format string using chrono
pub fn format_with(dt: &chrono::DateTime<Utc>, format: &str) -> String {
    dt.format(format).to_string()
}

/// Output datetime as RFC3339 string
pub fn to_rfc3339(dt: &chrono::DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Output datetime as Unix timestamp in seconds
pub fn to_unix_s(dt: &chrono::DateTime<Utc>) -> i64 {
    dt.timestamp()
}

/// Output datetime as Unix timestamp in milliseconds
pub fn to_unix_ms(dt: &chrono::DateTime<Utc>) -> i64 {
    dt.timestamp_millis()
}

/// Output datetime as date only string (YYYY-MM-DD)
pub fn to_date_string(dt: &chrono::DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
}
