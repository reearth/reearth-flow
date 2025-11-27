//! Feature size estimation utilities.
//!
//! Since the Feature type has many dependencies that don't implement DataSize,
//! we use JSON serialization to estimate the size. This is not perfectly accurate
//! but provides a reasonable approximation.

use serde::Serialize;

/// Estimate the size of a serializable value in bytes.
///
/// This uses JSON serialization to estimate the size. The actual in-memory
/// size may be different due to:
/// - JSON overhead (keys, quotes, etc.)
/// - Compression in actual memory representation
/// - Shared references and interning
///
/// However, this provides a consistent and useful approximation for
/// comparing relative sizes of features.
pub fn estimate_size<T: Serialize>(value: &T) -> usize {
    serde_json::to_string(value).map(|s| s.len()).unwrap_or(0)
}

/// Estimate the size of a value by serializing to msgpack (more compact).
/// Falls back to JSON if msgpack fails.
pub fn estimate_size_compact<T: Serialize>(value: &T) -> usize {
    // Try to estimate using a more compact representation
    // For now, just use JSON length as it's simple and consistent
    estimate_size(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_estimate_size_simple() {
        let value = json!({"name": "test", "value": 42});
        let size = estimate_size(&value);
        assert!(size > 0);
        assert!(size < 100);
    }

    #[test]
    fn test_estimate_size_array() {
        let small = json!([1, 2, 3]);
        let large = json!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let small_size = estimate_size(&small);
        let large_size = estimate_size(&large);

        assert!(large_size > small_size);
    }

    #[test]
    fn test_estimate_size_nested() {
        let value = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "data": "deeply nested value"
                    }
                }
            }
        });

        let size = estimate_size(&value);
        assert!(size > 50);
    }
}
