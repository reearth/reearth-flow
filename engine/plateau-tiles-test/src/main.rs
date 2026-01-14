mod align_cesium;
mod align_mvt;
mod cast_config;
mod compare_attributes;
mod runner;
mod test_cesium_attributes;
mod test_cesium_statistics;
mod test_json_attributes;
mod test_mvt_attributes;
mod test_mvt_lines;
mod test_mvt_points;
mod test_mvt_polygons;

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use test_cesium_attributes::CesiumAttributesConfig;
use test_cesium_statistics::CesiumStatisticsConfig;
use test_json_attributes::JsonFileConfig;
use test_mvt_attributes::MvtAttributesConfig;
use test_mvt_lines::MvtLinesConfig;
use test_mvt_points::MvtPointsConfig;
use test_mvt_polygons::MvtPolygonsConfig;
use tracing::info;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

static INIT: Once = Once::new();

fn init_logging() {
    INIT.call_once(|| {
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::EnvFilter;
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,plateau_tiles_test=debug"));

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().with_timer(
                tracing_subscriber::fmt::time::ChronoLocal::new("%H:%M:%S".to_string()),
            ))
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
    mvt_lines: Option<MvtLinesConfig>,
    #[serde(default)]
    mvt_points: Option<MvtPointsConfig>,
    #[serde(default)]
    cesium_attributes: Option<CesiumAttributesConfig>,
    #[serde(default)]
    cesium_statistics: Option<CesiumStatisticsConfig>,
}

fn zip_dir(src_dir: &Path, zip_path: &Path) {
    let file = fs::File::create(zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(src_dir).unwrap();
            let zip_path = relative_path.to_string_lossy().to_string();
            zip.start_file(zip_path, options).unwrap();
            let content = fs::read(path).unwrap();
            std::io::Write::write_all(&mut zip, &content).unwrap();
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
    "data-convert/plateau4/03-frn-veg/curvemembers",
    "data-convert/plateau4/03-frn-veg/frn",
    "data-convert/plateau4/03-frn-veg/veg",
    "data-convert/plateau4/04-luse-lsld/luse",
    "data-convert/plateau4/04-luse-lsld/lsld",
    "data-convert/plateau4/06-area-urf/urf",
    "data-convert/plateau4/06-area-urf/nested",
    "data-convert/plateau4/06-area-urf/area",
    "data-convert/plateau4/07-brid-tun-cons/brid",
    "data-convert/plateau4/07-brid-tun-cons/brid_dm_geometric_attributes",
    "data-convert/plateau4/07-brid-tun-cons/cons",
    "data-convert/plateau4/10-wtr/lod1",
];

fn run_test<F>(test_name: &str, relative_path: &std::path::Display, test_fn: F)
where
    F: FnOnce() -> Result<(), String>,
{
    info!("Starting test: {}/{}", relative_path, test_name);
    let start_time = std::time::Instant::now();

    if let Err(e) = test_fn() {
        panic!("Test failed: {}/{} - {}", relative_path, test_name, e);
    }

    let elapsed = start_time.elapsed();
    info!(
        "Completed test: {}/{} ({:.2}s)",
        relative_path,
        test_name,
        elapsed.as_secs_f64()
    );
}

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

    if stages.contains('r') {
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();

        tracing::debug!("packing citymodel zip...");
        let zip_stem = profile
            .citygml_zip_name
            .strip_suffix(".zip")
            .unwrap_or(&profile.citygml_zip_name);

        // Create citymodel zip (zip only the udx subdirectory)
        let citymodel_udx_dir = test_path.join("citymodel/udx");
        assert!(citymodel_udx_dir.exists());
        let citymodel_path = output_dir.join(zip_stem.to_string() + ".zip");
        zip_dir(&citymodel_udx_dir, &citymodel_path);

        // Create codelists zip if directory exists (symlinked from artifacts)
        let codelist_dir = test_path.join("citymodel/codelists");
        let codelist_path = codelist_dir.exists().then(|| {
            let path = output_dir.join(format!("{}_codelists.zip", zip_stem));
            zip_dir(&codelist_dir, &path);
            path
        });

        // Create schemas zip if directory exists (symlinked from artifacts)
        let schemas_dir = test_path.join("citymodel/schemas");
        let schemas_path = schemas_dir.exists().then(|| {
            let path = output_dir.join(format!("{}_schemas.zip", zip_stem));
            zip_dir(&schemas_dir, &path);
            path
        });

        info!(
            "Starting run: {} to {}",
            relative_path.display(),
            output_dir.display()
        );
        let start_time = std::time::Instant::now();

        runner::run_workflow(
            &workflow_path,
            &citymodel_path,
            &output_dir,
            codelist_path.as_deref(),
            schemas_path.as_deref(),
        );

        let elapsed = start_time.elapsed();
        info!(
            "Completed run: {} ({:.2}s)",
            relative_path.display(),
            elapsed.as_secs_f64()
        );
    }

    if stages.contains('e') {
        // Extract FME zip files from testcase to output_dir/fme_extracted
        let fme_source_dir = test_path.join("fme");
        let fme_extracted_dir = output_dir.join("fme_extracted");
        extract_toplevel_zips(&fme_source_dir, &fme_extracted_dir);

        // Extract Flow output zip files to output_dir/flow_extracted
        let flow_source_dir = output_dir.join("flow");
        let flow_extracted_dir = output_dir.join("flow_extracted");
        extract_toplevel_zips(&flow_source_dir, &flow_extracted_dir);

        // Decompress draco-compressed glb in flow output
        // fme.zip should be preprocessed to contain only decompressed glb files
        decompress_glbs(&flow_extracted_dir);

        let tests = &profile.tests;
        let relative_path_display = relative_path.display();

        if let Some(cfg) = &tests.json_attributes {
            run_test("json_attributes", &relative_path_display, || {
                test_json_attributes::test_json_attributes(
                    &fme_extracted_dir,
                    &flow_extracted_dir,
                    cfg,
                )
            });
        }

        if let Some(cfg) = &tests.mvt_attributes {
            run_test("mvt_attributes", &relative_path_display, || {
                test_mvt_attributes::test_mvt_attributes(
                    &fme_extracted_dir,
                    &flow_extracted_dir,
                    cfg,
                )
            });
        }

        if let Some(cfg) = &tests.mvt_polygons {
            run_test("mvt_polygons", &relative_path_display, || {
                test_mvt_polygons::test_mvt_polygons(&fme_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.mvt_lines {
            run_test("mvt_lines", &relative_path_display, || {
                test_mvt_lines::test_mvt_lines(&fme_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.mvt_points {
            run_test("mvt_points", &relative_path_display, || {
                test_mvt_points::test_mvt_points(&fme_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.cesium_attributes {
            run_test("cesium_attributes", &relative_path_display, || {
                test_cesium_attributes::test_cesium_attributes(
                    &fme_extracted_dir,
                    &flow_extracted_dir,
                    cfg,
                )
            });
        }

        if let Some(cfg) = &tests.cesium_statistics {
            run_test("cesium_statistics", &relative_path_display, || {
                test_cesium_statistics::test_cesium_statistics(
                    &fme_extracted_dir,
                    &flow_extracted_dir,
                    cfg,
                )
            });
        }
    }

    if let Some("1") = env::var("PLATEAU_TILES_TEST_CLEANUP").ok().as_deref() {
        info!("Cleaning up output directory: {}", output_dir.display());
        fs::remove_dir_all(&output_dir).unwrap();
    }
}

fn extract_toplevel_zips(source_dir: &Path, output_dir: &Path) {
    if !source_dir.exists() {
        return;
    }
    fs::create_dir_all(output_dir).unwrap();

    for entry in fs::read_dir(source_dir).unwrap().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "zip") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let out = output_dir.join(stem);
            let _ = fs::remove_dir_all(&out);
            fs::create_dir_all(&out).unwrap();
            let mut zip = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
            zip.extract(&out).unwrap();
        }
    }
}

fn decompress_glbs(flow_extracted_dir: &Path) {
    for entry in WalkDir::new(flow_extracted_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "glb") {
            tracing::debug!("Decompressing glb file: {}", path.display());
            let status = std::process::Command::new("npx")
                .arg("glb-decompress")
                .arg(path.as_os_str())
                .status()
                .expect("Failed to execute glb-decompress command");
            if !status.success() {
                panic!("glb-decompress failed");
            }
        }
    }
}

fn main() {
    init_logging();

    // Set to 0ms for local test runs - we don't need event propagation delay
    // since we're not sending events to external systems (GCP Pub/Sub, etc.)
    env::set_var("FLOW_RUNTIME_NODE_STATUS_PROPAGATION_DELAY_MS", "0");

    let testcases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testcases");
    let results_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("results");

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let input = &args[1];
        let stages = if args.len() > 2 { &args[2] } else { "re" };

        // Check if input is a profile.toml path
        let test_name = if input.ends_with("profile.toml") {
            let profile_path = fs::canonicalize(PathBuf::from(input)).unwrap();
            let test_dir = profile_path.parent().unwrap();
            let relative = test_dir.strip_prefix(&testcases_dir).unwrap();
            relative.to_string_lossy().to_string()
        } else {
            input.to_string()
        };
        eprintln!("test_name: {}", test_name);

        run_testcase(&testcases_dir, &results_dir, &test_name, stages);
    } else {
        for name in DEFAULT_TESTS {
            run_testcase(&testcases_dir, &results_dir, name, "re");
        }
    }
}
