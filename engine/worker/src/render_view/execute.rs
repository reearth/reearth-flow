//! Running a render and recording what happened.
//!
//! The report is written for every terminal outcome, including the ones that
//! drew nothing, and only a system fault returns `Err`. That split is what lets
//! "this row holds 2D geometry" reach the user as an explanation rather than as
//! a failed job.

use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
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

    let loaded = match load_selected(&input, selection, &storage_resolver) {
        Ok(loaded) => loaded,
        Err(error) => {
            let outcome = classify(&error);
            if outcome.fatal {
                return Err(outcome.error.unwrap_or_else(|| error.to_string()));
            }
            // A filter that will not compile lands here, and is the user's to fix.
            return write_report(
                &args,
                &storage_resolver,
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

    let destination = Destination {
        root: &output,
        prefix: &args.name,
        storage_resolver: &storage_resolver,
    };
    let rendered: Result<RenderedView, _> = match args.shape {
        Shape::Gltf => render_feature(&loaded.selection[0], &options, &destination),
        Shape::Tiles => render_tileset(&loaded.selection, &options, &destination),
    };

    let report = match rendered {
        Ok(view) => {
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

    write_report(&args, &storage_resolver, report)
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

/// Write the report. A failure here is fatal: the report is the only structured
/// channel back to the caller, so if it cannot be written there is nothing left
/// to say. Uses the same storage call `probe_schema.rs` uses (see its
/// `run_probe`): the write goes through the async `put`, because object-store
/// backends like GCS do not implement OpenDAL's blocking layer, and `put_sync`
/// fails there with `Unsupported (persistent) at blocking_write`.
fn write_report(
    args: &RenderViewArgs,
    storage_resolver: &StorageResolver,
    report: Report,
) -> Result<(), String> {
    let uri = Uri::from_str(&args.report_url).map_err(|e| format!("bad --report-url: {e}"))?;
    let body = serde_json::to_vec(&report.to_json())
        .map_err(|e| format!("cannot serialise the report: {e}"))?;
    let storage = storage_resolver
        .resolve(&uri)
        .map_err(|e| format!("cannot resolve --report-url: {e}"))?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("cannot create tokio runtime: {e}"))?;
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
}
