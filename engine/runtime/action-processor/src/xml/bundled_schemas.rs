//! Bundled XML schemas for offline validation
//!
//! This module provides fallback schemas that are bundled with the application
//! to avoid HTTP 429 (Too Many Requests) errors when fetching schemas from
//! external sources like w3.org.

/// Get bundled schema content by URL
///
/// Returns the bundled schema content if available for the given URL.
/// This is used as a fallback when HTTP fetching fails.
pub fn get(url: &str) -> Option<&'static str> {
    match url {
        // W3C schemas
        "http://www.w3.org/1999/xlink.xsd" => {
            Some(include_str!("./bundled_schemas/schemas/w3c/xlink.xsd"))
        }
        "http://www.w3.org/2001/xml.xsd" => {
            Some(include_str!("./bundled_schemas/schemas/w3c/xml.xsd"))
        }

        // OASIS schemas
        "http://docs.oasis-open.org/election/external/xAL.xsd" => {
            Some(include_str!("./bundled_schemas/schemas/oasis/xAL.xsd"))
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_xlink_schema() {
        let schema = get("http://www.w3.org/1999/xlink.xsd");
        assert!(schema.is_some());
        let content = schema.unwrap();
        assert!(content.contains("xlink"));
        assert!(content.contains("http://www.w3.org/1999/xlink"));
    }

    #[test]
    fn test_get_xml_schema() {
        let schema = get("http://www.w3.org/2001/xml.xsd");
        assert!(schema.is_some());
        let content = schema.unwrap();
        assert!(content.contains("xml:lang") || content.contains("xml:space"));
    }

    #[test]
    fn test_get_xal_schema() {
        let schema = get("http://docs.oasis-open.org/election/external/xAL.xsd");
        assert!(schema.is_some());
        let content = schema.unwrap();
        assert!(content.contains("xAL"));
    }

    #[test]
    fn test_get_unknown_schema() {
        let schema = get("http://example.com/unknown.xsd");
        assert!(schema.is_none());
    }
}
