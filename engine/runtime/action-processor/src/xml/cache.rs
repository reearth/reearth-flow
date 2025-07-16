use std::path::PathBuf;
use std::sync::Arc;

use super::errors::Result;

/// Trait for caching operations needed by XML validator
/// This abstracts the cache storage to make it easier to test and use
pub trait SchemaCache: Send + Sync {
    /// Save raw schema content to cache
    fn put_schema(&self, key: &str, content: &[u8]) -> Result<()>;

    /// Get raw schema content from cache
    #[allow(dead_code)]
    fn get_schema(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Get the file path for a cached schema (for libxml to access)
    fn get_schema_path(&self, key: &str) -> Result<std::path::PathBuf>;

    /// Check if cache is available
    fn is_available(&self) -> bool;

    /// Check if a schema is already cached
    fn is_cached(&self, key: &str) -> bool;
}

/// No-op implementation when cache is not available
pub struct NoOpSchemaCache;

impl SchemaCache for NoOpSchemaCache {
    fn put_schema(&self, _key: &str, _content: &[u8]) -> Result<()> {
        Ok(())
    }

    fn get_schema(&self, _key: &str) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn get_schema_path(&self, _key: &str) -> Result<std::path::PathBuf> {
        Err(super::errors::XmlProcessorError::Validator(
            "Schema cache not available".to_string(),
        ))
    }

    fn is_available(&self) -> bool {
        false
    }

    fn is_cached(&self, _key: &str) -> bool {
        false
    }
}

/// Filesystem-based implementation of SchemaCache
pub struct FileSystemSchemaCache {
    root_path: PathBuf,
}

impl FileSystemSchemaCache {
    pub fn new(root_path: PathBuf) -> Result<Self> {
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&root_path).map_err(|e| {
            super::errors::XmlProcessorError::Validator(format!(
                "Failed to create cache directory: {e}"
            ))
        })?;

        Ok(Self { root_path })
    }

    fn get_full_path(&self, key: &str) -> PathBuf {
        // Replace backslashes to avoid path traversal issues
        let safe_key = key.replace('\\', "_");

        // Split by forward slash and build proper path for the platform
        let path_parts: Vec<&str> = safe_key.split('/').collect();

        // Build path using proper platform separators
        let mut path = self.root_path.clone();
        for (i, part) in path_parts.iter().enumerate() {
            if i == path_parts.len() - 1 {
                // Last part is the filename
                if part.ends_with(".xsd") {
                    path = path.join(part);
                } else {
                    path = path.join(format!("{part}.xsd"));
                }
            } else {
                // Directory parts
                path = path.join(part);
            }
        }

        path
    }
}

impl SchemaCache for FileSystemSchemaCache {
    fn put_schema(&self, key: &str, content: &[u8]) -> Result<()> {
        let path = self.get_full_path(key);
        tracing::debug!(
            "Saving schema to cache: key={}, path={}",
            key,
            path.display()
        );

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                super::errors::XmlProcessorError::Validator(format!(
                    "Failed to create parent directories: {e}"
                ))
            })?;
        }

        std::fs::write(&path, content).map_err(|e| {
            super::errors::XmlProcessorError::Validator(format!(
                "Failed to write schema to {}: {e}",
                path.display()
            ))
        })?;

        tracing::debug!("Successfully saved schema to {}", path.display());
        Ok(())
    }

    fn get_schema(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_full_path(key);

        match std::fs::read(&path) {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(super::errors::XmlProcessorError::Validator(format!(
                "Failed to read schema from {}: {e}",
                path.display()
            ))),
        }
    }

    fn get_schema_path(&self, key: &str) -> Result<std::path::PathBuf> {
        Ok(self.get_full_path(key))
    }

    fn is_available(&self) -> bool {
        true
    }

    fn is_cached(&self, key: &str) -> bool {
        let path = self.get_full_path(key);
        path.exists()
    }
}

/// Create a filesystem-based schema cache
pub fn create_filesystem_cache(root_path: PathBuf) -> Result<Arc<dyn SchemaCache>> {
    Ok(Arc::new(FileSystemSchemaCache::new(root_path)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::env;

    #[test]
    fn test_filesystem_cache_basic_operations() {
        // Create a temporary directory for testing
        let temp_dir = env::temp_dir().join(format!("xml_cache_test_{}", uuid::Uuid::new_v4()));

        // Create filesystem cache
        let cache = FileSystemSchemaCache::new(temp_dir.clone()).unwrap();

        // Test put and get
        let key = "test_schema";
        let content = b"<?xml version=\"1.0\"?><schema>test</schema>";

        // Test is_cached before putting schema
        assert!(!cache.is_cached(key));

        // Put schema
        cache.put_schema(key, content).unwrap();

        // Test is_cached after putting schema
        assert!(cache.is_cached(key));

        // Get schema
        let retrieved = cache.get_schema(key).unwrap();
        assert_eq!(retrieved, Some(content.to_vec()));

        // Get non-existent schema
        let not_found = cache.get_schema("non_existent").unwrap();
        assert_eq!(not_found, None);

        // Test is_cached for non-existent key
        assert!(!cache.is_cached("non_existent"));

        // Check path
        let path = cache.get_schema_path(key).unwrap();
        assert!(path.exists());
        assert!(path.to_str().unwrap().contains("test_schema.xsd"));

        // Check availability
        assert!(cache.is_available());

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_filesystem_cache_path_safety() {
        let temp_dir = env::temp_dir().join(format!("xml_cache_test_{}", uuid::Uuid::new_v4()));
        let cache = FileSystemSchemaCache::new(temp_dir.clone()).unwrap();

        // Test that forward slashes are preserved for directory structure
        let key_with_slashes = "path/to/schema";
        let path1 = cache.get_schema_path(key_with_slashes).unwrap();

        // Forward slashes should be preserved
        assert_eq!(path1.file_name().unwrap().to_str().unwrap(), "schema.xsd");
        assert!(path1.to_str().unwrap().contains("path/to/"));

        // Test that backslashes are replaced for safety
        let key_with_backslashes = "path\\to\\schema";
        let path2 = cache.get_schema_path(key_with_backslashes).unwrap();

        // Backslashes should be replaced
        assert!(path2
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("path_to_schema"));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_filesystem_cache_nested_keys() {
        let temp_dir = env::temp_dir().join(format!("xml_cache_test_{}", uuid::Uuid::new_v4()));
        let cache = FileSystemSchemaCache::new(temp_dir.clone()).unwrap();

        // Test with nested-looking keys
        let key = "xmlvalidator-schema/12345678-schema.xsd";
        let content = b"<schema>nested</schema>";

        cache.put_schema(key, content).unwrap();

        let retrieved = cache.get_schema(key).unwrap();
        assert_eq!(retrieved, Some(content.to_vec()));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_create_filesystem_cache() {
        let temp_dir = env::temp_dir().join(format!("xml_cache_test_{}", uuid::Uuid::new_v4()));

        // Create using factory function
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        // Test basic operations
        let key = "factory_test";
        let content = b"factory content";

        cache.put_schema(key, content).unwrap();
        let retrieved = cache.get_schema(key).unwrap();
        assert_eq!(retrieved, Some(content.to_vec()));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_filesystem_cache_cross_platform_paths() {
        let temp_dir = env::temp_dir().join(format!("xml_cache_test_{}", uuid::Uuid::new_v4()));
        let cache = FileSystemSchemaCache::new(temp_dir.clone()).unwrap();

        // Test that paths work correctly on all platforms
        let key = "xmlvalidator-schema/12345678/citygml/2.0/cityGMLBase";
        let path = cache.get_schema_path(key).unwrap();

        // Path should use platform-specific separators
        let path_str = path.to_str().unwrap();

        // Check that the file has .xsd extension
        assert!(path_str.ends_with(".xsd"));

        // Check that the path structure is preserved
        #[cfg(windows)]
        {
            assert!(
                path_str.contains("xmlvalidator-schema\\12345678\\citygml\\2.0\\cityGMLBase.xsd")
            );
        }
        #[cfg(not(windows))]
        {
            assert!(path_str.contains("xmlvalidator-schema/12345678/citygml/2.0/cityGMLBase.xsd"));
        }

        // Test that we can actually create and read from such paths
        let content = b"test schema content";
        cache.put_schema(key, content).unwrap();

        // Verify the file was created with correct structure
        assert!(path.exists());
        assert!(path.parent().unwrap().exists()); // 2.0 directory
        assert!(path.parent().unwrap().parent().unwrap().exists()); // citygml directory

        let retrieved = cache.get_schema(key).unwrap();
        assert_eq!(retrieved, Some(content.to_vec()));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_noop_schema_cache() {
        let cache = NoOpSchemaCache;

        // Test all operations
        assert!(cache.put_schema("test", b"content").is_ok());
        assert_eq!(cache.get_schema("test").unwrap(), None);
        assert!(!cache.is_available());
        assert!(!cache.is_cached("test"));
        assert!(cache.get_schema_path("test").is_err());
    }

    #[test]
    fn test_schema_cache_hit() {
        // Test that schemas are cached and reused properly
        let temp_dir = env::temp_dir().join(format!("xml_cache_hit_test_{}", uuid::Uuid::new_v4()));
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        let schema_key = "https://example.com/cached.xsd";
        let schema_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="test" type="xs:string"/>
</xs:schema>"#;

        // First access - cache miss
        assert!(!cache.is_cached(schema_key));

        // Put schema in cache
        cache
            .put_schema(schema_key, schema_content.as_bytes())
            .unwrap();

        // Second access - cache hit
        assert!(cache.is_cached(schema_key));

        // Verify content is retrievable
        let retrieved = cache.get_schema(schema_key).unwrap();
        assert_eq!(retrieved, Some(schema_content.as_bytes().to_vec()));

        // Get path should work
        let path = cache.get_schema_path(schema_key).unwrap();
        assert!(path.exists());

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_cache_directory_structure_preservation() {
        // Test that cache preserves directory structure for schema dependencies
        let temp_dir = env::temp_dir().join(format!("xml_cache_dir_test_{}", uuid::Uuid::new_v4()));
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        // Test with nested path
        let schema_key = "example.com/schemas/dep/dependency.xsd";
        let schema_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="item" type="xs:string"/>
</xs:schema>"#;

        cache
            .put_schema(schema_key, schema_content.as_bytes())
            .unwrap();

        // Verify directory structure is preserved
        let path = cache.get_schema_path(schema_key).unwrap();
        assert!(path.exists());
        assert!(path.to_str().unwrap().contains("schemas"));
        assert!(path.to_str().unwrap().contains("dep"));

        // Verify parent directories exist
        assert!(path.parent().unwrap().exists());
        assert!(path.parent().unwrap().parent().unwrap().exists());

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_multiple_schema_caching() {
        // Test caching multiple schemas with different paths
        let temp_dir =
            env::temp_dir().join(format!("xml_multi_cache_test_{}", uuid::Uuid::new_v4()));
        let cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        let schemas = HashMap::from([
            (
                "schema1.xsd",
                "<xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\"/>",
            ),
            (
                "path/to/schema2.xsd",
                "<xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\"/>",
            ),
            (
                "deep/path/to/schema3.xsd",
                "<xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\"/>",
            ),
        ]);

        // Cache all schemas
        for (key, content) in &schemas {
            cache.put_schema(key, content.as_bytes()).unwrap();
        }

        // Verify all are cached
        for key in schemas.keys() {
            assert!(cache.is_cached(key));
            assert!(cache.get_schema_path(key).unwrap().exists());
        }

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
