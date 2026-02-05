//! Executor-specific cache directory management.
//!
//! Each workflow execution gets its own unique cache directory under
//! `<temp_dir>/reearth-flow-engine-cache/{executor_id}/` to prevent
//! collisions between concurrent workflow executions.

use std::path::PathBuf;

use uuid::Uuid;

/// Returns the base cache directory for a specific executor.
///
/// Path: `<temp_dir>/reearth-flow-engine-cache/{executor_id}/`
pub fn executor_cache_dir(executor_id: Uuid) -> PathBuf {
    std::env::temp_dir()
        .join("reearth-flow-engine-cache")
        .join(executor_id.to_string())
}

/// Returns a subdirectory within the executor-specific cache.
///
/// # Arguments
/// * `executor_id` - Unique identifier for this workflow execution
/// * `subdir` - Name of the subdirectory (e.g., "channel-buffers", "sorter")
pub fn executor_cache_subdir(executor_id: Uuid, subdir: &str) -> PathBuf {
    executor_cache_dir(executor_id).join(subdir)
}

/// Cleans up the executor-specific cache directory.
/// Should be called at the end of workflow execution.
pub fn cleanup_executor_cache(executor_id: Uuid) {
    let cache_dir = executor_cache_dir(executor_id);
    if cache_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&cache_dir) {
            tracing::warn!(
                "Failed to cleanup executor cache directory {:?}: {}",
                cache_dir,
                e
            );
        } else {
            tracing::info!("Cleaned up executor cache directory {:?}", cache_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_cache_dir() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let dir1 = executor_cache_dir(id1);
        let dir2 = executor_cache_dir(id2);
        // Different executor IDs should produce different directories
        assert_ne!(dir1, dir2);
        // Same executor ID should produce same directory
        assert_eq!(executor_cache_dir(id1), executor_cache_dir(id1));
    }

    #[test]
    fn test_executor_cache_subdir() {
        let id = Uuid::new_v4();
        let subdir = executor_cache_subdir(id, "test-subdir");
        assert!(subdir.ends_with("test-subdir"));
        assert!(subdir
            .to_string_lossy()
            .contains("reearth-flow-engine-cache"));
        assert!(subdir.to_string_lossy().contains(&id.to_string()));
    }
}
