use plateau_tiles_test::conv::cesium as conv_cesium;
use plateau_tiles_test::conv::mvt;
use plateau_tiles_test::conv::mvt_png;
use plateau_tiles_test::conv::raster3d as conv_raster3d;
use plateau_tiles_test::file::{decompress_glbs, extract_dir, zip_dir};
use plateau_tiles_test::profile_config::Convs;
use plateau_tiles_test::runner;
use plateau_tiles_test::tester::cesium::{self, CesiumConfig};
use plateau_tiles_test::tester::cesium_statistics;
use plateau_tiles_test::tester::json_attributes::{self, JsonFileConfig};
use plateau_tiles_test::tester::json_attributes_v2::{self, JsonFileV2Config};
use plateau_tiles_test::tester::json_object_key_order::{self, KeyOrderConfig};
use plateau_tiles_test::tester::mvt_lines::{self, MvtLinesConfig};
use plateau_tiles_test::tester::mvt_points::{self, MvtPointsConfig};
use plateau_tiles_test::tester::mvt_polygons::{self, MvtPolygonsConfig};
use plateau_tiles_test::tester::output_files::{self, OutputFilesConfig};
use plateau_tiles_test::tester::raster::{self, RasterConfig};
use plateau_tiles_test::tester::raster3d::{self, Raster3dConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

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
    #[serde(default)]
    raster3d: Option<HashMap<String, Raster3dConfig>>,
    #[serde(default)]
    output_files: Option<HashMap<String, OutputFilesConfig>>,
}

fn pack_inputs(
    test_path: &Path,
    output_dir: &Path,
    zip_stem: &str,
) -> HashMap<&'static str, PathBuf> {
    tracing::debug!("packing citymodel zip...");

    // Pack the whole citymodel (udx + codelists + schemas) into one archive so
    // that, once extracted, each gml keeps codelists/schemas as siblings of its
    // `udx` dir. The new CityGML reader resolves `codeSpace` relative to the gml,
    // so the co-located layout is required; splitting codelists into a separate
    // zip would break relative resolution.
    let citymodel_dir = test_path.join(zip_stem);
    assert!(citymodel_dir.join("udx").exists());
    let citymodel = output_dir.join(format!("{}.zip", zip_stem));
    zip_dir(&citymodel_dir, &citymodel).unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("citymodel", citymodel);
    inputs
}

fn direct_inputs(test_path: &Path, zip_stem: &str) -> HashMap<&'static str, PathBuf> {
    let citymodel = test_path.join(zip_stem);
    assert!(
        citymodel.exists(),
        "citymodel dir not found: {}",
        citymodel.display()
    );
    let mut inputs = HashMap::new();
    inputs.insert("citymodel", citymodel);
    inputs
}

#[cfg(not(feature = "new-geometry"))]
const DEFAULT_TESTS: &[&str] = &[
    "data-convert/plateau4/01-bldg/fld",
    "data-convert/plateau4/01-bldg/tako-machi",
    "data-convert/plateau4/01-bldg/ogasawara-mura",
    "data-convert/plateau4/01-bldg/ward",
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
    "examples/citygml-roundtrip/tun",
];

#[cfg(feature = "new-geometry")]
const DEFAULT_TESTS: &[&str] = &[
    "data-convert/plateau6/01-bldg/ward",
    "data-convert/plateau6/01-bldg/osaka-ward",
    "data-convert/plateau6/02-tran-rwy-trk-squr-wwy/multipolygon",
    "data-convert/plateau6/02-tran-rwy-trk-squr-wwy/dm",
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

        // Feed the citymodel directory directly by default. The new CityGML reader
        // resolves `codeSpace` relative to each gml's own location, so codelists and
        // schemas must sit alongside the gml; the real directory already has that
        // layout, whereas packing splits them into separate zips. Opt into packing
        // with PLATEAU_TILES_TEST_PACK=1 to exercise the archive-extraction path.
        let zip_stem = profile
            .citygml_zip_name
            .strip_suffix(".zip")
            .unwrap_or(&profile.citygml_zip_name);
        let pack = env::var("PLATEAU_TILES_TEST_PACK").ok().as_deref() == Some("1");
        let inputs = if pack {
            pack_inputs(&test_path, &output_dir, zip_stem)
        } else {
            direct_inputs(&test_path, zip_stem)
        };

        info!(
            "Starting run: {} to {}",
            relative_path.display(),
            output_dir.display()
        );
        let start_time = std::time::Instant::now();

        let target_package = zip_stem
            .find("_op_")
            .map(|pos| zip_stem[pos + 4..].to_string());

        if let Err(e) = runner::run_workflow(
            &workflow_path,
            &inputs["citymodel"],
            &output_dir,
            inputs.get("codelists").map(PathBuf::as_path),
            inputs.get("schemas").map(PathBuf::as_path),
            target_package.as_deref(),
        ) {
            panic!("Run failed: {} - {}", relative_path.display(), e);
        }

        let elapsed = start_time.elapsed();
        info!(
            "Completed run: {} ({:.2}s)",
            relative_path.display(),
            elapsed.as_secs_f64()
        );
    }

    if stages.contains('e') {
        // Extract truth zip files and copy other items from testcase to output_dir/truth_extracted
        let truth_dir = test_path.join("truth");
        let truth_extracted_dir = output_dir.join("truth_extracted");
        extract_dir(&truth_dir, &truth_extracted_dir).unwrap();

        // Extract Flow output zip files to output_dir/flow_extracted
        let flow_source_dir = output_dir.join("flow");
        let flow_extracted_dir = output_dir.join("flow_extracted");
        extract_dir(&flow_source_dir, &flow_extracted_dir).unwrap();

        decompress_glbs(&flow_extracted_dir);

        let tests = &profile.tests;
        let relative_path_display = relative_path.display();

        if let Some(cfg) = &tests.json_attributes {
            run_test("json_attributes", &relative_path_display, || {
                json_attributes::test_json_attributes(
                    &truth_dir,
                    &flow_source_dir,
                    &truth_extracted_dir,
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
                        entry.mode,
                    )?;
                }
                Ok(())
            });
        }

        if !profile.convs.raster3d.is_empty() {
            run_test("convs_raster3d", &relative_path_display, || {
                for entry in profile.convs.raster3d.values() {
                    let tileset_dir = output_dir.join("flow_extracted").join(&entry.path);
                    let png_dir = output_dir.join("flow_extracted").join(&entry.truth_path);
                    fs::create_dir_all(&png_dir)
                        .map_err(|e| format!("Failed to create {:?}: {}", png_dir, e))?;
                    let (w, h) = entry.size.dimensions();
                    conv_raster3d::render_cameras_to_pngs(
                        &tileset_dir,
                        &png_dir,
                        &entry.cameras,
                        w,
                        h,
                    )?;
                }
                Ok(())
            });
        }

        if !profile.convs.cesium_attributes.is_empty() {
            run_test("convs_cesium_attributes", &relative_path_display, || {
                for entry in profile.convs.cesium_attributes.values() {
                    let tileset_dir = output_dir.join("flow_extracted").join(&entry.path);
                    let output_path = output_dir.join("flow_extracted").join(&entry.truth_path);
                    conv_cesium::write_cesium_json(
                        &tileset_dir,
                        &output_path,
                        entry.casts.as_ref(),
                    )?;
                }
                Ok(())
            });
        }

        if !profile.convs.cesium_statistics.is_empty() {
            run_test("cesium_statistics", &relative_path_display, || {
                cesium_statistics::test_cesium_statistics(
                    &truth_dir,
                    &flow_extracted_dir,
                    &profile.convs.cesium_statistics,
                )
            });
        }

        if let Some(cfg) = &tests.json_attributes_v2 {
            run_test("json_attributes_v2", &relative_path_display, || {
                json_attributes_v2::test_json_attributes_v2(&output_dir, &test_path, cfg)
            });
        }

        if let Some(cfg) = &tests.output_files {
            run_test("output_files", &relative_path_display, || {
                for entry in cfg.values() {
                    output_files::test_output_files(&flow_source_dir, entry)?;
                }
                Ok(())
            });
        }

        if let Some(cfg) = &tests.mvt_polygons {
            run_test("mvt_polygons", &relative_path_display, || {
                mvt_polygons::test_mvt_polygons(&truth_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.mvt_lines {
            run_test("mvt_lines", &relative_path_display, || {
                mvt_lines::test_mvt_lines(&truth_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.mvt_points {
            run_test("mvt_points", &relative_path_display, || {
                mvt_points::test_mvt_points(&truth_extracted_dir, &flow_extracted_dir, cfg)
            });
        }

        if let Some(cfg) = &tests.cesium {
            run_test("cesium", &relative_path_display, || {
                cesium::test_cesium(&truth_extracted_dir, &flow_extracted_dir, cfg)
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
                let truth_dir = truth_extracted_dir.join(&conv_entry.truth_path);
                let id = id.clone();
                run_test(&format!("raster/{}", id), &relative_path_display, || {
                    raster::test_raster(&truth_dir, &flow_png_dir, cfg)
                });
            }
        }

        if let Some(raster3d_tests) = &tests.raster3d {
            for (id, cfg) in raster3d_tests {
                let conv_entry = profile.convs.raster3d.get(id).unwrap_or_else(|| {
                    panic!(
                        "tests.raster3d.{} references missing convs.raster3d.{}",
                        id, id
                    )
                });
                let flow_png_dir = output_dir
                    .join("flow_extracted")
                    .join(&conv_entry.truth_path);
                let truth_dir = truth_extracted_dir.join(&conv_entry.truth_path);
                let id = id.clone();
                run_test(&format!("raster3d/{}", id), &relative_path_display, || {
                    raster3d::test_raster3d(&truth_dir, &flow_png_dir, cfg)
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
        // Run testcases concurrently across the testcase set. Each testcase
        // writes to its own output directory and spawns its own Runner with
        // isolated state/storage/logger, so they are safe to interleave.
        // Override the default via PLATEAU_TILES_TEST_CONCURRENCY.
        let concurrency = env::var("PLATEAU_TILES_TEST_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .map(|n| n.max(1)) // 0 would panic rayon::ThreadPoolBuilder::num_threads
            .unwrap_or_else(|| (num_cpus::get() / 2).clamp(1, 4));

        eprintln!(
            "Running {} testcases with concurrency={}",
            DEFAULT_TESTS.len(),
            concurrency
        );

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(concurrency)
            .thread_name(|i| format!("plateau-tiles-test-{i}"))
            .build()
            .expect("failed to build rayon thread pool");

        // Collect outcomes so a single panic doesn't abort the whole run —
        // we still surface every failure, but only after all testcases have
        // had a chance to run (and after their isolated state is cleaned up).
        let failures: Vec<String> = pool.install(|| {
            use rayon::prelude::*;
            DEFAULT_TESTS
                .par_iter()
                .filter_map(|name| {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        run_testcase(&testcases_dir, &results_dir, name, "re");
                    }));
                    if result.is_err() {
                        Some((*name).to_string())
                    } else {
                        None
                    }
                })
                .collect()
        });

        if !failures.is_empty() {
            eprintln!("\n{} testcases failed:", failures.len());
            for name in &failures {
                eprintln!("  - {name}");
            }
            std::process::exit(1);
        }
    }
}
