use rhai::export_module;

use reearth_flow_common::uri::Uri;

#[export_module]
pub(crate) mod file_module {
    use std::str::FromStr;

    use rhai::plugin::*;

    pub fn extract_filename(file_path: &str) -> String {
        let uri = Uri::from_str(file_path);
        if uri.is_err() {
            return "".to_string();
        }
        uri.unwrap()
            .file_name()
            .map(|s| s.to_str().unwrap_or_default())
            .unwrap_or_default()
            .to_string()
    }

    pub fn join_path(path1: &str, path2: &str) -> String {
        Uri::from_str(path1)
            .and_then(|uri| uri.join(path2))
            .map(|uri| uri.to_string())
            .unwrap_or_default()
    }
}
#[cfg(test)]
mod tests {
    use super::file_module::*;

    #[test]
    fn test_extract_filename() {
        // Test with valid file path
        assert_eq!(extract_filename("/path/to/file.txt"), "file.txt");

        // Test with empty file path
        assert_eq!(extract_filename(""), "");
    }

    #[test]
    fn test_join_path() {
        // Test with valid paths
        assert_eq!(
            join_path("/path/to", "file.txt"),
            "file:///path/to/file.txt"
        );

        // Test with empty paths
        assert_eq!(join_path("", ""), "");

        // Test with invalid paths
        assert_eq!(
            join_path("/path/to/", "file.txt"),
            "file:///path/to/file.txt"
        );
    }
}
