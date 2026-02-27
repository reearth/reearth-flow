use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Include shared type definitions
include!("../shared_types.rs");

fn main() -> Result<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    // Navigate to testdata directory
    let testdata_dir = Path::new(&manifest_dir)
        .parent()
        .unwrap_or(Path::new("."))
        .join("testdata");

    if !testdata_dir.exists() {
        // Try relative path from current directory (testing/data/testcases)
        let testdata_dir = Path::new("testing/data/testcases");
        if testdata_dir.exists() {
            return normalize_all(testdata_dir);
        }
        // Legacy path fallback
        let testdata_dir = Path::new("runtime/examples/fixture/testdata");
        if testdata_dir.exists() {
            return normalize_all(testdata_dir);
        }
        anyhow::bail!(
            "Test data directory not found. Run from engine root or fixture/tests directory."
        );
    }

    normalize_all(&testdata_dir)
}

fn normalize_all(testdata_dir: &Path) -> Result<()> {
    let mut count = 0;
    let mut modified = 0;

    for entry in WalkDir::new(testdata_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "workflow_test.json" {
            count += 1;
            let path = entry.path();

            // Read and parse
            let original = fs::read_to_string(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;

            let profile: WorkflowTestProfile = serde_json::from_str(&original)
                .with_context(|| format!("Failed to parse {}", path.display()))?;

            // Serialize back with proper formatting
            let normalized = serde_json::to_string_pretty(&profile)
                .with_context(|| format!("Failed to serialize {}", path.display()))?;

            // Add trailing newline
            let normalized = format!("{}\n", normalized);

            // Only write if changed
            if original != normalized {
                fs::write(path, &normalized)
                    .with_context(|| format!("Failed to write {}", path.display()))?;
                println!("Normalized: {}", path.display());
                modified += 1;
            }
        }
    }

    println!("\nProcessed {} files, modified {} files", count, modified);
    Ok(())
}
