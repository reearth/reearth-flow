use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};

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

pub fn base64_decode<T: AsRef<[u8]>>(s: T) -> crate::Result<String> {
    let result = base64_decode_byte(s)?;
    Ok(String::from_utf8_lossy(&result).to_string())
}

pub fn base64_decode_byte<T: AsRef<[u8]>>(s: T) -> crate::Result<Vec<u8>> {
    general_purpose::STANDARD
        .decode(s)
        .map_err(|e| crate::Error::Str(format!("{e}")))
}

pub fn remove_trailing_slash(s: &str) -> String {
    if s.ends_with('/') {
        s.strip_suffix('/').unwrap_or_default().to_string()
    } else {
        s.to_string()
    }
}

pub fn is_boolean<T: AsRef<str>>(s: T) -> bool {
    matches!(
        s.as_ref().to_ascii_lowercase().as_str(),
        "true" | "false" | "yes" | "no" | "on" | "off" | "1" | "0"
    )
}

pub fn parse_boolean<T: AsRef<str>>(s: T) -> bool {
    match s.as_ref().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => true,
        "false" | "no" | "off" | "0" => false,
        _ => unreachable!(),
    }
}

pub fn to_hash(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let mut result = String::new();
    for b in hasher.finalize() {
        result.push_str(&format!("{b:02x}"));
    }
    result
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

    #[test]
    fn test_to_hash() {
        // Test case with empty string
        let s = "";
        assert_eq!(
            to_hash(s),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        // Test case with non-empty string
        let s = "Hello, World!";
        assert_eq!(
            to_hash(s),
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );

        // Test case with special characters
        let s = "Hello, @#$%^&*()_+{}:\"<>?|\\ World!";
        assert_eq!(
            to_hash(s),
            "8173c9467d7111acce18d5fcd7de17c548de44737692fec3bf370ac868d9c168"
        );

        // Test case with Unicode characters
        let s = "こんにちは、世界！";
        assert_eq!(
            to_hash(s),
            "81cf0fb2f41dab4e93c086815bc082140642d0efa1155398597a823c232bf4fa"
        );
    }
}
