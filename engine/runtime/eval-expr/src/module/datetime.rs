use rhai::export_module;

#[export_module]
pub(crate) mod datetime_module {
    use chrono::{Datelike, Timelike};
    use rhai::plugin::*;

    pub fn extract_year(datetime: &str) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime).map(|v| v.year()) {
            v.to_string()
        } else {
            "".to_string()
        }
    }

    pub fn extract_month(datetime: &str) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime).map(|v| v.month()) {
            v.to_string()
        } else {
            "".to_string()
        }
    }

    pub fn extract_day(datetime: &str) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime).map(|v| v.day()) {
            v.to_string()
        } else {
            "".to_string()
        }
    }

    pub fn extract_hour(datetime: &str) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            v.naive_local().hour().to_string()
        } else {
            "".to_string()
        }
    }

    pub fn extract_minute(datetime: &str) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            v.naive_local().minute().to_string()
        } else {
            "".to_string()
        }
    }

    pub fn extract_second(datetime: &str) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            v.naive_local().second().to_string()
        } else {
            "".to_string()
        }
    }

    pub fn add_year(datetime: &str, num: i64) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            v.with_year(v.year() + num as i32)
                .map(|v| v.to_rfc3339())
                .unwrap_or_default()
        } else {
            "".to_string()
        }
    }

    pub fn add_month(datetime: &str, num: i64) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            if num < 0 {
                v.with_month((v.month() as i32 - num as i32) as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            } else {
                v.with_month(v.month() + num as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            }
        } else {
            "".to_string()
        }
    }

    pub fn add_day(datetime: &str, num: i64) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            if num < 0 {
                v.with_day((v.day() as i32 - num as i32) as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            } else {
                v.with_day(v.day() + num as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            }
        } else {
            "".to_string()
        }
    }

    pub fn add_hour(datetime: &str, num: i64) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            if num < 0 {
                v.with_hour((v.hour() as i32 - num as i32) as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            } else {
                v.with_hour(v.hour() + num as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            }
        } else {
            "".to_string()
        }
    }

    pub fn add_minute(datetime: &str, num: i64) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            if num < 0 {
                v.with_minute((v.minute() as i32 - num as i32) as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            } else {
                v.with_minute(v.minute() + num as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            }
        } else {
            "".to_string()
        }
    }

    pub fn add_second(datetime: &str, num: i64) -> String {
        if let Ok(v) = reearth_flow_common::datetime::try_from(datetime) {
            if num < 0 {
                v.with_second((v.second() as i32 - num as i32) as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            } else {
                v.with_second(v.second() + num as u32)
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default()
            }
        } else {
            "".to_string()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::datetime_module::*;

    #[test]
    fn test_extract_year() {
        // Test with valid datetime
        assert_eq!(extract_year("2021-01-01"), "2021");

        // Test with invalid datetime
        assert_eq!(extract_year("invalid datetime"), "");
    }

    #[test]
    fn test_extract_month() {
        // Test with valid datetime
        assert_eq!(extract_month("2021-01-01"), "1");

        // Test with invalid datetime
        assert_eq!(extract_month("invalid datetime"), "");
    }

    #[test]
    fn test_extract_day() {
        // Test with valid datetime
        assert_eq!(extract_day("2021-01-01"), "1");

        // Test with invalid datetime
        assert_eq!(extract_day("invalid datetime"), "");
    }

    #[test]
    fn test_extract_hour() {
        // Test with valid datetime
        assert_eq!(extract_hour("2021-01-01T00:00:00Z"), "0");

        // Test with invalid datetime
        assert_eq!(extract_hour("invalid datetime"), "");
    }

    #[test]

    fn test_extract_minute() {
        // Test with valid datetime
        assert_eq!(extract_minute("2021-01-01T00:00:00Z"), "0");

        // Test with invalid datetime
        assert_eq!(extract_minute("invalid datetime"), "");
    }

    #[test]

    fn test_extract_second() {
        // Test with valid datetime
        assert_eq!(extract_second("2021-01-01T00:00:00Z"), "0");

        // Test with invalid datetime
        assert_eq!(extract_second("invalid datetime"), "");
    }

    #[test]
    fn test_add_year() {
        // Test with valid datetime
        assert_eq!(
            add_year("2021-01-01T00:00:00Z", 1),
            "2022-01-01T00:00:00+00:00"
        );

        // Test with invalid datetime
        assert_eq!(add_year("invalid datetime", 1), "");
    }

    #[test]
    fn test_add_month() {
        // Test with valid datetime
        assert_eq!(
            add_month("2021-01-01T00:00:00Z", 1),
            "2021-02-01T00:00:00+00:00"
        );

        // Test with invalid datetime
        assert_eq!(add_month("invalid datetime", 1), "");
    }

    #[test]
    fn test_add_day() {
        // Test with valid datetime
        assert_eq!(
            add_day("2021-01-01T00:00:00Z", 1),
            "2021-01-02T00:00:00+00:00"
        );

        // Test with invalid datetime
        assert_eq!(add_day("invalid datetime", 1), "");
    }

    #[test]
    fn test_add_hour() {
        // Test with valid datetime
        assert_eq!(
            add_hour("2021-01-01T00:00:00Z", 1),
            "2021-01-01T01:00:00+00:00"
        );

        // Test with invalid datetime
        assert_eq!(add_hour("invalid datetime", 1), "");
    }

    #[test]
    fn test_add_minute() {
        // Test with valid datetime
        assert_eq!(
            add_minute("2021-01-01T00:00:00Z", 1),
            "2021-01-01T00:01:00+00:00"
        );

        // Test with invalid datetime
        assert_eq!(add_minute("invalid datetime", 1), "");
    }

    #[test]
    fn test_add_second() {
        // Test with valid datetime
        assert_eq!(
            add_second("2021-01-01T00:00:00Z", 1),
            "2021-01-01T00:00:01+00:00"
        );

        // Test with invalid datetime
        assert_eq!(add_second("invalid datetime", 1), "");
    }
}
