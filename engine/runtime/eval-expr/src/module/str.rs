use regex::Regex;
use rhai::export_module;

#[export_module]
pub(crate) mod str_module {
    use rhai::plugin::*;

    pub fn extract_single_by_regex(regex: &str, haystack: &str) -> String {
        let regex = Regex::new(regex).unwrap();
        let capture = regex.captures(haystack);
        match capture {
            Some(capture) => capture.get(1).unwrap().as_str().to_string(),
            None => "".to_string(),
        }
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
}
