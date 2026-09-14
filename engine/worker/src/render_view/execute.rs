//! Running a render and recording what happened.
//!
//! The report is written for every terminal outcome, including the ones that
//! drew nothing, and only a system fault returns `Err`. That split is what lets
//! "this row holds 2D geometry" reach the user as an explanation rather than as
//! a failed job.

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_common::uri::{Protocol, Uri};
use reearth_flow_feature_view::{
    load_selected, render_feature, render_tileset, Destination, RenderedView, Selection,
    ViewOptions,
};
use reearth_flow_storage::resolve::StorageResolver;

use super::args::RenderViewArgs;
use super::report::{classify, format_of, relativise, Report, Shape, Status};

/// Render, then write the report. Returns `Err` only for a fault the caller
/// should see as a failed job.
pub fn execute(args: RenderViewArgs) -> Result<(), String> {
    let storage_resolver = Arc::new(StorageResolver::new());

    let input = Uri::from_str(&args.input).map_err(|e| format!("bad --input: {e}"))?;
    let output = Uri::from_str(&args.output).map_err(|e| format!("bad --output: {e}"))?;

    // One runtime drives every async storage call this run needs: staging a
    // remote input in, uploading a rendered output back out, and writing the
    // report. All three exist for the same reason `write_report`'s runtime
    // originally did: opendal's object-store backends (GCS included) declare
    // `BlockingReader = ()` / `BlockingWriter = ()`, so the synchronous
    // `get_sync` / `put_sync` calls `load_selected` and `Destination` use fail
    // immediately against `gs://`, before any network call. `main()` is plain
    // sync with no ambient tokio runtime, so the async calls need one built
    // and driven explicitly. See `probe_schema.rs`'s `run_probe` for the same
    // pattern with the same comment.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("cannot create tokio runtime: {e}"))?;

    // RAII: dropping `stage` removes the temp directory on every exit path
    // below, including an early `?`/`return Err`, without needing a cleanup
    // call at each one.
    let stage = Staging::new().map_err(|e| format!("cannot create a staging directory: {e}"))?;

    // Fast path: a local `file://` input needs no copy, since `load_selected`'s
    // `get_sync` works fine against the `fs` backend.
    let local_input = if input.protocol() == Protocol::File {
        input.clone()
    } else {
        stage_input(&input, &storage_resolver, &runtime, &stage.input_dir)?
    };

    let options = view_options(&args);
    let variables = Arc::new(serde_json::Map::new());
    let selection = match (args.shape, args.row, &args.filter) {
        (Shape::Gltf, Some(row), _) => Selection::Row(row),
        (Shape::Tiles, _, Some(expr)) => Selection::Filter {
            expr,
            variables: variables.clone(),
        },
        _ => Selection::All,
    };

    let loaded = match load_selected(&local_input, selection, &storage_resolver) {
        Ok(loaded) => loaded,
        Err(error) => {
            let outcome = classify(&error);
            if outcome.fatal {
                return Err(outcome.error.unwrap_or_else(|| error.to_string()));
            }
            // A filter the user wrote lands here, and is theirs to fix.
            //
            // `scanned` is 0 rather than a partial count. For `FilterCompile`
            // that is exact: compilation happens before the first line is read.
            // For `FilterEval` it undercounts, because the loader returns an
            // error rather than a `Loaded`, so the lines it examined before the
            // expression failed are not surfaced to us. Reporting 0 is honest
            // about what we know; carrying the real count would mean adding it
            // to `feature_view::Error::FilterEval`.
            return write_report(
                &args,
                &storage_resolver,
                &runtime,
                Report {
                    status: outcome.status,
                    shape: args.shape,
                    format: None,
                    row: args.row,
                    filter: args.filter.clone(),
                    selected_features: 0,
                    rendered_features: 0,
                    scanned: 0,
                    entry_point: None,
                    written: vec![],
                    error: outcome.error,
                },
            );
        }
    };

    let selected_features = loaded.selection.len();
    let scanned = loaded.scanned;

    // Nothing selected renders nothing. Say so rather than asking the renderer.
    if loaded.selection.is_empty() {
        return write_report(
            &args,
            &storage_resolver,
            &runtime,
            Report {
                status: Status::Empty,
                shape: args.shape,
                format: None,
                row: args.row,
                filter: args.filter.clone(),
                selected_features,
                rendered_features: 0,
                scanned,
                entry_point: None,
                written: vec![],
                error: Some("the selection kept no features".to_string()),
            },
        );
    }

    // Fast path: a local `file://` output needs no staging, since
    // `Destination`'s `put_sync` works fine against the `fs` backend. A
    // remote output is rendered into a local temp directory instead, then
    // uploaded below, because `Destination` -> `SinkOutput::write` also goes
    // through `put_sync`.
    let stage_output = output.protocol() != Protocol::File;
    let render_root = if stage_output {
        file_uri(&stage.output_dir)?
    } else {
        output.clone()
    };
    let destination = Destination {
        root: &render_root,
        prefix: &args.name,
        storage_resolver: &storage_resolver,
    };
    let rendered: Result<RenderedView, _> = match args.shape {
        Shape::Gltf => render_feature(&loaded.selection[0], &options, &destination),
        Shape::Tiles => render_tileset(&loaded.selection, &options, &destination),
    };

    let report = match rendered {
        Ok(view) => {
            let view = if stage_output {
                upload_rendered(&view, &render_root, &output, &storage_resolver, &runtime)?
            } else {
                view
            };
            let root = output.as_str();
            Report {
                status: Status::Ready,
                shape: args.shape,
                format: Some(format_of(args.shape, view.entry_point.as_str())),
                row: args.row,
                filter: args.filter.clone(),
                selected_features,
                rendered_features: view.rendered_features,
                scanned,
                entry_point: Some(relativise(root, view.entry_point.as_str())),
                written: view
                    .written
                    .iter()
                    .map(|uri| relativise(root, uri.as_str()))
                    .collect(),
                error: None,
            }
        }
        Err(error) => {
            let outcome = classify(&error);
            if outcome.fatal {
                return Err(outcome.error.unwrap_or_else(|| error.to_string()));
            }
            Report {
                status: outcome.status,
                shape: args.shape,
                format: None,
                row: args.row,
                filter: args.filter.clone(),
                selected_features,
                rendered_features: 0,
                scanned,
                entry_point: None,
                written: vec![],
                error: outcome.error,
            }
        }
    };

    write_report(&args, &storage_resolver, &runtime, report)
}

/// Everything the request does not carry keeps the library default, so a request
/// and an equivalent CLI run render identically. `compute_flat_normal`,
/// `atlas_size`, `atlas_extrusion` and `wrap_tolerance` are deliberately not in
/// the request: adding knobs the consumer never sets would be surface with no
/// caller.
fn view_options(args: &RenderViewArgs) -> ViewOptions {
    ViewOptions {
        draco: args.draco,
        texel_size: args.texel_size,
        texture_codec: args.texture_codec,
        target_tile_size: args.target_tile_size,
        min_zoom: args.min_zoom,
        max_zoom: args.max_zoom,
        extent: args.extent,
        max_tile_bytes: args.max_tile_bytes,
        ..ViewOptions::default()
    }
}

/// A scratch directory this run owns, holding a staged copy of the input (if
/// remote) and the local render output (if the real output is remote). Held
/// for the lifetime of `execute`; dropping it removes both subdirectories.
struct Staging {
    // Kept only to hold the RAII guard alive; never read directly.
    _dir: tempfile::TempDir,
    input_dir: PathBuf,
    output_dir: PathBuf,
}

impl Staging {
    fn new() -> std::io::Result<Self> {
        let dir = tempfile::tempdir()?;
        let input_dir = dir.path().join("in");
        let output_dir = dir.path().join("out");
        std::fs::create_dir_all(&input_dir)?;
        std::fs::create_dir_all(&output_dir)?;
        Ok(Self {
            _dir: dir,
            input_dir,
            output_dir,
        })
    }
}

/// A `file://` URI addressing `path`.
fn file_uri(path: &Path) -> Result<Uri, String> {
    Uri::from_str(&format!("file://{}", path.display()))
        .map_err(|e| format!("cannot address local path {}: {e}", path.display()))
}

/// Copy `input` into `dir`, driving the storage resolver's ASYNC `get` from
/// `runtime`, and return the local `file://` URI `load_selected` should read
/// instead. Only called for a non-`file://` input: opendal's GCS backend
/// declares `BlockingReader = ()`, so `load_selected`'s own `get_sync` fails
/// with `Unsupported (persistent) at blocking_stat` before any network call.
fn stage_input(
    input: &Uri,
    storage_resolver: &StorageResolver,
    runtime: &tokio::runtime::Runtime,
    dir: &Path,
) -> Result<Uri, String> {
    let storage = storage_resolver
        .resolve(input)
        .map_err(|e| format!("cannot resolve --input: {e}"))?;
    let bytes = runtime
        .block_on(async { storage.get(input.path().as_path()).await?.bytes().await })
        .map_err(|e| format!("cannot read {}: {e}", input.as_str()))?;

    let file_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("--input {} has no file name", input.as_str()))?;
    let local_path = dir.join(file_name);
    std::fs::write(&local_path, &bytes)
        .map_err(|e| format!("cannot stage --input to {}: {e}", local_path.display()))?;

    file_uri(&local_path)
}

/// Upload every file a render wrote from the local staging root to the real
/// output root, driving the storage resolver's ASYNC `put` from `runtime`, and
/// return a [`RenderedView`] whose `entry_point` and `written` name the
/// uploaded (real) locations rather than the local ones. Only called when the
/// real output is not `file://`: `Destination` writes through
/// `SinkOutput::write`, which is opendal's synchronous `put_sync` and fails
/// against GCS the same way `load_selected`'s read does (`BlockingWriter =
/// ()`).
fn upload_rendered(
    view: &RenderedView,
    local_root: &Uri,
    real_root: &Uri,
    storage_resolver: &StorageResolver,
    runtime: &tokio::runtime::Runtime,
) -> Result<RenderedView, String> {
    let mut written = Vec::with_capacity(view.written.len());
    let mut entry_point = None;
    for uri in &view.written {
        // The relative path is identical under either root: both are the same
        // `Destination { root, prefix }` shape, just staged vs. real.
        let relative = relativise(local_root.as_str(), uri.as_str());
        let bytes = std::fs::read(uri.path().as_path())
            .map_err(|e| format!("cannot read staged render output {}: {e}", uri.as_str()))?;
        let final_uri = real_root
            .join(&relative)
            .map_err(|e| format!("cannot address upload target for {relative}: {e}"))?;
        let storage = storage_resolver
            .resolve(&final_uri)
            .map_err(|e| format!("cannot resolve upload target {}: {e}", final_uri.as_str()))?;
        runtime
            .block_on(storage.put(final_uri.path().as_path(), bytes::Bytes::from(bytes)))
            .map_err(|e| format!("cannot upload {} to {}: {e}", relative, final_uri.as_str()))?;
        if uri.as_str() == view.entry_point.as_str() {
            entry_point = Some(final_uri.clone());
        }
        written.push(final_uri);
    }
    let entry_point = entry_point
        .ok_or_else(|| "the render's entry point was not among its written files".to_string())?;
    Ok(RenderedView {
        rendered_features: view.rendered_features,
        entry_point,
        written,
    })
}

/// Write the report. A failure here is fatal: the report is the only structured
/// channel back to the caller, so if it cannot be written there is nothing left
/// to say. Uses the same storage call `probe_schema.rs` uses (see its
/// `run_probe`): the write goes through the async `put`, because object-store
/// backends like GCS do not implement OpenDAL's blocking layer, and `put_sync`
/// fails there with `Unsupported (persistent) at blocking_write`.
fn write_report(
    args: &RenderViewArgs,
    storage_resolver: &StorageResolver,
    runtime: &tokio::runtime::Runtime,
    report: Report,
) -> Result<(), String> {
    let uri = Uri::from_str(&args.report_url).map_err(|e| format!("bad --report-url: {e}"))?;
    let body = serde_json::to_vec(&report.to_json())
        .map_err(|e| format!("cannot serialise the report: {e}"))?;
    let storage = storage_resolver
        .resolve(&uri)
        .map_err(|e| format!("cannot resolve --report-url: {e}"))?;
    runtime
        .block_on(storage.put(uri.path().as_path(), bytes::Bytes::from(body)))
        .map_err(|e| format!("cannot write the report to {}: {e}", args.report_url))?;
    tracing::info!("Wrote render-view report to {}", args.report_url);
    Ok(())
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use std::io::Write;

    /// One 2D point feature, as the intermediate-data writer emits it.
    ///
    /// `Feature`, `Geometry`, `CoordinateFrame` and `Point2D` all derive plain
    /// `serde` with no `#[serde(tag = ...)]`, so an enum serialises externally
    /// tagged: `Geometry::Euclidean2D(...)` is `{"Euclidean2D": ...}`, and
    /// `CoordinateFrame::Crs(4326)` is `{"Crs": 4326}`. See
    /// `engine/schema/feature-intermediate.schema.json` for the full shape.
    fn write_fixture(dir: &std::path::Path) -> String {
        let path = dir.join("in.jsonl");
        let mut file = std::fs::File::create(&path).expect("create fixture");
        writeln!(
            file,
            r#"{{"id":"00000000-0000-0000-0000-000000000001","attributes":{{}},"geometry":{{"Euclidean2D":{{"Point":{{"frame":{{"Crs":4326}},"position":[35.0,139.0]}}}}}}}}"#
        )
        .expect("write fixture");
        format!("file://{}", path.display())
    }

    fn args_for(dir: &std::path::Path, input: &str, shape: Shape) -> RenderViewArgs {
        RenderViewArgs {
            input: input.to_string(),
            output: format!("file://{}", dir.join("out").display()),
            report_url: format!("file://{}", dir.join("report.json").display()),
            name: "view".to_string(),
            shape,
            row: if matches!(shape, Shape::Gltf) {
                Some(0)
            } else {
                None
            },
            filter: None,
            draco: true,
            texel_size: 0.0,
            texture_codec: reearth_flow_feature_view::TextureCodec::Jpeg,
            target_tile_size: 1_048_576,
            min_zoom: 0,
            max_zoom: 2,
            extent: 4096,
            max_tile_bytes: 500_000,
        }
    }

    fn read_report(dir: &std::path::Path) -> serde_json::Value {
        let raw = std::fs::read_to_string(dir.join("report.json")).expect("report written");
        serde_json::from_str(&raw).expect("report is json")
    }

    #[test]
    fn a_tiles_render_writes_a_ready_report_with_relative_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = write_fixture(dir.path());
        execute(args_for(dir.path(), &input, Shape::Tiles)).expect("a 2D point renders");

        let report = read_report(dir.path());
        assert_eq!(report["version"], 1);
        assert_eq!(report["status"], "ready");
        assert_eq!(report["shape"], "tiles");
        assert_eq!(report["format"], "vector_tiles");
        assert_eq!(report["selectedFeatures"], 1);
        assert_eq!(report["renderedFeatures"], 1);
        assert_eq!(report["scanned"], 1);

        let entry = report["entryPoint"].as_str().expect("an entry point");
        assert!(
            !entry.starts_with("file://") && !entry.starts_with('/'),
            "paths must be relative to the view directory, got {entry}",
        );
        for written in report["written"].as_array().expect("written") {
            let path = written.as_str().unwrap();
            assert!(
                !path.starts_with("file://"),
                "written path {path} is absolute"
            );
        }
    }

    #[test]
    fn a_gltf_view_of_a_two_dimensional_row_is_reported_not_failed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = write_fixture(dir.path());
        // Exits Ok: the reason belongs in the report, not in the exit code.
        execute(args_for(dir.path(), &input, Shape::Gltf)).expect("not a fatal error");

        let report = read_report(dir.path());
        assert_eq!(report["status"], "unsupported_geometry");
        assert!(report["error"].as_str().is_some());
        assert!(report.get("entryPoint").is_none());
    }

    #[test]
    fn a_filter_matching_nothing_is_empty_and_not_fatal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = write_fixture(dir.path());
        let mut args = args_for(dir.path(), &input, Shape::Tiles);
        args.filter = Some("false".to_string());
        execute(args).expect("not a fatal error");

        let report = read_report(dir.path());
        assert_eq!(report["status"], "empty");
        assert_eq!(report["selectedFeatures"], 0);
        assert_eq!(report["renderedFeatures"], 0);
    }

    #[test]
    fn a_missing_input_is_fatal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = format!("file://{}", dir.path().join("nope.jsonl").display());
        let result = execute(args_for(dir.path(), &missing, Shape::Tiles));
        assert!(result.is_err(), "a storage fault must exit non-zero");
    }

    /// I3: `"false"` compiles fine and exercises the empty-selection short
    /// circuit, not the filter-*failure* path. A genuinely malformed
    /// expression is needed to reach `Error::FilterCompile` end to end.
    #[test]
    fn a_malformed_filter_expression_is_reported_failed_not_fatal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = write_fixture(dir.path());
        let mut args = args_for(dir.path(), &input, Shape::Tiles);
        args.filter = Some("(".to_string());
        execute(args).expect("a bad filter is reported, not fatal");

        let report = read_report(dir.path());
        assert_eq!(report["status"], "failed");
        assert_eq!(report["selectedFeatures"], 0);
        assert_eq!(report["renderedFeatures"], 0);
        let error = report["error"].as_str().expect("an error message");
        assert!(
            error.contains('('),
            "the report should name the failing expression, got {error:?}"
        );
    }

    /// C1 regression coverage: every `gs://` render used to die at the first
    /// `load_selected` read, before any report was written, because opendal's
    /// GCS backend has no blocking reader/writer. This proves the staged
    /// read/render/upload path works against a real (emulated) GCS backend.
    ///
    /// Skipped (not `#[ignore]`, so it never silently bit-rots CI's radar) when
    /// `STORAGE_EMULATOR_HOST` is unset, which is the case for every CI run:
    /// CI does not run `docker compose`. Run locally with:
    ///
    /// ```sh
    /// docker network create reearth-flow-net   # once
    /// docker compose -f engine/compose.yml up -d gcs
    /// STORAGE_EMULATOR_HOST=http://localhost:4443 cargo test -p reearth-flow-worker \
    ///     a_gs_render_survives_the_synchronous_io_gap -- --nocapture
    /// ```
    #[test]
    fn a_gs_render_survives_the_synchronous_io_gap() {
        let Ok(emulator) = std::env::var("STORAGE_EMULATOR_HOST") else {
            eprintln!(
                "skipping a_gs_render_survives_the_synchronous_io_gap: \
                 STORAGE_EMULATOR_HOST is not set (needs `docker compose -f \
                 engine/compose.yml up -d gcs`)"
            );
            return;
        };
        eprintln!("using STORAGE_EMULATOR_HOST={emulator}");

        let bucket = "reearth-flow-oss-bucket";
        // Unique per run so repeated local runs never collide with what a
        // previous run left in the emulator's persisted volume.
        let test_id = uuid::Uuid::new_v4();
        let prefix = format!("render-view-c1-test/{test_id}");

        let storage_resolver = StorageResolver::new();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");

        // Seed the input the render will read, straight into the emulator.
        let input_uri = Uri::for_test(&format!("gs://{bucket}/{prefix}/in.jsonl"));
        let fixture = concat!(
            r#"{"id":"00000000-0000-0000-0000-000000000001","attributes":{},"geometry":"#,
            r#"{"Euclidean2D":{"Point":{"frame":{"Crs":4326},"position":[35.0,139.0]}}}}"#,
            "\n"
        );
        let storage = storage_resolver
            .resolve(&input_uri)
            .expect("resolve gs:// input");
        runtime
            .block_on(storage.put(
                input_uri.path().as_path(),
                bytes::Bytes::from_static(fixture.as_bytes()),
            ))
            .expect("seed the gs:// input fixture");

        let output_uri = format!("gs://{bucket}/{prefix}/out");
        let report_uri_str = format!("gs://{bucket}/{prefix}/report.json");

        let args = RenderViewArgs {
            input: input_uri.as_str().to_string(),
            output: output_uri,
            report_url: report_uri_str.clone(),
            name: "view".to_string(),
            shape: Shape::Tiles,
            row: None,
            filter: None,
            draco: true,
            texel_size: 0.0,
            texture_codec: reearth_flow_feature_view::TextureCodec::Jpeg,
            target_tile_size: 1_048_576,
            min_zoom: 0,
            max_zoom: 2,
            extent: 4096,
            max_tile_bytes: 500_000,
        };

        // Before the C1 fix this died inside `load_selected`'s `get_sync`
        // with `Unsupported (persistent) at blocking_stat`, before any report
        // was written at all.
        execute(args).expect("a gs:// render must not die on the synchronous I/O gap");

        let report_uri = Uri::for_test(&report_uri_str);
        let report_storage = storage_resolver
            .resolve(&report_uri)
            .expect("resolve gs:// report");
        let report_bytes = runtime
            .block_on(async {
                report_storage
                    .get(report_uri.path().as_path())
                    .await?
                    .bytes()
                    .await
            })
            .expect("the report was written to gs://");
        let report: serde_json::Value =
            serde_json::from_slice(&report_bytes).expect("report is json");

        assert_eq!(report["status"], "ready", "report was: {report}");
        assert_eq!(report["shape"], "tiles");
        let entry = report["entryPoint"].as_str().expect("an entry point");
        assert!(
            !entry.starts_with("gs://") && !entry.starts_with('/'),
            "entryPoint must be relative to the view directory, got {entry}",
        );
        for written in report["written"].as_array().expect("written") {
            let path = written.as_str().unwrap();
            assert!(
                !path.starts_with("gs://"),
                "written path {path} is absolute"
            );
        }

        // The uploaded entry point must actually exist at the real location
        // the relative path implies, not just be named in the report.
        let real_root = Uri::for_test(&format!("gs://{bucket}/{prefix}/out"));
        let entry_uri = real_root.join(entry).expect("join entry point");
        let entry_storage = storage_resolver
            .resolve(&entry_uri)
            .expect("resolve uploaded entry point");
        runtime
            .block_on(async {
                entry_storage
                    .get(entry_uri.path().as_path())
                    .await?
                    .bytes()
                    .await
            })
            .expect("the entry point was actually uploaded to gs://");
    }
}
