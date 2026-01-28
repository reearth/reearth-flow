use anyhow::{Context, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Include shared type definitions
include!("shared_types.rs");

struct TestCase {
    name: String,
    path: PathBuf,
    profile: WorkflowTestProfile,
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shared_types.rs");
    println!("cargo:rerun-if-changed=../fixture/testdata");

    let out_dir = env::var("OUT_DIR").context("OUT_DIR not set")?;
    let out_path = Path::new(&out_dir).join("generated_tests.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?;
    let testdata_dir = Path::new(&manifest_dir).parent().unwrap().join("testdata");

    let test_cases = discover_tests(&testdata_dir)?;
    let generated_code = generate_test_code(&test_cases, &testdata_dir)?;

    fs::write(&out_path, generated_code.to_string())?;

    Ok(())
}

fn discover_tests(testdata_dir: &Path) -> Result<Vec<TestCase>> {
    let mut test_cases = Vec::new();

    if !testdata_dir.exists() {
        return Err(anyhow::anyhow!(
            "Test data directory does not exist: {testdata_dir:?}"
        ));
    }

    for entry in WalkDir::new(testdata_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "workflow_test.json" {
            let test_dir = entry.path().parent().unwrap();
            let test_name = test_dir
                .strip_prefix(testdata_dir)
                .unwrap()
                .to_string_lossy()
                .replace(['/', '-'], "_")
                .to_lowercase();

            let profile_str = fs::read_to_string(entry.path())?;
            let profile: WorkflowTestProfile = serde_json::from_str(&profile_str)?;

            test_cases.push(TestCase {
                name: test_name,
                path: test_dir.to_path_buf(),
                profile,
            });
        }
    }

    test_cases.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(test_cases)
}

fn generate_test_code(test_cases: &[TestCase], testdata_dir: &Path) -> Result<TokenStream> {
    let mut test_functions = Vec::new();

    for test_case in test_cases {
        let test_name = format_ident!("test_{}", test_case.name);
        let test_name_str = test_case.name.clone();
        let test_dir_str = test_case.path.to_string_lossy().to_string();
        let fixture_dir_str = testdata_dir.parent().unwrap().to_string_lossy().to_string();

        let skip_attr = if test_case.profile.skip {
            let reason = test_case
                .profile
                .skip_reason
                .as_deref()
                .unwrap_or("Skipped");
            quote! { #[ignore = #reason] }
        } else {
            quote! {}
        };

        let _description = test_case.profile.description.as_deref().unwrap_or("");

        test_functions.push(quote! {
            #[test]
            #skip_attr
            fn #test_name() -> Result<()> {
                // Load test profile
                let profile_path = PathBuf::from(#test_dir_str).join("workflow_test.json");
                let profile_str = fs::read_to_string(&profile_path)?;
                let profile: WorkflowTestProfile = serde_json::from_str(&profile_str)?;

                // Create test context
                let mut ctx = TestContext::new(
                    #test_name_str.to_string(),
                    PathBuf::from(#test_dir_str),
                    PathBuf::from(#fixture_dir_str),
                    profile,
                )?;

                // Run test and ensure cleanup happens even on failure
                let result = (|| -> Result<()> {
                    let workflow = ctx.load_workflow()?;
                    ctx.run_workflow(workflow)?;
                    ctx.verify_output()?;
                    ctx.verify_intermediate_data()?;
                    ctx.verify_summary_output()?;
                    ctx.verify_no_unexpected_output_files()?;
                    ctx.verify_result_ok_file()?;
                    Ok(())
                })();

                // Clean up generated ZIP files
                let cleanup_result = ctx.cleanup_generated_zips();

                // Return the test result (cleanup errors are logged but don't fail the test)
                if let Err(e) = cleanup_result {
                    tracing::warn!("Failed to cleanup generated zips: {}", e);
                }

                result
            }
        });
    }

    Ok(quote! {
        #[cfg(test)]
        mod generated_tests {
            use super::*;
            use std::path::PathBuf;

            #(#test_functions)*
        }
    })
}
