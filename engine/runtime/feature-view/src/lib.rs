//! Renders a viewable form of the features a run leaves on an edge.
//!
//! The input is one JSONL file. The output is either a glb holding one picked
//! feature, or a 3D Tiles tileset for streaming a whole edge. Rendered features
//! carry no attributes, only [`ROW_INDEX_PROPERTY`], so a click in a viewer
//! resolves to a row in the table the file came from.

// The whole crate is new-geometry only; see Cargo.toml.
#![cfg(feature = "new-geometry")]

use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex, PoisonError};

use bytes::Bytes;
use reearth_flow_action_sink::errors::SinkError;
use reearth_flow_action_sink::file::cesium3dtiles::next::{
    build, build_glb, MetadataOptions, RenderOptions,
};
use reearth_flow_action_sink::SinkOutput;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::Geometry;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{
    Attribute, AttributeValue, Attributes, Code, CodeType, CompiledCode, Feature,
};
use thiserror::Error as ThisError;

pub use reearth_flow_action_sink::file::cesium3dtiles::next::TextureCodec;

/// Extensions a feature file carries, longest first so the compressed form is
/// matched before the plain one it ends with.
const FEATURE_EXTENSIONS: [&str; 2] = [".jsonl.zst", ".jsonl"];

/// The one property a rendered view carries: the row a picked feature sits at in
/// the table its file came from.
pub const ROW_INDEX_PROPERTY: &str = "rowIndex";

/// The base name a view takes from its input: the file name with the
/// intermediate-data extension dropped.
pub fn default_name(input: &Uri) -> Option<String> {
    let name = input.file_name()?.to_str()?;
    Some(strip_feature_extension(name).to_string())
}

fn strip_feature_extension(name: &str) -> &str {
    FEATURE_EXTENSIONS
        .iter()
        .find_map(|ext| name.strip_suffix(ext))
        .unwrap_or(name)
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Failed to read features from {uri}: {source}")]
    Read {
        uri: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Filter expression {expr:?} failed to compile: {source}")]
    FilterCompile {
        expr: String,
        #[source]
        source: reearth_flow_expr::Error,
    },
    #[error("Filter expression {expr:?} failed on feature {feature_id}: {source}")]
    FilterEval {
        expr: String,
        feature_id: String,
        #[source]
        source: reearth_flow_types::error::Error,
    },
    #[error("Failed to render the view: {0}")]
    Render(#[from] SinkError),
    #[error("Views of 2D geometry are not supported yet")]
    TwoDimensional,
    #[error("Failed to write {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

/// How to render, shared by both view shapes.
#[derive(Clone, Debug)]
pub struct ViewOptions {
    /// Draco mesh compression. On by default.
    pub draco: bool,
    /// Per-polygon flat normals, so surfaces light rather than read flat.
    pub compute_flat_normal: bool,
    /// Target texel size in metres per pixel. `0.0` keeps full texture detail.
    pub texel_size: f64,
    pub atlas_size: u32,
    pub atlas_extrusion: u32,
    /// JPEG by default: a view is built on demand, so we need fast encoding.
    pub texture_codec: TextureCodec,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
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
            array_map_separator: None,
        }
    }
}

/// A selected feature, as it was read, and the line it came from.
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
    /// One row, for a glTF view. The scan stops there, and no later line is
    /// read at all.
    Row(usize),
    /// The features a Flow expression evaluates true for.
    Filter {
        expr: &'a str,
        variables: Arc<serde_json::Map<String, serde_json::Value>>,
    },
}

/// What a read produced.
pub struct Loaded {
    pub selection: Vec<Selected>,
    /// Non-empty lines examined; short of the file's total under
    /// [`Selection::Row`], which stops early.
    pub scanned: usize,
}

/// A [`Selection`] with its per-line work done up front: the filter compiled.
/// Holding it in the value the loop matches on leaves no separate state to keep
/// in step with it.
enum Scan<'a> {
    All,
    Row(usize),
    Filter {
        expr: &'a str,
        code: CompiledCode,
        variables: Arc<serde_json::Map<String, serde_json::Value>>,
    },
}

impl<'a> Scan<'a> {
    fn prepare(selection: Selection<'a>) -> Result<Self> {
        Ok(match selection {
            Selection::All => Scan::All,
            Selection::Row(row) => Scan::Row(row),
            Selection::Filter { expr, variables } => {
                let code = Code::<{ CodeType::FlowExpr as u32 }> {
                    ty: CodeType::FlowExpr,
                    value: expr.to_string(),
                };
                Scan::Filter {
                    code: code.compile().map_err(|source| Error::FilterCompile {
                        expr: expr.to_string(),
                        source,
                    })?,
                    expr,
                    variables,
                }
            }
        })
    }

    /// Whether `row` is the last one this scan has any reason to read.
    fn stops_at(&self, row: usize) -> bool {
        matches!(self, Scan::Row(wanted) if *wanted == row)
    }
}

/// Read a feature file, keeping only what `selection` asks for.
///
/// Lines are decompressed and parsed one at a time and dropped unless kept, so
/// the decompressed file is never held whole, and [`Selection::Row`] stops at
/// its row rather than decoding the rest. The compressed bytes are read whole,
/// which is what the storage backends offer. A row past the end is ignored, so
/// a stale table selection yields an empty view.
pub fn load_selected(
    input: &Uri,
    selection: Selection<'_>,
    storage_resolver: &StorageResolver,
) -> Result<Loaded> {
    let scan = Scan::prepare(selection)?;

    let (uri, raw) = read_raw(input, storage_resolver)?;
    let read_error = |source: std::io::Error| Error::Read {
        uri: uri.as_str().to_string(),
        source,
    };
    let parse = |line: &[u8], row: usize| -> Result<Feature> {
        serde_json::from_slice(line)
            .map_err(|e| read_error(std::io::Error::other(format!("row {row}: {e}"))))
    };

    let mut lines = line_reader(raw.as_ref()).map_err(read_error)?;
    let mut kept = Vec::new();
    let mut row = 0usize;
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        if lines.read_until(b'\n', &mut buffer).map_err(read_error)? == 0 {
            break;
        }
        let line = buffer.trim_ascii();
        if line.is_empty() {
            continue;
        }
        let keep = match &scan {
            Scan::All => Some(parse(line, row)?),
            Scan::Row(wanted) => match *wanted == row {
                true => Some(parse(line, row)?),
                false => None,
            },
            Scan::Filter {
                expr,
                code,
                variables,
            } => {
                let feature = parse(line, row)?;
                let matched =
                    code.eval_bool(&feature, Arc::clone(variables))
                        .map_err(|source| Error::FilterEval {
                            expr: expr.to_string(),
                            feature_id: feature.id.to_string(),
                            source,
                        })?;
                matched.then_some(feature)
            }
        };
        if let Some(feature) = keep {
            kept.push(Selected { row, feature });
        }
        let done = scan.stops_at(row);
        row += 1;
        if done {
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
    // Report the failure for the file that was actually asked for; the
    // counterpart extension is a fallback, and its error only distracts.
    let mut asked_for: Option<std::io::Error> = None;
    for candidate in read_candidates(input) {
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
            Err(e) => asked_for = asked_for.or(Some(e)),
        }
    }
    Err(Error::Read {
        uri: input.as_str().to_string(),
        source: asked_for.unwrap_or_else(|| std::io::Error::other("no candidate jsonl found")),
    })
}

/// `input` itself, then the same stem under every other feature-file extension.
fn read_candidates(input: &Uri) -> Vec<Uri> {
    let mut candidates = vec![input.clone()];
    let (Some(name), Some(dir)) = (
        input
            .file_name()
            .and_then(|n| n.to_str().map(str::to_string)),
        input.parent(),
    ) else {
        return candidates;
    };
    let stem = strip_feature_extension(&name);
    for ext in FEATURE_EXTENSIONS {
        if let Ok(candidate) = dir.join(format!("{stem}{ext}")) {
            if candidate.as_str() != input.as_str() {
                candidates.push(candidate);
            }
        }
    }
    candidates
}

/// Lines of `bytes`, decompressing as it goes if they are a zstd frame (magic
/// `28 B5 2F FD`); anything else is already plain text.
fn line_reader(bytes: &[u8]) -> std::io::Result<Box<dyn BufRead + '_>> {
    if bytes.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
        Ok(Box::new(BufReader::new(zstd::stream::read::Decoder::new(
            bytes,
        )?)))
    } else {
        Ok(Box::new(bytes))
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
    /// Write `{prefix}{suffix}`, for a shape whose whole output is one file.
    fn write_suffixed(&self, suffix: &str, bytes: Vec<u8>) -> Result<Uri> {
        self.write_path(&format!("{}{suffix}", self.prefix), bytes)
    }

    /// Write `{prefix}/{relative_path}`, for a shape that writes a directory.
    fn write_under(&self, relative_path: &str, bytes: Vec<u8>) -> Result<Uri> {
        self.write_path(&format!("{}/{relative_path}", self.prefix), bytes)
    }

    fn write_path(&self, path: &str, bytes: Vec<u8>) -> Result<Uri> {
        let write = || {
            let out = SinkOutput::new(self.root, path, self.storage_resolver)?;
            let uri = out.uri().clone();
            out.write(Bytes::from(bytes))?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(uri)
        };
        write().map_err(|source| Error::Write {
            path: path.to_string(),
            source,
        })
    }
}

/// What a render produced.
#[derive(Debug)]
pub struct RenderedView {
    /// Selected features that carried renderable geometry; the rest are absent
    /// from the output.
    pub rendered_features: usize,
    /// The entry point a viewer opens: the glb, or the tileset's
    /// `tileset.json`.
    pub entry_point: Option<Uri>,
    /// Every file written, the entry point included.
    pub written: Vec<Uri>,
}

/// Render one feature into `{prefix}.glb`, untiled: the view behind clicking a
/// table row.
pub fn render_feature(
    selected: &Selected,
    options: &ViewOptions,
    destination: &Destination<'_>,
) -> Result<RenderedView> {
    reject_two_dimensional(std::slice::from_ref(selected))?;
    let feature = selected
        .feature
        .with_attributes(row_index_attributes(selected.row));

    let Some(glb) = build_glb(
        &feature,
        options.metadata_options(),
        options.render_options(),
    )?
    else {
        return Ok(RenderedView {
            rendered_features: 0,
            entry_point: None,
            written: Vec::new(),
        });
    };

    let uri = destination.write_suffixed(".glb", glb)?;
    Ok(RenderedView {
        rendered_features: 1,
        entry_point: Some(uri.clone()),
        written: vec![uri],
    })
}

/// Render a whole selection into a tileset under `{prefix}/`: the view of an
/// entire edge.
pub fn render_tileset(
    selection: &[Selected],
    target_tile_size: u64,
    options: &ViewOptions,
    destination: &Destination<'_>,
) -> Result<RenderedView> {
    reject_two_dimensional(selection)?;
    let features: Vec<Feature> = selection
        .iter()
        .map(|s| s.feature.with_attributes(row_index_attributes(s.row)))
        .collect();

    // Content glbs are handed over as they are built, so peak memory stays at
    // one tile rather than the whole tileset; collect only their paths.
    let written = Mutex::new(Vec::new());
    let built = build(
        &features,
        options.metadata_options(),
        target_tile_size,
        options.render_options(),
        |relative_path, bytes| {
            let uri = destination
                .write_under(&relative_path, bytes)
                .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e}")))?;
            written
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(uri);
            Ok(())
        },
    )?;

    let mut written = written.into_inner().unwrap_or_else(PoisonError::into_inner);
    for (relative_path, bytes) in built.subtrees {
        written.push(destination.write_under(&relative_path, bytes)?);
    }
    let entry_point = destination.write_under("tileset.json", built.tileset_json.into_bytes())?;
    written.push(entry_point.clone());

    Ok(RenderedView {
        rendered_features: built.rendered_features,
        entry_point: Some(entry_point),
        written,
    })
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

/// 2D has no view yet, so say so instead of writing a file with nothing in it.
fn reject_two_dimensional(selection: &[Selected]) -> Result<()> {
    match selection
        .iter()
        .any(|s| holds_two_dimensional(&s.feature.geometry))
    {
        true => Err(Error::TwoDimensional),
        false => Ok(()),
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
    use reearth_flow_geometry::point::Point2D;
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry};

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

    fn feature_to(dir: &Path, selected: &Selected, options: &ViewOptions) -> RenderedView {
        let root = Uri::for_test(&format!("file://{}", dir.display()));
        let resolver = StorageResolver::new();
        render_feature(selected, options, &destination(&root, &resolver)).expect("render")
    }

    fn tileset_to(dir: &Path, selection: &[Selected], options: &ViewOptions) -> RenderedView {
        let root = Uri::for_test(&format!("file://{}", dir.display()));
        let resolver = StorageResolver::new();
        render_tileset(
            selection,
            1_048_576,
            options,
            &destination(&root, &resolver),
        )
        .expect("render")
    }

    fn destination<'a>(root: &'a Uri, resolver: &'a StorageResolver) -> Destination<'a> {
        Destination {
            root,
            prefix: "out",
            storage_resolver: resolver,
        }
    }

    fn glb_json(bytes: &[u8]) -> serde_json::Value {
        let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        serde_json::from_slice(&bytes[20..20 + len]).unwrap()
    }

    /// The compressed form reads like the plain one, is found under either
    /// extension, and a row selection decodes no further than it must.
    #[test]
    fn a_compressed_file_reads_by_row() {
        let dir = tempfile::tempdir().expect("temp dir");
        let features: Vec<Feature> = (0..4).map(|i| mesh_feature("k", 35.0 + i as f64)).collect();
        let lines: Vec<String> = features
            .iter()
            .map(|f| serde_json::to_string(f).expect("a feature serializes"))
            .collect();
        std::fs::write(
            dir.path().join("node.default.jsonl.zst"),
            zstd::stream::encode_all(lines.join("\n").as_bytes(), 0).expect("compress"),
        )
        .expect("write features");
        let resolver = StorageResolver::new();

        let compressed = Uri::for_test(&format!(
            "file://{}/node.default.jsonl.zst",
            dir.path().display()
        ));
        let loaded = load_selected(&compressed, Selection::Row(1), &resolver).expect("rows");
        assert_eq!(
            loaded.selection.iter().map(|s| s.row).collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(loaded.selection[0].feature.id, features[1].id);
        assert_eq!(loaded.scanned, 2, "the scan stops at the row it asked for");

        let plain = Uri::for_test(&format!(
            "file://{}/node.default.jsonl",
            dir.path().display()
        ));
        let by_plain_name = load_selected(&plain, Selection::Row(1), &resolver)
            .expect("the compressed file stands in for the plain name");
        assert_eq!(by_plain_name.selection[0].feature.id, features[1].id);
    }

    /// A feature the renderer cannot use is dropped, and the count says so
    /// rather than reporting everything selected.
    #[test]
    fn the_count_is_of_features_that_reached_the_tileset() {
        let dir = tempfile::tempdir().expect("temp dir");
        let selection = [
            Selected {
                row: 0,
                feature: Feature::from(Attributes::new()),
            },
            Selected {
                row: 1,
                feature: mesh_feature("k", 35.0),
            },
        ];
        let view = tileset_to(dir.path(), &selection, &ViewOptions::default());

        assert_eq!(view.rendered_features, 1);
    }

    /// 2D is not renderable yet, and saying so is an error rather than a panic.
    #[test]
    fn a_two_dimensional_selection_is_rejected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = Uri::for_test(&format!("file://{}", dir.path().display()));
        let resolver = StorageResolver::new();
        let selected = Selected {
            row: 0,
            feature: feature(
                "k",
                Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                    CoordinateFrame::Euclidean,
                    [0.0, 0.0],
                ))),
            ),
        };

        let error = render_feature(
            &selected,
            &ViewOptions::default(),
            &destination(&root, &resolver),
        )
        .expect_err("2D has no view");

        assert!(matches!(error, Error::TwoDimensional));
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

        let row = load_selected(&input, Selection::Row(3), &resolver).expect("row");
        assert_eq!(
            row.selection.iter().map(|s| s.row).collect::<Vec<_>>(),
            vec![3]
        );

        let filtered = load_selected(
            &input,
            Selection::Filter {
                expr: r#"attributes["kind"] == "keep""#,
                variables: Arc::new(serde_json::Map::new()),
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
        let selected = Selected {
            row: 4242,
            feature: mesh_feature("keep", 35.0),
        };
        feature_to(dir.path(), &selected, &ViewOptions::default());

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
        let selected = Selected {
            row: 0,
            feature: feature("k", collection),
        };
        // Draco packs positions away from the accessor, so read the count off an
        // uncompressed render.
        let options = ViewOptions {
            draco: false,
            ..ViewOptions::default()
        };
        feature_to(dir.path(), &selected, &options);

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
        let view = tileset_to(dir.path(), &selection, &ViewOptions::default());

        assert!(dir.path().join("out/tileset.json").exists());
        assert!(view
            .written
            .iter()
            .any(|uri| uri.as_str().contains("/out/content/")));
    }

    #[test]
    fn a_feature_without_geometry_writes_nothing() {
        let dir = tempfile::tempdir().expect("temp dir");
        let selected = Selected {
            row: 0,
            feature: Feature::from(Attributes::new()),
        };
        let view = feature_to(dir.path(), &selected, &ViewOptions::default());

        assert_eq!(view.rendered_features, 0);
        assert!(view.entry_point.is_none());
        assert!(view.written.is_empty());
    }
}
