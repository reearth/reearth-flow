use plateau_tiles_test::conv::mvt;
use plateau_tiles_test::conv::mvt_png;
use plateau_tiles_test::file::{extract_dir, zip_dir};
use plateau_tiles_test::profile_config::Convs;
use plateau_tiles_test::runner;
use plateau_tiles_test::tester::cesium::{self, CesiumConfig};
use plateau_tiles_test::tester::json_attributes::{self, JsonFileConfig};
use plateau_tiles_test::tester::json_attributes_v2::{self, JsonFileV2Config};
use plateau_tiles_test::tester::json_object_key_order::{self, KeyOrderConfig};
use plateau_tiles_test::tester::mvt_lines::{self, MvtLinesConfig};
use plateau_tiles_test::tester::mvt_points::{self, MvtPointsConfig};
use plateau_tiles_test::tester::mvt_polygons::{self, MvtPolygonsConfig};
use plateau_tiles_test::tester::raster::{self, RasterConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use walkdir::WalkDir;

static INIT: Once = Once::new();

fn init_logging() {
    INIT.call_once(|| {
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
    #[serde(default)]
    convs: Convs,
}

#[derive(Debug, Deserialize, Default)]
struct Tests {
    #[serde(default)]
    json_attributes: Option<HashMap<String, JsonFileConfig>>,
    #[serde(default)]
    json_attributes_v2: Option<HashMap<String, JsonFileV2Config>>,
    #[serde(default)]
    mvt_polygons: Option<MvtPolygonsConfig>,
    #[serde(default)]
    mvt_lines: Option<MvtLinesConfig>,
    #[serde(default)]
    mvt_points: Option<MvtPointsConfig>,
    #[serde(default)]
    cesium: Option<CesiumConfig>,
    #[serde(default)]
    json_object_key_order: Option<KeyOrderConfig>,
    #[serde(default)]
    raster: Option<HashMap<String, RasterConfig>>,
}

fn pack_inputs(
    test_path: &Path,
    output_dir: &Path,
    zip_stem: &str,
) -> HashMap<&'static str, PathBuf> {
    tracing::debug!("packing citymodel zip...");

    let citymodel_udx_dir = test_path.join("citymodel/udx");
    assert!(citymodel_udx_dir.exists());
    let citymodel = output_dir.join(format!("{}.zip", zip_stem));
    zip_dir(&citymodel_udx_dir, &citymodel).unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("citymodel", citymodel);

    let codelists_dir = test_path.join("citymodel/codelists");
    if codelists_dir.exists() {
        let path = output_dir.join(format!("{}_codelists.zip", zip_stem));
        zip_dir(&codelists_dir, &path).unwrap();
        inputs.insert("codelists", path);
    }

    let schemas_dir = test_path.join("citymodel/schemas");
    if schemas_dir.exists() {
        let path = output_dir.join(format!("{}_schemas.zip", zip_stem));
        zip_dir(&schemas_dir, &path).unwrap();
        inputs.insert("schemas", path);
    }

    inputs
}

fn direct_inputs(test_path: &Path) -> HashMap<&'static str, PathBuf> {
    let citymodel = test_path.join("citymodel");
    assert!(
        citymodel.exists(),
        "citymodel dir not found: {}",
        citymodel.display()
    );
    let mut inputs = HashMap::new();
    inputs.insert("citymodel", citymodel);
    inputs
}

const DEFAULT_TESTS: &[&str] = &[
    "data-convert/plateau4/01-bldg/lod1",
    "data-convert/plateau4/01-bldg/tako-machi",
    "data-convert/plateau4/01-bldg/ogasawara-mura",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/multipolygon",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/squr",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/squr_xlink",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/dm",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/rwy",
    "data-convert/plateau4/02-tran-rwy-trk-squr-wwy/wwy",
    "data-convert/plateau4/03-frn-veg/curvemembers",
    "data-convert/plateau4/03-frn-veg/frn",
    "data-convert/plateau4/03-frn-veg/veg",
    "data-convert/plateau4/04-luse-lsld/luse",
    "data-convert/plateau4/04-luse-lsld/lsld",
    "data-convert/plateau4/05-fld/fld",
    "data-convert/plateau4/05-fld/tnm",
    "data-convert/plateau4/05-fld/htd",
    "data-convert/plateau4/05-fld/ifld",
    "data-convert/plateau4/05-fld/rfld",
    "data-convert/plateau4/06-area-urf/urf",
    "data-convert/plateau4/06-area-urf/nested",
    "data-convert/plateau4/06-area-urf/area",
    "data-convert/plateau4/07-brid-tun-cons/brid",
    "data-convert/plateau4/07-brid-tun-cons/brid_dm_geometric_attributes",
    "data-convert/plateau4/07-brid-tun-cons/tun",
    "data-convert/plateau4/07-brid-tun-cons/cons",
    "data-convert/plateau4/08-ubld/ubld",
    "data-convert/plateau4/09-unf/frn_lod3",
    "data-convert/plateau4/09-unf/unf",
    "data-convert/plateau4/10-wtr/wtr",
    "data-convert/plateau4/11-gen/mvt",
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
            .parent()
            .unwrap()
            .join(wp)
    } else {
        let workflow_parts: Vec<_> = relative_path.iter().collect();
        let workflow_parts = &workflow_parts[..workflow_parts.len() - 1];
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("runtime/examples/fixture/workflow")
            .join(workflow_parts.iter().collect::<PathBuf>())
            .join("workflow.yml")
    };

    if stages.contains('r') {
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).unwrap();

        // do not pack gml, codelists, schemas into zip if PLATEAU_TILES_TEST_NO_PACK=1 for quick local tests
        let no_pack = env::var("PLATEAU_TILES_TEST_NO_PACK").ok().as_deref() == Some("1");
        let inputs = if no_pack {
            direct_inputs(&test_path)
        } else {
            let zip_stem = profile
                .citygml_zip_name
                .strip_suffix(".zip")
                .unwrap_or(&profile.citygml_zip_name);
            pack_inputs(&test_path, &output_dir, zip_stem)
        };

        info!(
            "Starting run: {} to {}",
            relative_path.display(),
            output_dir.display()
        );
        let start_time = std::time::Instant::now();

        let zip_stem = profile
            .citygml_zip_name
            .strip_suffix(".zip")
            .unwrap_or(&profile.citygml_zip_name);
        let target_package = zip_stem
            .find("_op_")
            .map(|pos| zip_stem[pos + 4..].to_string());

        runner::run_workflow(
            &workflow_path,
            &inputs["citymodel"],
            &output_dir,
            inputs.get("codelists").map(PathBuf::as_path),
            inputs.get("schemas").map(PathBuf::as_path),
            target_package.as_deref(),
        );

        let elapsed = start_time.elapsed();
        info!(
            "Completed run: {} ({:.2}s)",
            relative_path.display(),
            elapsed.as_secs_f64()
        );
    }

    if stages.contains('e') {
        // Extract FME zip files and copy other items from testcase to output_dir/fme_extracted
        let fme_source_dir = test_path.join("fme");
        let fme_extracted_dir = output_dir.join("fme_extracted");
        extract_dir(&fme_source_dir, &fme_extracted_dir);

        // Extract Flow output zip files to output_dir/flow_extracted
        let flow_source_dir = output_dir.join("flow");
        let flow_extracted_dir = output_dir.join("flow_extracted");
        extract_dir(&flow_source_dir, &flow_extracted_dir);

        // Decompress draco-compressed glb in flow output
        // fme.zip should be preprocessed to contain only decompressed glb files
        decompress_glbs(&flow_extracted_dir);

        let tests = &profile.tests;
        let relative_path_display = relative_path.display();

        if let Some(cfg) = &tests.json_attributes {
            run_test("json_attributes", &relative_path_display, || {
                json_attributes::test_json_attributes(
                    &fme_source_dir,
                    &flow_source_dir,
                    &fme_extracted_dir,
                    &flow_extracted_dir,
                    cfg,
                )
            });
        }

        if !profile.convs.json.is_empty() {
            run_test("convs_json", &relative_path_display, || {
                for entry in profile.convs.json.values() {
                    let flow_file = output_dir.join("flow_extracted").join(&entry.flow_path);
                    let output_path = output_dir.join("flow_extracted").join(&entry.output_path);
                    plateau_tiles_test::conv::json::write_json(
                        &flow_file,
                        &output_path,
                        entry.json_path.as_deref(),
                        &entry.casts,
                    )?;
                }
                Ok(())
            });
        }

        if !profile.convs.mvt_attributes.is_empty() {
            run_test("convs_mvt_attributes", &relative_path_display, || {
                for entry in profile.convs.mvt_attributes.values() {
                    let mvt_dir = output_dir.join("flow_extracted").join(&entry.path);
                    let output_path = output_dir.join("flow_extracted").join(&entry.truth_path);
                    mvt::write_mvt_json(&mvt_dir, &output_path, entry.casts.as_ref())?;
                }
                Ok(())
            });
        }

        if !profile.convs.mvt_png.is_empty() {
            run_test("convs_mvt_png", &relative_path_display, || {
                for entry in profile.convs.mvt_png.values() {
                    let mvt_dir = output_dir.join("flow_extracted").join(&entry.path);
                    let png_dir = output_dir.join("flow_extracted").join(&entry.truth_path);
                    let (w, h) = entry.size.dimensions();
                    mvt_png::write_png_truth(
                        &mvt_dir,
                        &png_dir,
                        entry.tiles.as_deref(),
                        w,
                        h,
                        entry.stroke,
                    )?;
                }
                Ok(())
            });
        }

        if let Some(cfg) = &tests.json_attributes_v2 {
            run_test("json_attributes_v2", &relative_path_display, || {
                json_attributes_v2::test_json_attributes_v2(&output_dir, &test_path, cfg)
            });
        }

        if let Some(cfg) = &tests.mvt_polygons {
            run_test("mvt_polygons", &relative_path_display, || {
                mvt_polygons::test_mvt_polygons(&fme_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.mvt_lines {
            run_test("mvt_lines", &relative_path_display, || {
                mvt_lines::test_mvt_lines(&fme_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.mvt_points {
            run_test("mvt_points", &relative_path_display, || {
                mvt_points::test_mvt_points(&fme_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.cesium {
            run_test("cesium", &relative_path_display, || {
                cesium::test_cesium(&fme_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(raster_tests) = &tests.raster {
            for (id, cfg) in raster_tests {
                let conv_entry = profile.convs.mvt_png.get(id).unwrap_or_else(|| {
                    panic!(
                        "tests.raster.{} references missing convs.mvt_png.{}",
                        id, id
                    )
                });
                let flow_png_dir = output_dir
                    .join("flow_extracted")
                    .join(&conv_entry.truth_path);
                let truth_dir = fme_extracted_dir.join(&conv_entry.truth_path);
                let id = id.clone();
                run_test(&format!("raster/{}", id), &relative_path_display, || {
                    raster::test_raster(&truth_dir, &flow_png_dir, cfg)
                });
            }
        }

        if let Some(cfg) = &tests.json_object_key_order {
            run_test("json_object_key_order", &relative_path_display, || {
                json_object_key_order::test_json_object_key_order(
                    &flow_source_dir,
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

fn decompress_glbs(flow_extracted_dir: &Path) {
    let glb_files: Vec<_> = WalkDir::new(flow_extracted_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "glb") {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
        .collect();

    let mut cmd = std::process::Command::new("glb-decompress");
    for glb_file in &glb_files {
        cmd.arg(glb_file.as_os_str());
    }

    let status = cmd
        .status()
        .expect("Failed to execute glb-decompress command");
    if !status.success() {
        panic!("glb-decompress failed");
    }
}

fn main() {
    init_logging();

    // Set to 0ms for local test runs - we don't need event propagation delay
    // since we're not sending events to external systems (GCP Pub/Sub, etc.)
    env::set_var("FLOW_RUNTIME_NODE_STATUS_PROPAGATION_DELAY_MS", "0");

    let testcases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("data/testcases");
    let results_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("data/results");

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
