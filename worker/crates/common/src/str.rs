use base64::{engine::general_purpose, Engine as _};

pub fn remove_bom(s: &str) -> &str {
    if s.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) {
        &s[3..]
    } else {
        s
    }
}

pub fn base64_encode<T: AsRef<[u8]>>(s: T) -> String {
    general_purpose::STANDARD.encode(s)
}

pub fn remove_trailing_slash(s: &str) -> String {
    if s.ends_with('/') {
        s.strip_suffix('/').unwrap_or_default().to_string()
    } else {
        s.to_string()
    }
}

pub fn is_boolean(s: &str) -> bool {
    matches!(s.to_ascii_lowercase().as_str(), "true" | "false")
}

pub fn is_number(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_bom() {
        // Test case with BOM
        let s = "\u{feff}Hello, World!";
        assert_eq!(remove_bom(s), "Hello, World!");

        // Test case without BOM
        let s = "Hello, World!";
        assert_eq!(remove_bom(s), "Hello, World!");
    }

    #[test]
    fn test_base64_encode() {
        // Test case with empty string
        let s = "";
        assert_eq!(base64_encode(s), "");

        // Test case with non-empty string
        let s = "Hello, World!";
        assert_eq!(base64_encode(s), "SGVsbG8sIFdvcmxkIQ==");

        // Test case with special characters
        let s = "Hello, @#$%^&*()_+{}:\"<>?|\\ World!";
        assert_eq!(
            base64_encode(s),
            "SGVsbG8sIEAjJCVeJiooKV8re306Ijw+P3xcIFdvcmxkIQ=="
        );

        // Test case with Unicode characters
        let s = "こんにちは、世界！";
        assert_eq!(base64_encode(s), "44GT44KT44Gr44Gh44Gv44CB5LiW55WM77yB");

        // Test case with binary data
        let s = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        assert_eq!(base64_encode(s), "AAECAwQF");
    }

    #[test]
    fn test_remove_trailing_slash() {
        // Test case with trailing slash
        let s = "path/";
        assert_eq!(remove_trailing_slash(s), "path");

        // Test case without trailing slash
        let s = "path";
        assert_eq!(remove_trailing_slash(s), "path");

        // Test case with multiple trailing slashes
        let s = "path////";
        assert_eq!(remove_trailing_slash(s), "path///");

        // Test case with empty string
        let s = "";
        assert_eq!(remove_trailing_slash(s), "");

        // Test case with only slash
        let s = "/";
        assert_eq!(remove_trailing_slash(s), "");
    }
}
