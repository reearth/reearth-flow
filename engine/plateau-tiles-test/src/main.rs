mod cast_config;
mod compare_attributes;
mod runner;
mod test_json_attributes;
mod test_mvt_attributes;
mod test_mvt_polygons;

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use test_json_attributes::JsonFileConfig;
use test_mvt_attributes::MvtAttributesConfig;
use test_mvt_polygons::MvtPolygonsConfig;
use tracing::info;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

static INIT: Once = Once::new();

fn init_logging(verbosity: &str) {
    INIT.call_once(|| {
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::EnvFilter;

        let filter = match verbosity {
            "0" => EnvFilter::new("warn,plateau_tiles_test=info"),
            "1" => EnvFilter::new("info,plateau_tiles_test=debug"),
            _ => EnvFilter::from_default_env(),
        };

        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new("%H:%M:%S".to_string()))
            )
            .init();
    });
}

#[derive(Debug, Deserialize)]
struct Profile {
    citygml_zip_name: String,
    workflow_path: Option<String>,
    #[serde(default, rename = "tests")]
    tests: Tests,
}

#[derive(Debug, Deserialize, Default)]
struct Tests {
    #[serde(default)]
    json_attributes: Option<HashMap<String, JsonFileConfig>>,
    #[serde(default)]
    mvt_attributes: Option<MvtAttributesConfig>,
    #[serde(default)]
    mvt_polygons: Option<MvtPolygonsConfig>,
    #[serde(default)]
    mvt_lines: Option<serde_json::Value>,
}


fn pack_citymodel_zip(zip_stem: &str, testcase_dir: &Path, artifacts_base: &Path, output_path: &Path) {
    let artifact_dir = artifacts_base.join(zip_stem);
    let testcase_citymodel = testcase_dir.join("citymodel");

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let file = fs::File::create(output_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for dirname in ["codelists", "schemas"] {
        let src = artifact_dir.join(dirname);
        if src.exists() {
            for entry in WalkDir::new(&src).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let relative_path = path.strip_prefix(&src).unwrap();
                    let zip_path = format!("{}/{}", dirname, relative_path.display());
                    zip.start_file(zip_path, options).unwrap();
                    let content = fs::read(path).unwrap();
                    std::io::Write::write_all(&mut zip, &content).unwrap();
                }
            }
        }
    }

    if testcase_citymodel.exists() {
        for entry in WalkDir::new(&testcase_citymodel)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                let relative_path = path.strip_prefix(&testcase_citymodel).unwrap();
                let zip_path = relative_path.to_string_lossy().to_string();
                zip.start_file(zip_path, options).unwrap();
                let content = fs::read(path).unwrap();
                std::io::Write::write_all(&mut zip, &content).unwrap();
            }
        }
    }

    zip.finish().unwrap();
}

const DEFAULT_TESTS: &[&str] = &[
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/multipolygon",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/squr",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/dm",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/rwy",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/wwy",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/3dtiles",
    "data-convert/plateau4/06-area-urf/urf",
    "data-convert/plateau4/06-area-urf/nested",
];

fn run_testcase(testcases_dir: &Path, results_dir: &Path, name: &str, stages: &str) {
    let test_path = testcases_dir.join(name);
    let profile_path = test_path.join("profile.toml");
    let profile_content = fs::read_to_string(&profile_path).unwrap();
    let profile: Profile = toml::from_str(&profile_content).unwrap();

    let relative_path = test_path.strip_prefix(testcases_dir).unwrap();
    let output_dir = results_dir.join(relative_path);

    let workflow_path = if let Some(ref wp) = profile.workflow_path {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(wp)
    } else {
        let workflow_parts: Vec<_> = relative_path.iter().collect();
        let workflow_parts = &workflow_parts[..workflow_parts.len() - 1];
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("runtime/examples/fixture/workflow")
            .join(workflow_parts.iter().collect::<PathBuf>())
            .join("workflow.yml")
    };

    let citygml_path = output_dir.join(&profile.citygml_zip_name);

    if stages.contains('r') {
        if !citygml_path.exists() {
            let zip_stem = profile
                .citygml_zip_name
                .strip_suffix(".zip")
                .unwrap_or(&profile.citygml_zip_name);
            let artifacts_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("artifacts")
                .join("citymodel");
            pack_citymodel_zip(zip_stem, &test_path, &artifacts_base, &citygml_path);
        }

        info!("Starting run: {}", relative_path.display());

        fs::create_dir_all(&output_dir).unwrap();

        runner::run_workflow(&workflow_path, &citygml_path, &output_dir);

        info!("Completed run: {}", relative_path.display());
    }

    if stages.contains('e') {
        let fme_output_path = test_path.join("fme.zip");
        if !fme_output_path.exists() {
            panic!("FME output file not found in testcase: {}", fme_output_path.display());
        }

        let fme_dir = output_dir.join("fme");
        extract_fme_output(&fme_output_path, &fme_dir);

        let tests = &profile.tests;

        if let Some(cfg) = &tests.json_attributes {
            let test_name = "json_attributes";
            info!("Starting test: {}/{}", relative_path.display(), test_name);
            let start_time = std::time::Instant::now();

            if let Err(e) = test_json_attributes::test_json_attributes(&fme_dir, &output_dir.join("flow"), cfg) {
                panic!("Test failed: {}/{} - {}", relative_path.display(), test_name, e);
            }

            let elapsed = start_time.elapsed();
            info!("Completed test: {}/{} ({:.2}s)", relative_path.display(), test_name, elapsed.as_secs_f64());
        }

        if let Some(cfg) = &tests.mvt_attributes {
            let test_name = "mvt_attributes";
            info!("Starting test: {}/{}", relative_path.display(), test_name);
            let start_time = std::time::Instant::now();

            if let Err(e) = test_mvt_attributes::test_mvt_attributes(&fme_dir, &output_dir.join("flow"), cfg) {
                panic!("Test failed: {}/{} - {}", relative_path.display(), test_name, e);
            }

            let elapsed = start_time.elapsed();
            info!("Completed test: {}/{} ({:.2}s)", relative_path.display(), test_name, elapsed.as_secs_f64());
        }

        if let Some(cfg) = &tests.mvt_polygons {
            let test_name = "mvt_polygons";
            info!("Starting test: {}/{}", relative_path.display(), test_name);
            let start_time = std::time::Instant::now();

            if let Err(e) = test_mvt_polygons::test_mvt_polygons(&fme_dir, &output_dir.join("flow"), cfg) {
                panic!("Test failed: {}/{} - {}", relative_path.display(), test_name, e);
            }

            let elapsed = start_time.elapsed();
            info!("Completed test: {}/{} ({:.2}s)", relative_path.display(), test_name, elapsed.as_secs_f64());
        }

        if tests.mvt_lines.is_some() {
            info!("Skipping unsupported test type: mvt_lines");
        }
    }
}

fn extract_fme_output(fme_zip_path: &Path, fme_dir: &Path) {
    if let Some(parent) = fme_dir.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    // Check if we need to extract
    let mut needs_extract = true;
    if fme_dir.exists() {
        let fme_zip_mtime = fs::metadata(fme_zip_path).unwrap().modified().unwrap();
        let mut fme_dir_mtime = None;

        for entry in WalkDir::new(fme_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let mtime = fs::metadata(entry.path()).unwrap().modified().unwrap();
                if fme_dir_mtime.is_none() || mtime > fme_dir_mtime.unwrap() {
                    fme_dir_mtime = Some(mtime);
                }
            }
        }

        if let Some(dir_mtime) = fme_dir_mtime {
            if dir_mtime >= fme_zip_mtime {
                needs_extract = false;
            }
        }
    }

    if needs_extract {
        if fme_dir.exists() {
            fs::remove_dir_all(fme_dir).unwrap();
        }
        fs::create_dir_all(fme_dir).unwrap();

        tracing::debug!("Extracting FME output: {} -> {}", fme_zip_path.display(), fme_dir.display());

        let file = fs::File::open(fme_zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = fme_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }
    }
}

fn main() {
    let verbosity = env::var("PLATEAU_TILES_TEST_LOG_VERBOSITY").unwrap_or_else(|_| "1".to_string());
    init_logging(&verbosity);

    let testcases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testcases");
    let results_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("results");

    let args: Vec<String> = env::args().collect();
    let default_stages = env::var("PLATEAU_TILES_TEST_STAGES").unwrap_or_else(|_| "re".to_string());

    if args.len() > 1 {
        let name = &args[1];
        let stages = if args.len() > 2 {
            &args[2]
        } else {
            &default_stages
        };
        run_testcase(&testcases_dir, &results_dir, name, stages);
    } else {
        for name in DEFAULT_TESTS {
            run_testcase(&testcases_dir, &results_dir, name, &default_stages);
        }
    }
}
