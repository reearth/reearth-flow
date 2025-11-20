use std::collections::HashMap;
use std::sync::Arc;

use super::cache::SchemaCache;
use super::errors::{Result, XmlProcessorError};
use super::schema_resolver::SchemaResolutionResult;

/// Handles schema rewriting and caching operations
#[derive(Clone)]
pub struct SchemaRewriter {
    cache: Arc<dyn SchemaCache>,
}

impl SchemaRewriter {
    pub fn new(cache: Arc<dyn SchemaCache>) -> Self {
        Self { cache }
    }

    /// Process schemas: rewrite imports and cache them
    /// Returns the path to the cached root schema
    pub fn process_and_cache_schemas(
        &self,
        target_url: &str,
        resolution: &SchemaResolutionResult,
    ) -> Result<std::path::PathBuf> {
        // Check if already cached
        let target_cache_key = generate_cache_key(target_url);
        if self.cache.is_cached(&target_cache_key) {
            tracing::debug!("Schema already cached: {}", target_url);
            return self.cache.get_schema_path(&target_cache_key);
        }

        tracing::debug!(
            "Processing {} schemas for caching",
            resolution.schemas.len()
        );

        // Build URL to cache path mapping for all schemas
        let mut url_to_cache_path = HashMap::new();
        for url in resolution.schemas.keys() {
            let cache_key = generate_cache_key(url);
            let cache_path = self.cache.get_schema_path(&cache_key)?;
            url_to_cache_path.insert(url.clone(), cache_path);
        }

        // Process and cache each schema
        for (url, resolved_schema) in &resolution.schemas {
            // Rewrite import paths in schema content
            let mut content = resolved_schema.content.clone();
            content = rewrite_schema_imports(&content, url, &url_to_cache_path)?;

            let cache_key = generate_cache_key(url);

            // Log rewritten imports/includes for debugging
            if content.contains("schemaLocation") {
                tracing::debug!("Schema {} contains schemaLocation references", url);
                // Log all schemaLocation values for debugging
                for line in content.lines() {
                    if line.contains("schemaLocation") {
                        tracing::debug!("  {}", line.trim());
                    }
                }
            }

            // Remove XML declaration from non-root schemas to prevent
            // "multiple XML declaration" errors when schemas are included
            let final_content = if url != target_url {
                remove_xml_declaration(&content).to_string()
            } else {
                content
            };

            // Save rewritten schema content to cache
            tracing::debug!("Saving schema to cache: {} -> {}", url, cache_key);
            let cache_path = self.cache.get_schema_path(&cache_key)?;
            tracing::debug!("Cache path for {}: {:?}", url, cache_path);
            self.cache
                .put_schema(&cache_key, final_content.as_bytes())?;
        }

        // Return the cached root schema path
        self.cache.get_schema_path(&target_cache_key)
    }
}

/// Rewrite schema import paths to use local file paths
fn rewrite_schema_imports(
    content: &str,
    current_url: &str,
    url_to_cache_path: &HashMap<String, std::path::PathBuf>,
) -> Result<String> {
    let mut content = content.to_string();

    // Replace each import URL with its cache path
    for (import_url, cache_path) in url_to_cache_path {
        if import_url == current_url {
            continue; // Skip self-references
        }

        let cache_path_str = cache_path.to_str().ok_or_else(|| {
            XmlProcessorError::Validator(format!(
                "Invalid cache path for {import_url}: {cache_path:?}"
            ))
        })?;

        // Always use absolute file:// URLs
        let replacement_path = format!("file://{cache_path_str}");

        // Replace absolute URLs with appropriate paths
        let old_pattern1 = format!(r#"schemaLocation="{import_url}""#);
        let new_pattern1 = format!(r#"schemaLocation="{replacement_path}""#);
        if content.contains(&old_pattern1) {
            tracing::debug!("Replacing {} with {}", old_pattern1, new_pattern1);
            content = content.replace(&old_pattern1, &new_pattern1);
        }

        let old_pattern2 = format!(r#"schemaLocation='{import_url}'"#);
        let new_pattern2 = format!(r#"schemaLocation='{replacement_path}'"#);
        if content.contains(&old_pattern2) {
            tracing::debug!("Replacing {} with {}", old_pattern2, new_pattern2);
            content = content.replace(&old_pattern2, &new_pattern2);
        }

        // Also handle relative paths in the original schema
        // For simplicity, just handle filename-only references
        if let Some(last_slash) = import_url.rfind('/') {
            let import_filename = &import_url[last_slash + 1..];
            content = content
                .replace(
                    &format!(r#"schemaLocation="{import_filename}""#),
                    &format!(r#"schemaLocation="{replacement_path}""#),
                )
                .replace(
                    &format!(r#"schemaLocation='{import_filename}'"#),
                    &format!(r#"schemaLocation='{replacement_path}'"#),
                );
        }
    }

    Ok(content)
}

/// Generate a cache key for a schema URL
/// Preserves directory structure for relative path references
pub fn generate_cache_key(url: &str) -> String {
    // Remove protocol and host to get path-like structure
    if let Some(protocol_end) = url.find("://") {
        let after_protocol = &url[protocol_end + 3..];
        // Find the start of the path (after host)
        if let Some(path_start) = after_protocol.find('/') {
            let path = &after_protocol[path_start + 1..];
            // Use a hash of the host as prefix to avoid conflicts
            let host = &after_protocol[..path_start];
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            host.hash(&mut hasher);
            let host_hash = format!("{:x}", hasher.finish());
            // Preserve directory structure by keeping the path as-is
            return format!("xmlvalidator-schema/{}/{}", &host_hash[..8], path);
        }
    }
    // Fallback to simple hash-based approach
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    url.hash(&mut hasher);
    format!("xmlvalidator-schema/{:x}", hasher.finish())
}

/// Remove XML declaration from content to prevent "multiple XML declaration" errors
/// when schemas are included by other schemas
fn remove_xml_declaration(content: &str) -> &str {
    let trimmed = content.trim_start();
    if trimmed.starts_with("<?xml") {
        if let Some(end_pos) = trimmed.find("?>") {
            return trimmed[end_pos + 2..].trim_start();
        }
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::cache::create_filesystem_cache;
    use crate::xml::schema_fetcher::MockSchemaFetcher;
    use crate::xml::schema_resolver::XmlSchemaResolver;
    use std::env;

    #[test]
    fn test_generate_cache_key_preserves_structure() {
        let url = "http://example.com/schemas/main.xsd";
        let key = generate_cache_key(url);
        assert!(key.contains("schemas/main.xsd"));

        let nested_url = "http://example.com/schemas/v2/nested/schema.xsd";
        let nested_key = generate_cache_key(nested_url);
        assert!(nested_key.contains("schemas/v2/nested/schema.xsd"));
    }

    #[test]
    fn test_remove_xml_declaration() {
        let with_declaration = r#"<?xml version="1.0" encoding="UTF-8"?>
<schema>content</schema>"#;
        let result = remove_xml_declaration(with_declaration);
        assert!(!result.starts_with("<?xml"));
        assert!(result.trim_start().starts_with("<schema>"));

        let without_declaration = "<schema>content</schema>";
        let result2 = remove_xml_declaration(without_declaration);
        assert_eq!(result2, without_declaration);
    }

    #[test]
    fn test_schema_rewriter_with_dependencies() {
        let main_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:dep="http://example.com/dep"
           targetNamespace="http://example.com/main">
    <xs:import namespace="http://example.com/dep" 
               schemaLocation="http://example.com/dep.xsd"/>
</xs:schema>"#;

        let dep_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/dep">
    <xs:element name="item" type="xs:string"/>
</xs:schema>"#;

        // Create mock fetcher
        let fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/main.xsd", Ok(main_schema.to_string()))
            .with_response("http://example.com/dep.xsd", Ok(dep_schema.to_string()));

        // Create filesystem cache
        let temp_dir =
            env::temp_dir().join(format!("schema_rewriter_test_{}", uuid::Uuid::new_v4()));
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        // Create rewriter and resolver
        let rewriter = SchemaRewriter::new(cache.clone());
        let resolver = XmlSchemaResolver::new(Arc::new(fetcher));

        // First resolve schemas
        let resolution = resolver
            .resolve_schema_dependencies("http://example.com/main.xsd")
            .unwrap();

        // Then process schemas
        let result = rewriter.process_and_cache_schemas("http://example.com/main.xsd", &resolution);
        assert!(result.is_ok());

        let cached_path = result.unwrap();
        assert!(cached_path.exists());

        // Check that the import was rewritten
        let cached_content = std::fs::read_to_string(&cached_path).unwrap();
        assert!(cached_content.contains("schemaLocation=\"file://"));
        assert!(!cached_content.contains("http://example.com/dep.xsd"));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_schema_rewriter_cache_hit() {
        let schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="test" type="xs:string"/>
</xs:schema>"#;

        let fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/test.xsd", Ok(schema.to_string()));

        let temp_dir =
            env::temp_dir().join(format!("schema_rewriter_test_{}", uuid::Uuid::new_v4()));
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        let rewriter = SchemaRewriter::new(cache.clone());
        let resolver = XmlSchemaResolver::new(Arc::new(fetcher));

        // Resolve schemas once
        let resolution = resolver
            .resolve_schema_dependencies("http://example.com/test.xsd")
            .unwrap();

        // First call should fetch and cache
        let result1 =
            rewriter.process_and_cache_schemas("http://example.com/test.xsd", &resolution);
        assert!(result1.is_ok());

        // Second call should hit cache
        let result2 =
            rewriter.process_and_cache_schemas("http://example.com/test.xsd", &resolution);
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_schema_with_multiple_xml_declarations() {
        // This test reproduces an issue where multiple schemas are imported and each contains
        // its own XML declaration. When inlined, this causes parse errors.
        let main_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:dep1="http://example.com/dep1"
           xmlns:dep2="http://example.com/dep2"
           targetNamespace="http://example.com/main">
    <xs:import namespace="http://example.com/dep1" 
               schemaLocation="http://example.com/dep1.xsd"/>
    <xs:import namespace="http://example.com/dep2" 
               schemaLocation="http://example.com/dep2.xsd"/>
</xs:schema>"#;

        let dep1_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/dep1">
    <xs:element name="item1" type="xs:string"/>
</xs:schema>"#;

        let dep2_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/dep2">
    <xs:element name="item2" type="xs:string"/>
</xs:schema>"#;

        let fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/main.xsd", Ok(main_schema.to_string()))
            .with_response("http://example.com/dep1.xsd", Ok(dep1_schema.to_string()))
            .with_response("http://example.com/dep2.xsd", Ok(dep2_schema.to_string()));

        let temp_dir =
            env::temp_dir().join(format!("schema_multi_decl_test_{}", uuid::Uuid::new_v4()));
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        let rewriter = SchemaRewriter::new(cache.clone());
        let resolver = XmlSchemaResolver::new(Arc::new(fetcher));

        // Resolve schemas first
        let resolution = resolver
            .resolve_schema_dependencies("http://example.com/main.xsd")
            .unwrap();

        // Process schemas - should handle multiple XML declarations properly
        let result = rewriter.process_and_cache_schemas("http://example.com/main.xsd", &resolution);
        assert!(
            result.is_ok(),
            "Should process schemas with multiple imports"
        );

        let cached_path = result.unwrap();
        let cached_content = std::fs::read_to_string(&cached_path).unwrap();

        // Verify no XML declarations in the middle of content
        let decl_count = cached_content.matches("<?xml").count();
        assert_eq!(
            decl_count, 1,
            "Should only have one XML declaration at the start"
        );

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_nested_schema_dependencies() {
        // Reproduce the error: "failed to load external entity"
        // This occurs when a schema imports another schema, and the imported schema URL fails to load
        let main_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:level1="http://example.com/level1"
           targetNamespace="http://example.com/main">
    <xs:import namespace="http://example.com/level1" 
               schemaLocation="http://example.com/level1.xsd"/>
</xs:schema>"#;

        let level1_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:level2="http://example.com/level2"
           targetNamespace="http://example.com/level1">
    <xs:import namespace="http://example.com/level2" 
               schemaLocation="http://example.com/level2.xsd"/>
    <xs:element name="level1item" type="xs:string"/>
</xs:schema>"#;

        // Deliberately omit level2.xsd to simulate fetch failure
        let fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/main.xsd", Ok(main_schema.to_string()))
            .with_response(
                "http://example.com/level1.xsd",
                Ok(level1_schema.to_string()),
            )
            .with_response(
                "http://example.com/level2.xsd",
                Err(super::super::errors::XmlProcessorError::Validator(
                    "Schema fetch failed".to_string(),
                )),
            );

        let temp_dir = env::temp_dir().join(format!("schema_nested_test_{}", uuid::Uuid::new_v4()));
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        let _rewriter = SchemaRewriter::new(cache.clone());
        let resolver = XmlSchemaResolver::new(Arc::new(fetcher));

        // This should fail when resolving dependencies
        let resolution_result = resolver.resolve_schema_dependencies("http://example.com/main.xsd");
        assert!(
            resolution_result.is_err(),
            "Should fail when nested dependency cannot be fetched"
        );

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_resolved_nested_dependencies() {
        // Test successful validation when all nested schema dependencies are resolved
        let main_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:level1="http://example.com/level1"
           targetNamespace="http://example.com/main">
    <xs:import namespace="http://example.com/level1" 
               schemaLocation="http://example.com/level1.xsd"/>
    <xs:element name="root">
        <xs:complexType>
            <xs:sequence>
                <xs:element ref="level1:item"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        let level1_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:level2="http://example.com/level2"
           targetNamespace="http://example.com/level1">
    <xs:import namespace="http://example.com/level2" 
               schemaLocation="http://example.com/level2.xsd"/>
    <xs:element name="item">
        <xs:complexType>
            <xs:sequence>
                <xs:element ref="level2:subitem"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        let level2_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/level2">
    <xs:element name="subitem" type="xs:string"/>
</xs:schema>"#;

        // Provide all schemas
        let fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/main.xsd", Ok(main_schema.to_string()))
            .with_response(
                "http://example.com/level1.xsd",
                Ok(level1_schema.to_string()),
            )
            .with_response(
                "http://example.com/level2.xsd",
                Ok(level2_schema.to_string()),
            );

        let temp_dir =
            env::temp_dir().join(format!("schema_resolved_test_{}", uuid::Uuid::new_v4()));
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        let rewriter = SchemaRewriter::new(cache.clone());
        let resolver = XmlSchemaResolver::new(Arc::new(fetcher));

        // First resolve schemas
        let resolution = resolver
            .resolve_schema_dependencies("http://example.com/main.xsd")
            .unwrap();

        // Process all schemas successfully
        let result = rewriter.process_and_cache_schemas("http://example.com/main.xsd", &resolution);
        assert!(
            result.is_ok(),
            "Should succeed when all dependencies are resolved"
        );

        let cached_path = result.unwrap();
        assert!(cached_path.exists());

        // Verify all imports were rewritten to local files
        let cached_content = std::fs::read_to_string(&cached_path).unwrap();
        assert!(cached_content.contains("schemaLocation=\"file://"));
        assert!(!cached_content.contains("http://example.com/level1.xsd"));

        // Check that level1 schema was also properly cached and rewritten
        let level1_cache_key = generate_cache_key("http://example.com/level1.xsd");
        let level1_path = cache.get_schema_path(&level1_cache_key).unwrap();
        let level1_content = std::fs::read_to_string(&level1_path).unwrap();
        assert!(level1_content.contains("schemaLocation=\"file://"));
        assert!(!level1_content.contains("http://example.com/level2.xsd"));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
