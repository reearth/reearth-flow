//! The report a render leaves beside its output, and the classification that
//! decides what it says.
//!
//! The shape of this file is dictated by `server/api/pkg/featureview`, which
//! parses what we write and documents `ReportVersion` as "the schema version the
//! renderer writes and this package reads". Every string below is wire
//! protocol: renaming one is a breaking change, not a refactor.

use reearth_flow_feature_view::{Error, Format as ViewFormat, SizeLimited};
use serde_json::{json, Map, Value};

/// The report schema version. Bump only for a change readers cannot absorb.
const REPORT_VERSION: u32 = 1;

/// Which view was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    Gltf,
    Tiles,
}

impl Shape {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Shape::Gltf => "gltf",
            Shape::Tiles => "tiles",
        }
    }

    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "gltf" => Some(Shape::Gltf),
            "tiles" => Some(Shape::Tiles),
            _ => None,
        }
    }
}

/// What the render produced, as the API reports it to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    Ready,
    Empty,
    UnsupportedGeometry,
    Failed,
}

impl Status {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Status::Ready => "ready",
            Status::Empty => "empty",
            Status::UnsupportedGeometry => "unsupported_geometry",
            Status::Failed => "failed",
        }
    }
}

/// What one layer of the render was written as. Chosen by the engine from the
/// geometry it found, which is why the caller cannot know it in advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    Glb,
    Cesium3dTiles,
    VectorTiles,
}

impl From<ViewFormat> for Format {
    fn from(format: ViewFormat) -> Self {
        match format {
            ViewFormat::Glb => Format::Glb,
            ViewFormat::Tiles3d => Format::Cesium3dTiles,
            ViewFormat::VectorTiles => Format::VectorTiles,
        }
    }
}

impl Format {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Format::Glb => "glb",
            Format::Cesium3dTiles => "cesium_3d_tiles",
            Format::VectorTiles => "vector_tiles",
        }
    }
}

/// A classified error: what to report, and whether to exit non-zero.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Outcome {
    pub(crate) status: Status,
    pub(crate) error: Option<String>,
    /// Whether this is a system fault rather than a user-visible outcome. Only
    /// a fault exits non-zero; everything else writes a report and exits 0, so
    /// the reason reaches the user instead of being lost to an exit code.
    pub(crate) fatal: bool,
}

/// Decide what a render failure means to the caller.
pub(crate) fn classify(error: &Error) -> Outcome {
    let message = error.to_string();
    match error {
        // Nothing drew. Not a failure: the selection held nothing the view shows.
        Error::NothingRendered { .. } | Error::RowUndrawable { .. } => Outcome {
            status: Status::Empty,
            error: Some(message),
            fatal: false,
        },
        // A glTF view was asked of a purely 2D row, which is the exact case the
        // consumer's UNSUPPORTED_GEOMETRY status documents.
        Error::TwoDimensional => Outcome {
            status: Status::UnsupportedGeometry,
            error: Some(message),
            fatal: false,
        },
        // The user's expression is at fault, not the system. There is no
        // `invalid_request` status to use, so this is `failed` with a message
        // naming the expression, and a zero exit so the message survives.
        Error::FilterCompile { .. } | Error::FilterEval { .. } => Outcome {
            status: Status::Failed,
            error: Some(message),
            fatal: false,
        },
        // Storage or the renderer broke. A real fault.
        Error::Read { .. } | Error::Write { .. } | Error::Render(_) => Outcome {
            status: Status::Failed,
            error: Some(message),
            fatal: true,
        },
    }
}

/// A path relative to the view directory, which is the documented `version: 1`
/// schema for `entryPoint` and `written`. A URI that is not under `root` comes
/// back unchanged rather than mangled: an absolute (or otherwise wrong) path is
/// a clearer failure to notice than a corrupted relative one.
///
/// This is not, as an earlier version of this comment claimed, because the Go
/// consumer rejects an absolute path. Checked directly against
/// `server/api/pkg/featureview/report.go` on `origin/FLOW-DEV-257`:
/// `ParseReport` validates only `version` and that a `ready` status carries a
/// `format` and entry point, with no `ErrInvalidReport` case for an absolute
/// URI. `intermediatedataview.go`'s `entryPointName()` does not even read our
/// `entryPoint`: it deliberately rederives the name itself, commented as
/// guarding against a wrong or hostile report addressing a file outside the
/// view directory. `report.Written` is never read at all. Relativising here is
/// still correct and worth keeping regardless: it is the schema we document,
/// and it independently avoids leaking the worker's local filesystem layout
/// into a report meant for an external caller, but no current consumer
/// behaviour depends on it.
///
/// Stripping the prefix is not enough on its own: a sibling directory whose
/// name merely starts with `root`'s name (`.../abc` vs `.../abcdef`) would
/// also strip, leaving a relative-looking remainder that points at the wrong
/// location instead of tripping the "not under root" case. So what remains
/// after the strip must be empty (an exact match) or start with `/` (a real
/// path separator) before it counts as being under `root`.
pub(crate) fn relativise(root: &str, uri: &str) -> String {
    let root = root.strip_suffix('/').unwrap_or(root);
    match uri.strip_prefix(root) {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => {
            rest.trim_start_matches('/').to_string()
        }
        _ => uri.to_string(),
    }
}

/// One entry point of a finished view, and what was drawn into it.
#[derive(Debug, Clone)]
pub(crate) struct ReportLayer {
    pub(crate) format: Format,
    pub(crate) entry_point: String,
    pub(crate) selected_features: usize,
    pub(crate) rendered_features: usize,
    /// Written only for vector tiles, the one format with a size cap; absent
    /// means it does not apply.
    pub(crate) size_limited: Option<SizeLimited>,
}

impl ReportLayer {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("format".to_string(), json!(self.format.as_str()));
        map.insert("entryPoint".to_string(), json!(self.entry_point));
        insert_counts(&mut map, self.selected_features, self.rendered_features);
        insert_size_limited(&mut map, self.size_limited);
        Value::Object(map)
    }
}

/// What a render leaves beside its output. Written for every terminal outcome,
/// including the ones that wrote no view.
#[derive(Debug, Clone)]
pub(crate) struct Report {
    pub(crate) status: Status,
    pub(crate) shape: Shape,
    pub(crate) row: Option<usize>,
    pub(crate) filter: Option<String>,
    /// Features the selection kept.
    pub(crate) selected_features: usize,
    /// Selected features drawn into at least one layer.
    pub(crate) rendered_features: usize,
    pub(crate) scanned: usize,
    /// The view's entry points, 3D Tiles before vector tiles. Empty unless the
    /// view is ready.
    pub(crate) layers: Vec<ReportLayer>,
    pub(crate) written: Vec<String>,
    pub(crate) error: Option<String>,
}

impl Report {
    /// The report for a render that was stopped before it finished, written by
    /// the wrapper that stopped it: the render itself never gets to write one.
    ///
    /// The counts are 0 because a stopped render does not say how far it got,
    /// the same as for a filter that failed to evaluate.
    pub(crate) fn stopped(
        shape: Shape,
        row: Option<usize>,
        filter: Option<String>,
        error: String,
    ) -> Self {
        Report {
            status: Status::Failed,
            shape,
            row,
            filter,
            selected_features: 0,
            rendered_features: 0,
            scanned: 0,
            layers: vec![],
            written: vec![],
            error: Some(error),
        }
    }

    /// The JSON the consumer parses. Optional fields are omitted rather than
    /// null, matching the `omitempty` tags on the Go side.
    ///
    /// The top-level `format`, `entryPoint` and size-limited counts repeat the
    /// first layer's, for consumers that read a view as a single entry point.
    pub(crate) fn to_json(&self) -> Value {
        let first = self.layers.first();
        let mut map = Map::new();
        map.insert("version".to_string(), json!(REPORT_VERSION));
        map.insert("engineVersion".to_string(), json!(crate::ENGINE_VERSION));
        map.insert("status".to_string(), json!(self.status.as_str()));
        map.insert("shape".to_string(), json!(self.shape.as_str()));
        if let Some(first) = first {
            map.insert("format".to_string(), json!(first.format.as_str()));
        }
        if let Some(row) = self.row {
            map.insert("row".to_string(), json!(row));
        }
        if let Some(filter) = &self.filter {
            map.insert("filter".to_string(), json!(filter));
        }
        insert_counts(&mut map, self.selected_features, self.rendered_features);
        insert_size_limited(&mut map, first.and_then(|first| first.size_limited));
        map.insert("scanned".to_string(), json!(self.scanned));
        if let Some(first) = first {
            map.insert("entryPoint".to_string(), json!(first.entry_point));
        }
        if !self.layers.is_empty() {
            map.insert(
                "layers".to_string(),
                Value::Array(self.layers.iter().map(ReportLayer::to_json).collect()),
            );
        }
        if !self.written.is_empty() {
            map.insert("written".to_string(), json!(self.written));
        }
        if let Some(error) = &self.error {
            map.insert("error".to_string(), json!(error));
        }
        Value::Object(map)
    }
}

fn insert_counts(map: &mut Map<String, Value>, selected: usize, rendered: usize) {
    map.insert("selectedFeatures".to_string(), json!(selected));
    map.insert("renderedFeatures".to_string(), json!(rendered));
}

fn insert_size_limited(map: &mut Map<String, Value>, size_limited: Option<SizeLimited>) {
    if let Some(size_limited) = size_limited {
        map.insert(
            "sizeLimitedFeatures".to_string(),
            json!(size_limited.features),
        );
        map.insert("sizeLimitedTiles".to_string(), json!(size_limited.tiles));
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use reearth_flow_feature_view::Error;

    #[test]
    fn wire_strings_match_the_consumer() {
        // These exact strings are what server/api/pkg/featureview parses.
        // Changing one is a breaking protocol change, not a rename.
        assert_eq!(Shape::Gltf.as_str(), "gltf");
        assert_eq!(Shape::Tiles.as_str(), "tiles");
        assert_eq!(Status::Ready.as_str(), "ready");
        assert_eq!(Status::Empty.as_str(), "empty");
        assert_eq!(Status::UnsupportedGeometry.as_str(), "unsupported_geometry");
        assert_eq!(Status::Failed.as_str(), "failed");
        assert_eq!(Format::Glb.as_str(), "glb");
        assert_eq!(Format::Cesium3dTiles.as_str(), "cesium_3d_tiles");
        assert_eq!(Format::VectorTiles.as_str(), "vector_tiles");
    }

    #[test]
    fn a_render_that_drew_nothing_is_not_fatal() {
        // The whole point: "nothing matched your filter" must reach the user as
        // an explanation, not as a dead job.
        let outcome = classify(&Error::NothingRendered { selected: 7 });
        assert_eq!(outcome.status, Status::Empty);
        assert!(!outcome.fatal);
        assert!(outcome.error.is_some());
    }

    #[test]
    fn an_undrawable_row_is_empty_and_not_fatal() {
        let outcome = classify(&Error::RowUndrawable { row: 3 });
        assert_eq!(outcome.status, Status::Empty);
        assert!(!outcome.fatal);
    }

    #[test]
    fn a_two_dimensional_row_is_unsupported_geometry() {
        // The consumer's UNSUPPORTED_GEOMETRY status documents exactly this case.
        let outcome = classify(&Error::TwoDimensional);
        assert_eq!(outcome.status, Status::UnsupportedGeometry);
        assert!(!outcome.fatal);
    }

    #[test]
    fn a_storage_failure_is_fatal() {
        let error = Error::Read {
            uri: "gs://bucket/x.jsonl".to_string(),
            source: std::io::Error::other("boom"),
        };
        let outcome = classify(&error);
        assert_eq!(outcome.status, Status::Failed);
        assert!(outcome.fatal, "a real fault must exit non-zero");
    }

    // I3: the classification the spec argues hardest for — a bad filter is the
    // user's to fix, so it must be `failed` and NOT fatal — had no direct
    // test. `execute.rs`'s own filter test used `"false"`, which compiles fine
    // and only exercises the empty-selection short circuit, so flipping this
    // arm to `fatal: true` would have passed every test on the branch.

    #[test]
    fn a_filter_that_will_not_compile_is_failed_and_not_fatal() {
        let error = Error::FilterCompile {
            expr: "(".to_string(),
            source: reearth_flow_expr::eval_error("unexpected end of input"),
        };
        let outcome = classify(&error);
        assert_eq!(outcome.status, Status::Failed);
        assert!(
            !outcome.fatal,
            "the user's expression is theirs to fix, not a system fault"
        );
        assert!(outcome.error.is_some());
    }

    #[test]
    fn a_filter_that_fails_to_evaluate_is_failed_and_not_fatal() {
        let error = Error::FilterEval {
            expr: "foo > 1".to_string(),
            feature_id: "00000000-0000-0000-0000-000000000001".to_string(),
            source: reearth_flow_types::error::Error::input("foo is not a number"),
        };
        let outcome = classify(&error);
        assert_eq!(outcome.status, Status::Failed);
        assert!(
            !outcome.fatal,
            "the user's expression is theirs to fix, not a system fault"
        );
    }

    #[test]
    fn a_write_failure_is_fatal() {
        let error = Error::Write {
            path: "view/tileset.json".to_string(),
            source: Box::new(std::io::Error::other("disk full")),
        };
        let outcome = classify(&error);
        assert_eq!(outcome.status, Status::Failed);
        assert!(outcome.fatal, "a write fault must exit non-zero");
    }

    #[test]
    fn a_render_failure_is_fatal() {
        let error = Error::Render(
            reearth_flow_action_sink::errors::SinkError::cesium3dtiles_writer("boom"),
        );
        let outcome = classify(&error);
        assert_eq!(outcome.status, Status::Failed);
        assert!(outcome.fatal, "a renderer fault must exit non-zero");
    }

    #[test]
    fn every_view_format_has_its_wire_format() {
        assert_eq!(Format::from(ViewFormat::Glb), Format::Glb);
        assert_eq!(Format::from(ViewFormat::Tiles3d), Format::Cesium3dTiles);
        assert_eq!(Format::from(ViewFormat::VectorTiles), Format::VectorTiles);
    }

    #[test]
    fn paths_come_out_relative_to_the_view_directory() {
        // This is the documented `version: 1` schema, not something the Go
        // side enforces on us (it does not reject an absolute URI; see
        // `relativise`'s doc comment). The renderer does not know the public
        // URL its output is served from, so relative is what it can honestly
        // report either way.
        assert_eq!(
            relativise(
                "gs://bucket/feature-view/abc",
                "gs://bucket/feature-view/abc/x/tileset.json"
            ),
            "x/tileset.json",
        );
        // A trailing slash on the root must not leave a leading slash behind.
        assert_eq!(
            relativise(
                "gs://bucket/feature-view/abc/",
                "gs://bucket/feature-view/abc/x.glb"
            ),
            "x.glb",
        );
        // Not under the root: returned unchanged rather than silently mangled.
        assert_eq!(
            relativise("gs://bucket/a", "gs://other/b.glb"),
            "gs://other/b.glb"
        );
    }

    #[test]
    fn a_sibling_directory_that_merely_starts_with_the_root_name_is_not_under_it() {
        // "abcdef" is a sibling of "abc", not a descendant. A plain
        // strip_prefix would wrongly yield "def/x.glb", a normal-looking
        // relative path that silently points at the wrong location instead of
        // tripping the "not under root" fallback below.
        assert_eq!(
            relativise(
                "gs://bucket/feature-view/abc",
                "gs://bucket/feature-view/abcdef/x.glb"
            ),
            "gs://bucket/feature-view/abcdef/x.glb",
        );
    }

    fn tiles_3d_layer(selected: usize, rendered: usize) -> ReportLayer {
        ReportLayer {
            format: Format::Cesium3dTiles,
            entry_point: "x/tileset.json".to_string(),
            selected_features: selected,
            rendered_features: rendered,
            size_limited: None,
        }
    }

    fn vector_tiles_layer(selected: usize, rendered: usize) -> ReportLayer {
        ReportLayer {
            format: Format::VectorTiles,
            entry_point: "x/tilejson.json".to_string(),
            selected_features: selected,
            rendered_features: rendered,
            size_limited: Some(SizeLimited {
                features: 2,
                tiles: 5,
            }),
        }
    }

    fn ready_report(layers: Vec<ReportLayer>) -> Report {
        Report {
            status: Status::Ready,
            shape: Shape::Tiles,
            row: None,
            filter: None,
            selected_features: 10,
            rendered_features: 8,
            scanned: 12,
            layers,
            written: vec!["x/tileset.json".to_string(), "x/0.glb".to_string()],
            error: None,
        }
    }

    #[test]
    fn a_ready_report_serialises_every_field_the_consumer_reads() {
        let report = Report {
            filter: Some("foo".to_string()),
            ..ready_report(vec![tiles_3d_layer(10, 8)])
        };
        let json = report.to_json();
        assert_eq!(json["version"], 1);
        assert_eq!(json["status"], "ready");
        assert_eq!(json["shape"], "tiles");
        assert_eq!(json["format"], "cesium_3d_tiles");
        assert_eq!(json["filter"], "foo");
        assert_eq!(json["selectedFeatures"], 10);
        assert_eq!(json["renderedFeatures"], 8);
        assert_eq!(json["scanned"], 12);
        assert_eq!(json["entryPoint"], "x/tileset.json");
        assert_eq!(json["written"][1], "x/0.glb");
        assert!(json.get("error").is_none(), "error is omitted when absent");
        assert!(json.get("row").is_none(), "row is omitted when absent");
        assert!(
            json.get("sizeLimitedFeatures").is_none() && json.get("sizeLimitedTiles").is_none(),
            "size-limited counts are omitted where there is no size cap"
        );
        assert_eq!(
            json["layers"],
            serde_json::json!([{
                "format": "cesium_3d_tiles",
                "entryPoint": "x/tileset.json",
                "selectedFeatures": 10,
                "renderedFeatures": 8,
            }])
        );
    }

    #[test]
    fn a_vector_tile_report_carries_what_the_size_cap_left_out() {
        let json = ready_report(vec![vector_tiles_layer(10, 8)]).to_json();
        assert_eq!(json["sizeLimitedFeatures"], 2);
        assert_eq!(json["sizeLimitedTiles"], 5);
        assert_eq!(json["layers"][0]["sizeLimitedFeatures"], 2);
        assert_eq!(json["layers"][0]["sizeLimitedTiles"], 5);
    }

    /// A view of two formats lists both, each with its own counts, while the
    /// top level repeats the first layer and counts the whole view.
    #[test]
    fn a_two_layer_report_lists_both_and_leads_with_the_first() {
        let json = ready_report(vec![tiles_3d_layer(8, 8), vector_tiles_layer(8, 6)]).to_json();
        assert_eq!(json["format"], "cesium_3d_tiles");
        assert_eq!(json["entryPoint"], "x/tileset.json");
        assert!(
            json.get("sizeLimitedFeatures").is_none(),
            "the first layer has no size cap"
        );
        assert_eq!(json["selectedFeatures"], 10);
        assert_eq!(json["renderedFeatures"], 8);

        let layers = json["layers"].as_array().expect("layers");
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[1]["format"], "vector_tiles");
        assert_eq!(layers[1]["entryPoint"], "x/tilejson.json");
        assert_eq!(layers[1]["selectedFeatures"], 8);
        assert_eq!(layers[1]["renderedFeatures"], 6);
        assert_eq!(layers[1]["sizeLimitedFeatures"], 2);
    }

    #[test]
    fn every_report_names_the_engine_that_wrote_it() {
        // A cached view is only as current as the engine that rendered it, so
        // the report has to say which one that was, failures included.
        let report = Report::stopped(Shape::Tiles, None, None, "stopped".to_string());
        assert_eq!(report.to_json()["engineVersion"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn a_non_ready_report_omits_format_and_entry_point_and_carries_the_error() {
        let report = Report {
            status: Status::Empty,
            shape: Shape::Tiles,
            row: None,
            filter: None,
            selected_features: 4,
            rendered_features: 0,
            scanned: 9,
            layers: vec![],
            written: vec![],
            error: Some("nothing drew".to_string()),
        };
        let json = report.to_json();
        assert_eq!(json["status"], "empty");
        assert_eq!(json["error"], "nothing drew");
        assert!(json.get("format").is_none());
        assert!(json.get("entryPoint").is_none());
        assert!(json.get("layers").is_none());
        assert!(
            json.get("written").is_none(),
            "written is omitted when empty"
        );
        // Counts are always present, including zero.
        assert_eq!(json["renderedFeatures"], 0);
    }
}
