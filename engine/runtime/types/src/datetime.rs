use chrono::{
    offset::LocalResult, DateTime as ChronoDateTime, Datelike, FixedOffset, NaiveDate,
    SecondsFormat, TimeZone, Utc,
};
use serde::{de::Visitor, Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// DateTime enum representing different datetime value types.
/// Follows the "correct typing" principle:
/// - NaiveDate: for date-only values (no timezone)
/// - Utc: for absolute timestamps or when timezone is unknown
/// - FixedOffset: for datetime strings that include timezone info
///
/// Note: Uses custom serde to serialize/deserialize as a plain RFC3339 string
/// for backward compatibility with existing JSON data.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum DateTime {
    NaiveDate(NaiveDate),
    Utc(ChronoDateTime<Utc>),
    FixedOffset(ChronoDateTime<FixedOffset>),
}

impl Default for DateTime {
    fn default() -> Self {
        Self::Utc(Utc::now())
    }
}

impl DateTime {
    /// Convert to UTC datetime
    pub fn to_utc(&self) -> ChronoDateTime<Utc> {
        match self {
            DateTime::NaiveDate(d) => {
                ChronoDateTime::from_naive_utc_and_offset(d.and_hms_opt(0, 0, 0).unwrap(), Utc)
            }
            DateTime::Utc(dt) => *dt,
            DateTime::FixedOffset(dt) => dt.with_timezone(&Utc),
        }
    }

    /// Convert to RFC3339 string
    pub fn to_rfc3339(&self) -> String {
        match self {
            DateTime::FixedOffset(dt) => dt.to_rfc3339(),
            _ => self.to_utc().to_rfc3339(),
        }
    }

    /// Convert to Unix timestamp in seconds
    pub fn timestamp(&self) -> i64 {
        self.to_utc().timestamp()
    }

    /// Convert to Unix timestamp in milliseconds
    pub fn timestamp_millis(&self) -> i64 {
        self.to_utc().timestamp_millis()
    }

    /// Format with custom format string
    pub fn format(&self, fmt: &str) -> String {
        match self {
            DateTime::NaiveDate(d) => d.format(fmt).to_string(),
            DateTime::Utc(dt) => dt.format(fmt).to_string(),
            DateTime::FixedOffset(dt) => dt.format(fmt).to_string(),
        }
    }

    /// Get year
    pub fn year(&self) -> i32 {
        match self {
            DateTime::NaiveDate(d) => d.year(),
            DateTime::Utc(dt) => dt.year(),
            DateTime::FixedOffset(dt) => dt.year(),
        }
    }

    /// Get month
    pub fn month(&self) -> u32 {
        match self {
            DateTime::NaiveDate(d) => d.month(),
            DateTime::Utc(dt) => dt.month(),
            DateTime::FixedOffset(dt) => dt.month(),
        }
    }

    /// Get day
    pub fn day(&self) -> u32 {
        match self {
            DateTime::NaiveDate(d) => d.day(),
            DateTime::Utc(dt) => dt.day(),
            DateTime::FixedOffset(dt) => dt.day(),
        }
    }

    /// Convert the DateTime to a raw String
    pub fn to_raw(&self) -> String {
        self.to_utc().to_rfc3339_opts(SecondsFormat::AutoSi, true)
    }
}

/// Custom Serialize implementation to maintain backward compatibility.
/// Serializes as a plain RFC3339 string.
impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_rfc3339())
    }
}

/// Custom Deserialize implementation to maintain backward compatibility.
/// Parses from a string into the appropriate variant.
impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DateTimeVisitor;

        impl<'de> Visitor<'de> for DateTimeVisitor {
            type Value = DateTime;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a datetime string in RFC3339 format")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // Try RFC3339 with timezone first
                if let Ok(dt) = ChronoDateTime::parse_from_rfc3339(v) {
                    return Ok(DateTime::FixedOffset(dt));
                }
                // Try other formats via common datetime
                if let Ok(dt) = reearth_flow_common::datetime::try_from(v) {
                    return Ok(DateTime::Utc(dt));
                }
                // Try date-only format
                if let Ok(d) = NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                    return Ok(DateTime::NaiveDate(d));
                }
                Err(serde::de::Error::custom(format!(
                    "invalid datetime format: {}",
                    v
                )))
            }
        }

        deserializer.deserialize_str(DateTimeVisitor)
    }
}

impl TryFrom<i64> for DateTime {
    type Error = ();
    fn try_from(secs: i64) -> Result<Self, Self::Error> {
        if let Some(timestamp) = ChronoDateTime::from_timestamp(secs, 0) {
            Ok(Self::Utc(timestamp.with_timezone(&Utc)))
        } else {
            Err(())
        }
    }
}

impl From<ChronoDateTime<Utc>> for DateTime {
    fn from(v: ChronoDateTime<Utc>) -> Self {
        Self::Utc(v)
    }
}

impl From<ChronoDateTime<FixedOffset>> for DateTime {
    fn from(v: ChronoDateTime<FixedOffset>) -> Self {
        Self::FixedOffset(v)
    }
}

impl From<NaiveDate> for DateTime {
    fn from(v: NaiveDate) -> Self {
        Self::NaiveDate(v)
    }
}

impl From<DateTime> for ChronoDateTime<Utc> {
    fn from(x: DateTime) -> Self {
        x.to_utc()
    }
}

impl FromStr for DateTime {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<String> for DateTime {
    type Error = ();
    fn try_from(v: String) -> Result<Self, Self::Error> {
        Self::try_from(v.as_str())
    }
}

impl TryFrom<&str> for DateTime {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // Try RFC3339 with timezone first
        if let Ok(dt) = ChronoDateTime::parse_from_rfc3339(s) {
            return Ok(Self::FixedOffset(dt));
        }
        // Try other formats via common datetime
        if let Ok(dt) = reearth_flow_common::datetime::try_from(s) {
            return Ok(Self::Utc(dt));
        }
        // Try date-only format
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Ok(Self::NaiveDate(d));
        }
        Err(())
    }
}

impl TryFrom<(i64, u32)> for DateTime {
    type Error = ();
    fn try_from(v: (i64, u32)) -> Result<Self, Self::Error> {
        match Utc.timestamp_opt(v.0, v.1) {
            LocalResult::Single(v) => Ok(Self::Utc(v)),
            _ => Err(()),
        }
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.to_raw(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_as_string() {
        // Utc variant
        let dt = DateTime::Utc(Utc::now());
        let json = serde_json::to_string(&dt).unwrap();
        // Should be a plain string, not a tagged enum
        assert!(json.starts_with('"') && json.ends_with('"'));
        assert!(!json.contains("Utc"));

        // FixedOffset variant
        let dt = DateTime::FixedOffset(
            FixedOffset::east_opt(5 * 3600 + 30 * 60)
                .unwrap()
                .with_ymd_and_hms(2021, 1, 1, 12, 0, 0)
                .unwrap(),
        );
        let json = serde_json::to_string(&dt).unwrap();
        assert!(json.starts_with('"') && json.ends_with('"'));
        assert!(json.contains("+05:30"));

        // NaiveDate variant
        let dt = DateTime::NaiveDate(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        let json = serde_json::to_string(&dt).unwrap();
        assert!(json.starts_with('"') && json.ends_with('"'));
    }

    #[test]
    fn test_deserialize_from_rfc3339_utc() {
        let json = "\"2021-01-01T00:00:00Z\"";
        let dt: DateTime = serde_json::from_str(json).unwrap();
        match dt {
            DateTime::Utc(_) => {}
            DateTime::FixedOffset(_) => {}
            _ => panic!("Expected Utc or FixedOffset variant for UTC timestamp"),
        }
        assert_eq!(dt.timestamp(), 1609459200);
    }

    #[test]
    fn test_deserialize_from_rfc3339_with_timezone() {
        let json = "\"2021-01-01T12:00:00+05:30\"";
        let dt: DateTime = serde_json::from_str(json).unwrap();
        match dt {
            DateTime::FixedOffset(dt) => {
                assert_eq!(dt.offset().local_minus_utc(), 5 * 3600 + 30 * 60);
            }
            _ => panic!("Expected FixedOffset variant"),
        }
    }

    #[test]
    fn test_deserialize_from_date_only() {
        let json = "\"2021-01-01\"";
        let dt: DateTime = serde_json::from_str(json).unwrap();
        // Date-only string may be parsed as Utc (with time 00:00:00) or NaiveDate
        // depending on which parser matches first
        match dt {
            DateTime::NaiveDate(d) => {
                assert_eq!(d.year(), 2021);
                assert_eq!(d.month(), 1);
                assert_eq!(d.day(), 1);
            }
            DateTime::Utc(dt) => {
                assert_eq!(dt.year(), 2021);
                assert_eq!(dt.month(), 1);
                assert_eq!(dt.day(), 1);
            }
            _ => panic!("Expected NaiveDate or Utc variant for date-only string"),
        }
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Test that serialize -> deserialize preserves the value
        let dt = DateTime::FixedOffset(
            FixedOffset::east_opt(5 * 3600 + 30 * 60)
                .unwrap()
                .with_ymd_and_hms(2021, 1, 1, 12, 0, 0)
                .unwrap(),
        );
        let json = serde_json::to_string(&dt).unwrap();
        let dt2: DateTime = serde_json::from_str(&json).unwrap();
        // Timestamps should be equal
        assert_eq!(dt.timestamp(), dt2.timestamp());
    }
}
