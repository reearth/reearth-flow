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
            Err(crate::Error::datetime(format!("Invalid datetime: {}", s)))
        }
    } else {
        Err(crate::Error::datetime(format!("Invalid datetime: {}", s)))
    }
}
