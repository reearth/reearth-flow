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
        // Replace directory separators in key to avoid path traversal
        let safe_key = key.replace(['/', '\\'], "_");
        self.root_path.join(format!("{safe_key}.xsd"))
    }
}

impl SchemaCache for FileSystemSchemaCache {
    fn put_schema(&self, key: &str, content: &[u8]) -> Result<()> {
        let path = self.get_full_path(key);

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
        })
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

        // Test that directory separators are replaced
        let key_with_slashes = "path/to/schema";
        let key_with_backslashes = "path\\to\\schema";

        let path1 = cache.get_schema_path(key_with_slashes).unwrap();
        let path2 = cache.get_schema_path(key_with_backslashes).unwrap();

        // Both should result in safe filenames
        assert!(path1
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("path_to_schema"));
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
    fn test_noop_schema_cache() {
        let cache = NoOpSchemaCache;

        // Test all operations
        assert!(cache.put_schema("test", b"content").is_ok());
        assert_eq!(cache.get_schema("test").unwrap(), None);
        assert!(!cache.is_available());
        assert!(!cache.is_cached("test"));
        assert!(cache.get_schema_path("test").is_err());
    }
}
