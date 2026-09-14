//! The report a render leaves beside its output, and the classification that
//! decides what it says.
//!
//! The shape of this file is dictated by `server/api/pkg/featureview`, which
//! parses what we write and documents `ReportVersion` as "the schema version the
//! renderer writes and this package reads". Every string below is wire
//! protocol: renaming one is a breaking change, not a refactor.

use reearth_flow_feature_view::Error;
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

/// What the render actually wrote. Chosen by the engine from the geometry it
/// found, which is why the caller cannot know it in advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    Glb,
    Cesium3dTiles,
    VectorTiles,
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

/// The format a finished render produced, read off the entry point's file name.
/// A glTF view writes `<name>.glb`; a tileset writes `<name>/tileset.json` for
/// 3D and `<name>/tilejson.json` for 2D.
pub(crate) fn format_of(shape: Shape, entry_point: &str) -> Format {
    match shape {
        Shape::Gltf => Format::Glb,
        Shape::Tiles => {
            if entry_point.ends_with("tilejson.json") {
                Format::VectorTiles
            } else {
                Format::Cesium3dTiles
            }
        }
    }
}

/// A path relative to the view directory, which is what the consumer stores.
/// A URI that is not under `root` comes back unchanged rather than mangled: the
/// consumer rejects it, which is a clearer failure than a corrupted path.
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

/// What a render leaves beside its output. Written for every terminal outcome,
/// including the ones that wrote no view.
#[derive(Debug, Clone)]
pub(crate) struct Report {
    pub(crate) status: Status,
    pub(crate) shape: Shape,
    pub(crate) format: Option<Format>,
    pub(crate) row: Option<usize>,
    pub(crate) filter: Option<String>,
    pub(crate) selected_features: usize,
    pub(crate) rendered_features: usize,
    pub(crate) scanned: usize,
    pub(crate) entry_point: Option<String>,
    pub(crate) written: Vec<String>,
    pub(crate) error: Option<String>,
}

impl Report {
    /// The JSON the consumer parses. Optional fields are omitted rather than
    /// null, matching the `omitempty` tags on the Go side.
    pub(crate) fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("version".to_string(), json!(REPORT_VERSION));
        map.insert("status".to_string(), json!(self.status.as_str()));
        map.insert("shape".to_string(), json!(self.shape.as_str()));
        if let Some(format) = self.format {
            map.insert("format".to_string(), json!(format.as_str()));
        }
        if let Some(row) = self.row {
            map.insert("row".to_string(), json!(row));
        }
        if let Some(filter) = &self.filter {
            map.insert("filter".to_string(), json!(filter));
        }
        map.insert(
            "selectedFeatures".to_string(),
            json!(self.selected_features),
        );
        map.insert(
            "renderedFeatures".to_string(),
            json!(self.rendered_features),
        );
        map.insert("scanned".to_string(), json!(self.scanned));
        if let Some(entry_point) = &self.entry_point {
            map.insert("entryPoint".to_string(), json!(entry_point));
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

    #[test]
    fn format_follows_the_entry_point_file_name() {
        // A glTF view writes <name>.glb; a tileset writes <name>/tileset.json
        // for 3D and <name>/tilejson.json for 2D.
        assert_eq!(format_of(Shape::Gltf, "gs://b/v/x.glb"), Format::Glb);
        assert_eq!(
            format_of(Shape::Tiles, "gs://b/v/x/tileset.json"),
            Format::Cesium3dTiles,
        );
        assert_eq!(
            format_of(Shape::Tiles, "gs://b/v/x/tilejson.json"),
            Format::VectorTiles,
        );
    }

    #[test]
    fn paths_come_out_relative_to_the_view_directory() {
        // The Go side rejects an absolute URI as ErrInvalidReport: the renderer
        // does not know the public URL its output is served from.
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
        // relative path that is not absolute, so the Go consumer's
        // ErrInvalidReport check for absolute paths would accept it silently
        // instead of rejecting a URI that was never under the root.
        assert_eq!(
            relativise(
                "gs://bucket/feature-view/abc",
                "gs://bucket/feature-view/abcdef/x.glb"
            ),
            "gs://bucket/feature-view/abcdef/x.glb",
        );
    }

    #[test]
    fn a_ready_report_serialises_every_field_the_consumer_reads() {
        let report = Report {
            status: Status::Ready,
            shape: Shape::Tiles,
            format: Some(Format::Cesium3dTiles),
            row: None,
            filter: Some("foo".to_string()),
            selected_features: 10,
            rendered_features: 8,
            scanned: 12,
            entry_point: Some("x/tileset.json".to_string()),
            written: vec!["x/tileset.json".to_string(), "x/0.glb".to_string()],
            error: None,
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
    }

    #[test]
    fn a_non_ready_report_omits_format_and_entry_point_and_carries_the_error() {
        let report = Report {
            status: Status::Empty,
            shape: Shape::Tiles,
            format: None,
            row: None,
            filter: None,
            selected_features: 4,
            rendered_features: 0,
            scanned: 9,
            entry_point: None,
            written: vec![],
            error: Some("nothing drew".to_string()),
        };
        let json = report.to_json();
        assert_eq!(json["status"], "empty");
        assert_eq!(json["error"], "nothing drew");
        assert!(json.get("format").is_none());
        assert!(json.get("entryPoint").is_none());
        assert!(
            json.get("written").is_none(),
            "written is omitted when empty"
        );
        // Counts are always present, including zero.
        assert_eq!(json["renderedFeatures"], 0);
    }
}
