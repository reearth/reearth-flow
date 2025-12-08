use regex::Regex;
use rhai::export_module;

#[export_module]
pub(crate) mod str_module {
    use rhai::{plugin::*, Dynamic};

    /// Extracts the first captured group from a regex match in the given text.
    /// Returns the captured text as a string, or empty string if no match or no capture group.
    pub fn extract_single_by_regex(regex: &str, haystack: &str) -> String {
        let regex = Regex::new(regex).unwrap();
        let capture = regex.captures(haystack);
        match capture {
            Some(capture) => capture.get(1).unwrap().as_str().to_string(),
            None => "".to_string(),
        }
    }

    /// Checks if the given text matches the specified regular expression pattern.
    /// Returns true if the pattern matches anywhere in the text, false otherwise.
    pub fn matches(haystack: &str, regex: &str) -> bool {
        let regex = Regex::new(regex).unwrap();
        regex.is_match(haystack)
    }

    pub fn string_to_int(s: String) -> i64 {
        s.parse::<i64>().unwrap()
    }

    pub fn string_to_f64(s: String) -> f64 {
        s.parse::<f64>().unwrap()
    }

    pub fn sub_str_from_index_inclusive(s: String, start: i64, end: i64) -> String {
        if start < 0 || end < 0 {
            return String::new();
        }

        let start = start as usize;
        let end = end as usize;

        if start > end {
            return String::new();
        }

        let chars: Vec<char> = s.chars().collect();

        if start >= chars.len() {
            return String::new();
        }

        let end_idx = std::cmp::min(end + 1, chars.len()); // +1 to make end inclusive

        chars[start..end_idx].iter().collect()
    }
}
#[cfg(test)]
mod tests {
    use super::str_module::*;

    #[test]
    fn test_extract_single_by_regex() {
        // Test case 1: Regex matches and captures a single group
        let regex = r"(\d+)";
        let haystack = "abc123def";
        let result = extract_single_by_regex(regex, haystack);
        assert_eq!(result, "123");

        // Test case 2: Regex matches but does not capture any group
        let regex = r"\d+";
        let haystack = "abcdef";
        let result = extract_single_by_regex(regex, haystack);
        assert_eq!(result, "");

        // Test case 3: Regex does not match
        let regex = r"\d+";
        let haystack = "abcdef";
        let result = extract_single_by_regex(regex, haystack);
        assert_eq!(result, "");
    }

    #[test]
    fn test_matches() {
        // Test case 1: Pattern matches
        let haystack = "12345-bldg-789";
        let regex = r"^\d{5}-bldg-\d+$";
        let result = matches(haystack, regex);
        assert!(result);

        // Test case 2: Pattern does not match
        let haystack = "invalid-format";
        let regex = r"^\d{5}-bldg-\d+$";
        let result = matches(haystack, regex);
        assert!(!result);

        // Test case 3: Partial match
        let haystack = "prefix-12345-bldg-789-suffix";
        let regex = r"\d{5}-bldg-\d+";
        let result = matches(haystack, regex);
        assert!(result);

        // Test case 4: Test our specific failing case from GML file
        let haystack = "1621-bldg-77"; // Only 4 digits, should fail
        let regex = r"^\d{5}-bldg-\d+$";
        let result = matches(haystack, regex);
        assert!(!result); // This should fail because 1621 has only 4 digits, not 5
    }

    #[test]
    fn test_sub_str_from_index_inclusive() {
        // Basic functionality: extract substring from inclusive start to inclusive end
        let result = sub_str_from_index_inclusive("hello".to_string(), 1, 3);
        assert_eq!(result, "ell"); // Indexes 1, 2, 3 correspond to 'e', 'l', 'l'

        // Extract entire string
        let result = sub_str_from_index_inclusive("hello".to_string(), 0, 4);
        assert_eq!(result, "hello");

        // Extract first character
        let result = sub_str_from_index_inclusive("hello".to_string(), 0, 0);
        assert_eq!(result, "h");

        // Extract last character
        let result = sub_str_from_index_inclusive("hello".to_string(), 4, 4);
        assert_eq!(result, "o");

        // Out of bounds end index should be handled gracefully
        let result = sub_str_from_index_inclusive("hello".to_string(), 3, 10);
        assert_eq!(result, "lo");

        // Start index greater than end index should return empty string
        let result = sub_str_from_index_inclusive("hello".to_string(), 3, 1);
        assert_eq!(result, "");

        // Negative indices should return empty string
        let result = sub_str_from_index_inclusive("hello".to_string(), -1, 4);
        assert_eq!(result, "");
        let result = sub_str_from_index_inclusive("hello".to_string(), 0, -1);
        assert_eq!(result, "");

        // Start index beyond string length should return empty string
        let result = sub_str_from_index_inclusive("hello".to_string(), 10, 15);
        assert_eq!(result, "");

        // Empty string should return empty string
        let result = sub_str_from_index_inclusive("".to_string(), 0, 5);
        assert_eq!(result, "");

        // Unicode characters should work correctly
        let result = sub_str_from_index_inclusive("héllo".to_string(), 1, 3);
        assert_eq!(result, "éll");
    }
}
