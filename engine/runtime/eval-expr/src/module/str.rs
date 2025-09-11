use regex::Regex;
use rhai::export_module;

#[export_module]
pub(crate) mod str_module {
    use rhai::plugin::*;

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
}
