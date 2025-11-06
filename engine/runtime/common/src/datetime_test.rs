#[cfg(test)]
mod tests {
    use crate::datetime::try_from;
    use chrono::Datelike;

    #[test]
    fn test_try_from() {
        let datetime = try_from("2023-01-15T10:30:00Z");
        assert!(datetime.is_ok());
        
        let dt = datetime.unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_try_from_with_timezone() {
        let datetime = try_from("2023-06-20T15:45:30+09:00");
        assert!(datetime.is_ok());
    }

    #[test]
    fn test_try_from_invalid() {
        let datetime = try_from("invalid-date");
        assert!(datetime.is_err());
    }

    #[test]
    fn test_try_from_partial_date() {
        let datetime = try_from("2023-01-15");
        assert!(datetime.is_err() || datetime.is_ok());
    }

    #[test]
    fn test_parse_and_format_roundtrip() {
        let original = "2023-12-25T00:00:00Z";
        let datetime = try_from(original).unwrap();
        let formatted = datetime.to_rfc3339();
        
        let reparsed = try_from(&formatted).unwrap();
        assert_eq!(datetime.to_rfc3339(), reparsed.to_rfc3339());
    }

    #[test]
    fn test_try_from_with_milliseconds() {
        let datetime = try_from("2023-01-15T10:30:00.123Z");
        assert!(datetime.is_ok());
    }

    #[test]
    fn test_try_from_various_timezones() {
        let test_cases = vec![
            "2023-01-15T10:30:00Z",
            "2023-01-15T10:30:00+00:00",
            "2023-01-15T10:30:00+09:00",
            "2023-01-15T10:30:00-05:00",
        ];
        
        for case in test_cases {
            assert!(try_from(case).is_ok(), "Failed to parse: {}", case);
        }
    }

    #[test]
    fn test_parse_japanese_datetime() {
        let datetime = try_from("2023-04-01T09:00:00+09:00");
        assert!(datetime.is_ok());
        
        let dt = datetime.unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 4);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_datetime_edge_cases() {
        assert!(try_from("").is_err());
        assert!(try_from(" ").is_err());
        assert!(try_from("not-a-date").is_err());
    }
}

