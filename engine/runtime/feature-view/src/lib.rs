//! Renders a viewable form of the features a run leaves on an edge.
//!
//! The input is one JSONL file. The output is either a single glb holding the
//! whole selection, or a 3D Tiles tileset for streaming a whole edge. Rendered
//! features carry no attributes, only [`ROW_INDEX_PROPERTY`], so a click in a
//! viewer resolves to a row in the table the file came from.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_action_sink::file::cesium3dtiles::next::{
    build, build_glb, MetadataOptions, RenderOptions,
};
use reearth_flow_action_sink::SinkOutput;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::Geometry;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Code, CodeType, Feature};
use thiserror::Error as ThisError;

pub use reearth_flow_action_sink::file::cesium3dtiles::next::TextureCodec;

/// Extensions a feature file carries, longest first so the compressed form is
/// matched before the plain one it ends with.
const FEATURE_EXTENSIONS: [&str; 2] = [".jsonl.zst", ".jsonl"];

/// The one property a rendered view carries: the row a picked feature sits at in
/// the table its file came from.
pub const ROW_INDEX_PROPERTY: &str = "rowIndex";

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Failed to read features from {uri}: {source}")]
    Read {
        uri: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Filter expression {expr:?} failed to compile: {message}")]
    FilterCompile { expr: String, message: String },
    #[error("Filter expression {expr:?} failed on feature {feature_id}: {message}")]
    FilterEval {
        expr: String,
        feature_id: String,
        message: String,
    },
    #[error("Failed to render the view: {0}")]
    Render(String),
    #[error("Failed to write {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

/// Which of the two view shapes to render.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewFormat {
    /// One glb holding the whole selection, untiled.
    #[default]
    Gltf,
    /// A 3D Tiles tileset, for streaming a whole edge.
    Cesium3DTiles {
        /// Deepest quadtree level a feature may be placed at.
        max_zoom: u8,
    },
}

/// How to render, independent of which shape is being rendered.
#[derive(Clone, Debug)]
pub struct ViewOptions {
    pub format: ViewFormat,
    /// Draco mesh compression. On by default: it costs about 2% of render time
    /// and cuts a glb several-fold.
    pub draco: bool,
    /// Per-polygon flat normals, so surfaces light rather than read flat.
    pub compute_flat_normal: bool,
    /// Target texel size in metres per pixel. `0.0` keeps full texture detail.
    pub texel_size: f64,
    pub atlas_size: u32,
    pub atlas_extrusion: u32,
    /// JPEG by default rather than the tile writer's KTX2: a view is built on
    /// demand, and JPEG encodes several times faster at a comparable size.
    pub texture_codec: TextureCodec,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            format: ViewFormat::default(),
            draco: true,
            compute_flat_normal: true,
            texel_size: 0.0,
            atlas_size: 2048,
            atlas_extrusion: 0,
            texture_codec: TextureCodec::Jpeg,
        }
    }
}

impl ViewOptions {
    fn render_options(&self) -> RenderOptions {
        RenderOptions {
            draco: self.draco,
            compute_flat_normal: self.compute_flat_normal,
            texel_size: self.texel_size,
            atlas_size: self.atlas_size,
            atlas_extrusion: self.atlas_extrusion,
            texture_codec: self.texture_codec,
        }
    }

    /// The property table is the row index and nothing else, so there is no
    /// attribute exposure left for these to shape.
    fn metadata_options(&self) -> MetadataOptions<'static> {
        MetadataOptions {
            schema_key: None,
            skip_unexposed_attributes: false,
        }
    }
}

/// Where a view's files land: every path is `prefix` joined under `root`.
pub struct Destination<'a> {
    pub root: &'a Uri,
    /// A glTF view writes `{prefix}.glb`; a tileset writes
    /// `{prefix}/tileset.json` alongside its content and subtree files.
    pub prefix: &'a str,
    pub storage_resolver: &'a StorageResolver,
}

impl Destination<'_> {
    fn write(&self, relative_path: &str, bytes: Vec<u8>) -> Result<Uri> {
        let out =
            SinkOutput::new(self.root, relative_path, self.storage_resolver).map_err(|source| {
                Error::Write {
                    path: relative_path.to_string(),
                    source,
                }
            })?;
        let uri = out.uri().clone();
        out.write(Bytes::from(bytes))
            .map_err(|source| Error::Write {
                path: relative_path.to_string(),
                source,
            })?;
        Ok(uri)
    }
}

/// What a render produced.
#[derive(Debug)]
pub struct RenderedView {
    pub rendered_features: usize,
    /// The entry point a viewer opens: the glb, or the tileset's
    /// `tileset.json`.
    pub entry_point: Option<Uri>,
    /// Every file written, the entry point included.
    pub written: Vec<Uri>,
}

/// A selected feature and the line it came from.
#[derive(Clone, Debug)]
pub struct Selected {
    /// 0-based line in the file, which is the row a table shows it at. Survives
    /// filtering, so it still addresses the full table.
    pub row: usize,
    pub feature: Feature,
}

/// Which features a read keeps.
pub enum Selection<'a> {
    All,
    /// The scan stops after the highest row, and no other line is parsed.
    Rows(&'a [usize]),
    /// The features a Flow expression evaluates true for.
    Filter {
        expr: &'a str,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    },
}

/// What a read produced.
pub struct Loaded {
    pub selection: Vec<Selected>,
    /// Non-empty lines examined; short of the file's total under
    /// [`Selection::Rows`], which stops early.
    pub scanned: usize,
}

/// Read a feature file, keeping only what `selection` asks for.
///
/// One line at a time: a line that is not kept is dropped before the next is
/// read, and a kept one has its attributes swapped for the row index at once,
/// so peak memory tracks surviving geometry rather than the file. Rows past the
/// end are ignored, so a stale table selection yields an empty view.
pub fn load_selected(
    input: &Uri,
    selection: Selection<'_>,
    storage_resolver: &StorageResolver,
) -> Result<Loaded> {
    let compiled = match &selection {
        Selection::Filter { expr, .. } => {
            let code = Code::<{ CodeType::FlowExpr as u32 }> {
                ty: CodeType::FlowExpr,
                value: expr.to_string(),
            };
            Some(code.compile().map_err(|e| Error::FilterCompile {
                expr: expr.to_string(),
                message: format!("{e:?}"),
            })?)
        }
        _ => None,
    };

    let wanted: BTreeSet<usize> = match &selection {
        Selection::Rows(rows) => rows.iter().copied().collect(),
        _ => BTreeSet::new(),
    };
    if matches!(selection, Selection::Rows(_)) && wanted.is_empty() {
        return Ok(Loaded {
            selection: Vec::new(),
            scanned: 0,
        });
    }
    let last_wanted = wanted.iter().next_back().copied();

    let (uri, raw) = read_raw(input, storage_resolver)?;
    let data = decode_auto(raw.as_ref()).map_err(|source| Error::Read {
        uri: uri.as_str().to_string(),
        source,
    })?;

    let parse = |line: &[u8], row: usize| -> Result<Feature> {
        serde_json::from_slice(line).map_err(|e| Error::Read {
            uri: uri.as_str().to_string(),
            source: std::io::Error::other(format!("row {row}: {e}")),
        })
    };

    let mut kept = Vec::new();
    let mut row = 0usize;
    for line in data.split(|&b| b == b'\n') {
        let line = trim_ascii(line);
        if line.is_empty() {
            continue;
        }
        let keep = match &selection {
            Selection::All => Some(parse(line, row)?),
            Selection::Rows(_) => match wanted.contains(&row) {
                true => Some(parse(line, row)?),
                false => None,
            },
            Selection::Filter { expr, env_vars } => {
                let feature = parse(line, row)?;
                let matched = compiled
                    .as_ref()
                    .expect("a filter selection compiled its expression")
                    .eval_bool(&feature, Arc::clone(env_vars))
                    .map_err(|e| Error::FilterEval {
                        expr: expr.to_string(),
                        feature_id: feature.id.to_string(),
                        message: format!("{e:?}"),
                    })?;
                matched.then_some(feature)
            }
        };
        if let Some(feature) = keep {
            kept.push(Selected {
                row,
                feature: feature.with_attributes(row_index_attributes(row)),
            });
        }
        row += 1;
        if Some(row - 1) == last_wanted {
            break;
        }
    }

    Ok(Loaded {
        selection: kept,
        scanned: row,
    })
}

/// Read the named file, or its counterpart in the other feature-file extension.
fn read_raw(input: &Uri, storage_resolver: &StorageResolver) -> Result<(Uri, Bytes)> {
    let mut candidates = vec![input.clone()];
    if let (Some(name), Some(dir)) = (
        input
            .file_name()
            .and_then(|n| n.to_str().map(str::to_string)),
        input.parent(),
    ) {
        let stem = FEATURE_EXTENSIONS
            .iter()
            .find_map(|ext| name.strip_suffix(ext))
            .unwrap_or(name.as_str());
        for ext in FEATURE_EXTENSIONS {
            if let Ok(candidate) = dir.join(format!("{stem}{ext}")) {
                if candidate.as_str() != input.as_str() {
                    candidates.push(candidate);
                }
            }
        }
    }

    let mut last: Option<std::io::Error> = None;
    for candidate in candidates {
        let read = storage_resolver
            .resolve(&candidate)
            .map_err(std::io::Error::other)
            .and_then(|storage| {
                storage
                    .get_sync(candidate.path().as_path())
                    .map_err(std::io::Error::other)
            });
        match read {
            Ok(bytes) => return Ok((candidate, bytes)),
            Err(e) => last = Some(e),
        }
    }
    Err(Error::Read {
        uri: input.as_str().to_string(),
        source: last.unwrap_or_else(|| std::io::Error::other("no candidate jsonl found")),
    })
}

/// zstd frame magic is `28 B5 2F FD`; anything else is already plain text.
fn decode_auto(bytes: &[u8]) -> std::io::Result<Cow<'_, [u8]>> {
    if bytes.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
        zstd::stream::decode_all(bytes).map(Cow::Owned)
    } else {
        Ok(Cow::Borrowed(bytes))
    }
}

fn trim_ascii(mut line: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = line {
        if !first.is_ascii_whitespace() {
            break;
        }
        line = rest;
    }
    while let [rest @ .., last] = line {
        if !last.is_ascii_whitespace() {
            break;
        }
        line = rest;
    }
    line
}

/// The single-entry attribute set a rendered feature carries.
fn row_index_attributes(row: usize) -> Attributes {
    let mut attributes = Attributes::new();
    attributes.insert(
        Attribute::new(ROW_INDEX_PROPERTY),
        AttributeValue::Number(row.into()),
    );
    attributes
}

/// Render the selection and write it to `destination`.
pub fn render(
    selection: &[Selected],
    options: &ViewOptions,
    destination: &Destination<'_>,
) -> Result<RenderedView> {
    let features: Vec<Feature> = selection
        .iter()
        .map(|s| s.feature.with_attributes(row_index_attributes(s.row)))
        .collect();
    reject_two_dimensional(&features);

    match options.format {
        ViewFormat::Gltf => render_gltf(&features, options, destination),
        ViewFormat::Cesium3DTiles { max_zoom } => {
            render_tileset(&features, options, max_zoom, destination)
        }
    }
}

fn render_gltf(
    features: &[Feature],
    options: &ViewOptions,
    destination: &Destination<'_>,
) -> Result<RenderedView> {
    let glb = build_glb(
        features,
        options.metadata_options(),
        options.render_options(),
    )
    .map_err(|e| Error::Render(format!("{e:?}")))?;

    let Some(glb) = glb else {
        return Ok(RenderedView {
            rendered_features: 0,
            entry_point: None,
            written: Vec::new(),
        });
    };

    let uri = destination.write(&format!("{}.glb", destination.prefix), glb)?;
    Ok(RenderedView {
        rendered_features: features.len(),
        entry_point: Some(uri.clone()),
        written: vec![uri],
    })
}

fn render_tileset(
    features: &[Feature],
    options: &ViewOptions,
    max_zoom: u8,
    destination: &Destination<'_>,
) -> Result<RenderedView> {
    // Content glbs are handed over as they are built, so peak memory stays at
    // one tile rather than the whole tileset; collect only their paths.
    let written = std::sync::Mutex::new(Vec::new());
    let built = build(
        features,
        options.metadata_options(),
        max_zoom,
        options.render_options(),
        |relative_path, bytes| {
            let uri = destination
                .write(&format!("{}/{relative_path}", destination.prefix), bytes)
                .map_err(|e| {
                    reearth_flow_action_sink::errors::SinkError::Cesium3DTilesWriter(format!("{e}"))
                })?;
            written
                .lock()
                .expect("write log is never poisoned")
                .push(uri);
            Ok(())
        },
    )
    .map_err(|e| Error::Render(format!("{e:?}")))?;

    let mut written = written.into_inner().expect("write log is never poisoned");
    for (relative_path, bytes) in built.subtrees {
        written.push(destination.write(&format!("{}/{relative_path}", destination.prefix), bytes)?);
    }
    let entry_point = destination.write(
        &format!("{}/tileset.json", destination.prefix),
        built.tileset_json.into_bytes(),
    )?;
    written.push(entry_point.clone());

    Ok(RenderedView {
        rendered_features: features.len(),
        entry_point: Some(entry_point),
        written,
    })
}

/// 2D has no view yet. Reaching it is a caller error rather than an empty
/// render, so say so instead of writing a file with nothing in it.
fn reject_two_dimensional(features: &[Feature]) {
    if features.iter().any(|f| holds_two_dimensional(&f.geometry)) {
        unimplemented!("feature views of 2D geometry");
    }
}

fn holds_two_dimensional(geometry: &Geometry) -> bool {
    match geometry {
        Geometry::Euclidean2D(_) => true,
        Geometry::GeometryCollection(collection) => {
            collection.members().iter().any(holds_two_dimensional)
        }
        Geometry::None | Geometry::Euclidean3D(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use reearth_flow_geometry::collection::Collection3D;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
    use reearth_flow_geometry::Euclidean3DGeometry;

    use super::*;

    fn triangle(lat: f64) -> TriangularMesh3D {
        TriangularMesh3D::from_soup(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [
                [lat, 139.0, 10.0],
                [lat, 139.001, 10.0],
                [lat + 0.001, 139.0, 10.0],
            ],
        )
    }

    fn feature(kind: &str, geometry: Geometry) -> Feature {
        let mut attributes = Attributes::new();
        attributes.insert(
            Attribute::new("kind"),
            AttributeValue::String(kind.to_string()),
        );
        let mut feature = Feature::from(attributes);
        feature.set_geometry(geometry);
        feature
    }

    fn mesh_feature(kind: &str, lat: f64) -> Feature {
        feature(
            kind,
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(triangle(lat)))),
        )
    }

    fn write_features(dir: &Path, features: &[Feature]) -> Uri {
        let lines: Vec<String> = features
            .iter()
            .map(|f| serde_json::to_string(f).expect("a feature serializes"))
            .collect();
        let path = dir.join("node.default.jsonl");
        std::fs::write(&path, lines.join("\n")).expect("write features");
        Uri::for_test(&format!("file://{}", path.display()))
    }

    fn render_to(dir: &Path, selection: &[Selected], options: &ViewOptions) -> RenderedView {
        let root = Uri::for_test(&format!("file://{}", dir.display()));
        let resolver = StorageResolver::new();
        render(
            selection,
            options,
            &Destination {
                root: &root,
                prefix: "out",
                storage_resolver: &resolver,
            },
        )
        .expect("render")
    }

    fn glb_json(bytes: &[u8]) -> serde_json::Value {
        let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        serde_json::from_slice(&bytes[20..20 + len]).unwrap()
    }

    /// Every selection mode must agree on what row N is, and a filter must not
    /// renumber what survives it.
    #[test]
    fn selections_agree_on_row_numbers() {
        let dir = tempfile::tempdir().expect("temp dir");
        let features: Vec<Feature> = ["a", "keep", "b", "keep"]
            .iter()
            .enumerate()
            .map(|(i, k)| mesh_feature(k, 35.0 + i as f64))
            .collect();
        let input = write_features(dir.path(), &features);
        let resolver = StorageResolver::new();

        let all = load_selected(&input, Selection::All, &resolver).expect("all");
        assert_eq!(all.scanned, 4);
        assert_eq!(
            all.selection.iter().map(|s| s.row).collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );

        let rows = load_selected(&input, Selection::Rows(&[3, 1]), &resolver).expect("rows");
        assert_eq!(
            rows.selection.iter().map(|s| s.row).collect::<Vec<_>>(),
            vec![1, 3]
        );

        let filtered = load_selected(
            &input,
            Selection::Filter {
                expr: r#"attributes["kind"] == "keep""#,
                env_vars: Arc::new(serde_json::Map::new()),
            },
            &resolver,
        )
        .expect("filter");
        assert_eq!(filtered.scanned, 4);
        assert_eq!(
            filtered.selection.iter().map(|s| s.row).collect::<Vec<_>>(),
            vec![1, 3]
        );
        for selected in &filtered.selection {
            assert_eq!(selected.feature.id, features[selected.row].id);
        }
    }

    /// A view carries the row and no attribute data, and is draco-compressed
    /// with jpeg textures by default.
    #[test]
    fn the_view_carries_only_the_row_index() {
        let dir = tempfile::tempdir().expect("temp dir");
        let selection = [Selected {
            row: 4242,
            feature: mesh_feature("keep", 35.0),
        }];
        render_to(dir.path(), &selection, &ViewOptions::default());

        let glb = std::fs::read(dir.path().join("out.glb")).expect("the glb is written");
        let contains = |n: &str| glb.windows(n.len()).any(|w| w == n.as_bytes());
        assert_eq!(&glb[..4], b"glTF");
        assert!(contains(ROW_INDEX_PROPERTY), "the row index is carried");
        assert!(contains("4242"), "the row is the one it came from");
        assert!(!contains("kind"), "no source attribute name");
        assert!(!contains("keep"), "no source attribute value");

        let json = glb_json(&glb);
        let ext = json["extensionsUsed"].as_array().unwrap();
        assert!(ext.iter().any(|e| e == "KHR_draco_mesh_compression"));
    }

    /// A feature holding several meshes renders all of them, as one pickable
    /// feature.
    #[test]
    fn a_collection_renders_every_member_as_one_feature() {
        let dir = tempfile::tempdir().expect("temp dir");
        let collection =
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::TriangularMesh(Box::new(triangle(35.0))),
                Euclidean3DGeometry::TriangularMesh(Box::new(triangle(35.5))),
            ])));
        let selection = [Selected {
            row: 0,
            feature: feature("k", collection),
        }];
        // Draco packs positions away from the accessor, so read the count off an
        // uncompressed render.
        let options = ViewOptions {
            draco: false,
            ..ViewOptions::default()
        };
        render_to(dir.path(), &selection, &options);

        let json = glb_json(&std::fs::read(dir.path().join("out.glb")).unwrap());
        let accessor = json["meshes"][0]["primitives"][0]["attributes"]["POSITION"]
            .as_u64()
            .unwrap() as usize;
        assert_eq!(
            json["accessors"][accessor]["count"].as_u64(),
            Some(6),
            "both members render: three vertices each"
        );
        assert_eq!(
            json["extensions"]["EXT_structural_metadata"]["propertyTables"][0]["count"].as_u64(),
            Some(1),
            "the collection is one feature, not one per member"
        );
    }

    #[test]
    fn a_tileset_writes_a_tileset_and_content() {
        let dir = tempfile::tempdir().expect("temp dir");
        let selection: Vec<Selected> = [35.0, 35.5]
            .iter()
            .enumerate()
            .map(|(row, &lat)| Selected {
                row,
                feature: mesh_feature("k", lat),
            })
            .collect();
        let options = ViewOptions {
            format: ViewFormat::Cesium3DTiles { max_zoom: 18 },
            ..ViewOptions::default()
        };
        let view = render_to(dir.path(), &selection, &options);

        assert!(dir.path().join("out/tileset.json").exists());
        assert!(view
            .written
            .iter()
            .any(|uri| uri.as_str().contains("/out/content/")));
    }

    #[test]
    fn a_selection_without_geometry_writes_nothing() {
        let dir = tempfile::tempdir().expect("temp dir");
        let selection = [Selected {
            row: 0,
            feature: Feature::from(Attributes::new()),
        }];
        let view = render_to(dir.path(), &selection, &ViewOptions::default());

        assert_eq!(view.rendered_features, 0);
        assert!(view.entry_point.is_none());
        assert!(view.written.is_empty());
    }
}
