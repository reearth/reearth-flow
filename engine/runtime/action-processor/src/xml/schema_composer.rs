use std::collections::HashMap;
use std::path::PathBuf;

/// Generate a wrapper schema that imports all specified schemas
pub fn generate_wrapper_schema(
    schema_locations: &[(String, String)],
    cached_paths: &HashMap<String, PathBuf>,
) -> String {
        let imports = schema_locations
            .iter()
            .filter_map(|(namespace, location)| {
                // Use cached path if available, otherwise use original location
                let resolved_location = cached_paths
                    .get(location)
                    .and_then(|p| p.to_str())
                    .unwrap_or(location);

                // Skip empty namespaces
                if namespace.is_empty() {
                    None
                } else {
                    Some(format!(
                        r#"  <xs:import namespace="{}" schemaLocation="{}"/>"#,
                        namespace, resolved_location
                    ))
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           elementFormDefault="qualified"
           attributeFormDefault="unqualified">
{}
</xs:schema>"#,
            imports
        )
}

/// Generate an XML catalog file for schema resolution
pub fn generate_catalog(mappings: &HashMap<String, PathBuf>) -> String {
        let entries = mappings
            .iter()
            .filter_map(|(url, path)| {
                path.to_str()
                    .map(|p| format!(r#"  <uri name="{}" uri="file://{}"/>"#, url, p))
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE catalog PUBLIC "-//OASIS//DTD XML Catalogs V1.1//EN"
  "http://www.oasis-open.org/committees/entity/release/1.1/catalog.dtd">
<catalog xmlns="urn:oasis:names:tc:entity:xmlns:xml:catalog">
{}
</catalog>"#,
            entries
        )
}

/// Generate a unique cache key for a combination of schemas
/// Uses both namespace URIs and schema locations for uniqueness
pub fn generate_composite_cache_key(schema_locations: &[(String, String)]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Sort to ensure consistent key generation
        let mut sorted_locations = schema_locations.to_vec();
        sorted_locations.sort();

        // Hash both namespace URI and schema location
        for (namespace_uri, schema_location) in sorted_locations {
            namespace_uri.hash(&mut hasher);
            schema_location.hash(&mut hasher);
        }

        format!("wrapper_schema_{:x}", hasher.finish())
}

/// Generate a unique cache key for catalog based on namespace URIs and URLs
pub fn generate_catalog_cache_key(mappings: &HashMap<String, PathBuf>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Sort mappings to ensure consistent key generation
        let mut sorted_mappings: Vec<_> = mappings.iter().collect();
        sorted_mappings.sort_by_key(|(url, _)| *url);

        for (url, path) in sorted_mappings {
            url.hash(&mut hasher);
            path.to_string_lossy().hash(&mut hasher);
        }

    format!("catalog_{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wrapper_schema() {
        let schema_locations = vec![
            (
                "http://example.com/ns1".to_string(),
                "schema1.xsd".to_string(),
            ),
            (
                "http://example.com/ns2".to_string(),
                "schema2.xsd".to_string(),
            ),
        ];

        let cached_paths = HashMap::new();

        let wrapper = generate_wrapper_schema(&schema_locations, &cached_paths);

        assert!(wrapper.contains(r#"namespace="http://example.com/ns1""#));
        assert!(wrapper.contains(r#"namespace="http://example.com/ns2""#));
        assert!(wrapper.contains(r#"schemaLocation="schema1.xsd""#));
        assert!(wrapper.contains(r#"schemaLocation="schema2.xsd""#));
    }

    #[test]
    fn test_generate_catalog() {
        let mut mappings = HashMap::new();
        mappings.insert(
            "http://example.com/schema1.xsd".to_string(),
            PathBuf::from("/cache/schema1.xsd"),
        );
        mappings.insert(
            "http://example.com/schema2.xsd".to_string(),
            PathBuf::from("/cache/schema2.xsd"),
        );

        let catalog = generate_catalog(&mappings);

        assert!(catalog.contains(r#"name="http://example.com/schema1.xsd""#));
        assert!(catalog.contains(r#"uri="file:///cache/schema1.xsd""#));
    }

    #[test]
    fn test_composite_cache_key_consistency() {
        let locations1 = vec![
            ("ns1".to_string(), "loc1".to_string()),
            ("ns2".to_string(), "loc2".to_string()),
        ];

        let locations2 = vec![
            ("ns2".to_string(), "loc2".to_string()),
            ("ns1".to_string(), "loc1".to_string()),
        ];

        // Same locations in different order should produce the same key
        let key1 = generate_composite_cache_key(&locations1);
        let key2 = generate_composite_cache_key(&locations2);

        assert_eq!(key1, key2);
    }
}
