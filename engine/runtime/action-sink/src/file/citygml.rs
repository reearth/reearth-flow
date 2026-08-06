pub mod converter;
/// The converter→writer seam. Shared and unconditional: it names no geometry
/// type, so both worlds' converters build it and one writer consumes it.
pub mod model;
pub mod writer;

use std::collections::{HashMap, HashSet};
use std::io::BufWriter;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::geometry::GeometryValue;
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{CitygmlFeatureExt, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use converter::{compute_envelope, convert_citygml_geometry};
use model::{AppearanceBundle, BoundingEnvelope, CityObjectType};
use writer::CityGmlXmlWriter;

/// Write `features` as CityGML 2.0 to `output`, copying texture images alongside it.
///
/// This is the single canonical implementation shared by both the `CityGmlWriter` sink and
/// the `Feature Writer` processor.
#[cfg(not(feature = "new-geometry"))]
pub fn write_citygml_to_storage(
    output: &Uri,
    sandbox_root: &Uri,
    features: &[Feature],
    lod_mask: &LodMask,
    epsg_code: Option<u32>,
    pretty_print: bool,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<(), SinkError> {
    if features.is_empty() {
        return Ok(());
    }

    let srs_name = epsg_code
        .or_else(|| {
            features
                .first()
                .and_then(|f| f.geometry.epsg)
                .map(|e| e as u32)
        })
        .map(|code| format!("http://www.opengis.net/def/crs/EPSG/0/{code}"))
        .unwrap_or_else(|| "http://www.opengis.net/def/crs/EPSG/0/4326".to_string());

    // Compute bounding envelope from all features.
    let mut envelope: Option<BoundingEnvelope> = None;
    for feature in features {
        if let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value {
            if let Some(env) = compute_envelope(geom) {
                match &mut envelope {
                    Some(existing) => existing.merge(&env),
                    None => envelope = Some(env),
                }
            }
        }
    }

    let uri_remap = stage_textures(features, output, sandbox_root, storage_resolver)?;

    // Build and write XML.
    let buffer_size = (features.len() * 4096).clamp(32 * 1024, 512 * 1024);
    let mut xml_buffer = Vec::with_capacity(buffer_size);
    {
        let buf_writer = BufWriter::with_capacity(buffer_size, &mut xml_buffer);
        let mut xml_writer = CityGmlXmlWriter::new(buf_writer, pretty_print, srs_name);
        xml_writer.set_uri_remap(uri_remap);

        xml_writer.write_header(envelope.as_ref())?;

        for feature in features {
            let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value else {
                continue;
            };

            let feature_type_str = feature
                .feature_type()
                .unwrap_or_else(|| "gen:GenericCityObject".to_string());
            let feature_type = feature_type_str.as_str();
            let city_type = CityObjectType::from_feature_type(feature_type);

            let (geometries, appearance) = convert_citygml_geometry(geom, lod_mask);
            if geometries.is_empty() {
                continue;
            }

            let gml_id_str = feature
                .feature_id()
                .unwrap_or_else(|| feature.id.to_string());
            let appearance_opt: Option<&AppearanceBundle> = if appearance.has_content() {
                Some(&appearance)
            } else {
                None
            };
            xml_writer.write_city_object(
                city_type,
                &geometries,
                Some(gml_id_str.as_str()),
                appearance_opt,
            )?;
        }

        xml_writer.write_footer()?;
    }

    let storage = storage_resolver
        .resolve(output)
        .map_err(SinkError::citygml_writer)?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(xml_buffer))
        .map_err(SinkError::citygml_writer)?;

    Ok(())
}

/// Stage every texture image referenced by `features` next to the GML file and
/// return the source-URI → relative-staged-path remap the writer rewrites
/// `app:imageURI` with.
///
/// Images land in a `{gml_stem}_appearance/` directory beside the GML file, and
/// every destination is acquired through [`crate::SinkOutput`] so a hostile
/// source URI cannot escape the sandbox. The returned paths are relative to the
/// GML file, not to the sandbox root, because that is what `app:imageURI` means.
///
/// Two keys are tracked, deliberately distinct:
/// - dedup is keyed on the **source** URI, so one image referenced by many
///   features is read and written exactly once;
/// - uniqueness is keyed on the **destination** basename, because only the last
///   path segment of a source URI becomes the file name. Two distinct sources
///   named `a/tex.png` and `b/tex.png` would otherwise both stage to
///   `{stem}_appearance/tex.png`, and since `SinkOutput::write` is a full
///   overwrite the second would silently replace the first. The later one gets
///   a numbered suffix (`tex_1.png`) instead.
///
/// A per-texture failure warns and continues, leaving that texture's original
/// absolute `app:imageURI` in the output; only a destination that cannot be
/// derived at all is fatal.
#[cfg(not(feature = "new-geometry"))]
fn stage_textures(
    features: &[Feature],
    output: &Uri,
    sandbox_root: &Uri,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<HashMap<String, String>, SinkError> {
    // Compute appearance directory name from GML output stem (e.g. "foo_appearance")
    let gml_stem = output
        .path()
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let appearance_dir_name = format!("{}_appearance", gml_stem);

    // Compute the GML output's relative path under sandbox_root by stripping the
    // sandbox_root prefix. This is used to derive the texture dst relative path.
    let sandbox_root_str = sandbox_root.as_str().trim_end_matches('/');
    let output_str = output.as_str();
    // `output` was produced by SinkOutput::new (sandbox_root.join(relative)),
    // so it must always start with sandbox_root. If the prefix strip ever fails,
    // something upstream is broken — fail loudly rather than silently writing
    // textures to a flat appearance dir, which would collide across groups
    // and corrupt data.
    let gml_rel_path: String = output_str
        .strip_prefix(sandbox_root_str)
        .map(|s| s.trim_start_matches('/').to_string())
        .ok_or_else(|| {
            SinkError::CityGmlWriter(format!(
                "output URI {output} is not under sandbox_root {sandbox_root_str}; \
                 refusing to fall back to a flat appearance directory"
            ))
        })?;
    // Parent directory of the GML's relative path (e.g. "group" or "" if at root)
    let gml_rel_parent = gml_rel_path
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("");

    // Copy texture images to the appearance dir and build a URI → relative-path remap.
    let mut uri_remap: HashMap<String, String> = HashMap::new();
    let mut staged_names: HashSet<String> = HashSet::new();
    for feature in features {
        let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value else {
            continue;
        };
        for texture in &geom.textures {
            let src_str = texture.uri.to_string();
            if uri_remap.contains_key(&src_str) {
                continue;
            }
            let filename = match texture.uri.path_segments().and_then(|mut s| s.next_back()) {
                Some(name) => name.to_string(),
                None => {
                    tracing::warn!(
                        "texture URI has no path segments, skipping copy: {}",
                        src_str
                    );
                    continue;
                }
            };
            let src_uri = match Uri::from_str(&src_str) {
                Ok(u) => u,
                Err(e) => {
                    tracing::warn!("failed to parse texture source URI '{}': {}", src_str, e);
                    continue;
                }
            };
            let src_storage = match storage_resolver.resolve(&src_uri) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        "failed to resolve storage for texture source '{}': {}",
                        src_str,
                        e
                    );
                    continue;
                }
            };
            let bytes = match src_storage.get_sync(src_uri.path().as_path()) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("failed to read texture file '{}': {}", src_str, e);
                    continue;
                }
            };
            // Only a texture that is about to be written claims a destination
            // name, so a skipped one leaves the un-suffixed name free.
            let staged_name = unique_staged_name(&filename, &staged_names);
            // Compute the texture destination as a relative path under sandbox_root.
            // e.g. "group/foo_appearance/bar.png" (or "foo_appearance/bar.png" at root)
            let texture_rel_path = if gml_rel_parent.is_empty() {
                format!("{}/{}", appearance_dir_name, staged_name)
            } else {
                format!("{}/{}/{}", gml_rel_parent, appearance_dir_name, staged_name)
            };
            let dst_out =
                match crate::SinkOutput::new(sandbox_root, &texture_rel_path, storage_resolver) {
                    Ok(o) => o,
                    Err(e) => {
                        tracing::warn!(
                        "failed to acquire sandboxed SinkOutput for texture destination '{}': {}",
                        texture_rel_path,
                        e
                    );
                        continue;
                    }
                };
            if let Err(e) = dst_out.write(bytes) {
                tracing::warn!("failed to write texture file '{}': {}", texture_rel_path, e);
                continue;
            }
            uri_remap.insert(src_str, format!("{}/{}", appearance_dir_name, staged_name));
            staged_names.insert(staged_name);
        }
    }

    Ok(uri_remap)
}

/// Return `desired` if no texture has been staged under that name yet, otherwise
/// the first free `{stem}_{n}{.ext}` variant.
///
/// The suffix goes before the extension so the staged file keeps the extension
/// `mime_type_from_uri` sniffs `app:mimeType` from. Iteration order of the copy
/// loop is buffer order then per-feature texture order, so the numbering is
/// deterministic for a given input.
#[cfg(not(feature = "new-geometry"))]
fn unique_staged_name(desired: &str, taken: &HashSet<String>) -> String {
    if !taken.contains(desired) {
        return desired.to_string();
    }
    let (stem, ext) = match desired.rsplit_once('.') {
        // A leading dot is part of the name (".gitignore"), not an extension.
        Some((stem, ext)) if !stem.is_empty() => (stem, Some(ext)),
        _ => (desired, None),
    };
    (1u32..)
        .map(|n| match ext {
            Some(ext) => format!("{stem}_{n}.{ext}"),
            None => format!("{stem}_{n}"),
        })
        .find(|candidate| !taken.contains(candidate))
        .expect("an unbounded counter always yields a free name")
}

#[derive(Debug, Clone, Default)]
pub struct CityGmlWriterFactory;

impl SinkFactory for CityGmlWriterFactory {
    fn name(&self) -> &str {
        "CityGML Writer"
    }

    fn description(&self) -> &str {
        "Writes features to CityGML 2.0 files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CityGmlWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["citygml", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: CityGmlWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::CityGmlWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::CityGmlWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::CityGmlWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let lod_mask = build_lod_mask(&params.lod_filter);
        let output = params
            .output
            .compile()
            .map_err(|e| {
                SinkError::CityGmlWriterFactory(format!("Failed to compile `output`: {e:?}"))
            })?
            .eval_string_env_only(ctx.env_vars.clone())
            .map_err(|e| {
                SinkError::CityGmlWriterFactory(format!("Failed to evaluate `output`: {e:?}"))
            })?;
        Ok(Box::new(CityGmlWriterSink {
            params: CityGmlWriterCompiledParam {
                output,
                epsg_code: params.epsg_code,
                pretty_print: params.pretty_print,
            },
            lod_mask,
            buffer: Vec::new(),
            envelope: None,
        }))
    }
}

fn build_lod_mask(lod_filter: &Option<Vec<u8>>) -> LodMask {
    match lod_filter {
        Some(lods) if !lods.is_empty() => {
            let mut mask = LodMask::default();
            for lod in lods {
                mask.add_lod(*lod);
            }
            mask
        }
        _ => LodMask::all(),
    }
}

/// # CityGmlWriter Parameters
///
/// Configuration for writing features to CityGML 2.0 files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CityGmlWriterParam {
    /// # Output File
    /// Output path or expression for the CityGML file to create.
    pub output: Code,
    /// # Pretty Print
    /// Whether to indent the output for readability. Defaults to true.
    #[serde(default = "default_pretty_print")]
    pub pretty_print: Option<bool>,
    /// # LOD Filter
    /// Levels of detail to include, such as [0, 1, 2]. If empty, all levels are included.
    #[serde(default)]
    pub lod_filter: Option<Vec<u8>>,
    /// # EPSG Code
    /// EPSG code of the coordinate reference system to declare in the output.
    #[serde(default)]
    pub epsg_code: Option<u32>,
}

fn default_pretty_print() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone)]
struct CityGmlWriterCompiledParam {
    output: String,
    epsg_code: Option<u32>,
    pretty_print: Option<bool>,
}

#[derive(Debug, Clone)]
struct CityGmlWriterSink {
    params: CityGmlWriterCompiledParam,
    lod_mask: LodMask,
    buffer: Vec<Feature>,
    envelope: Option<BoundingEnvelope>,
}

impl Sink for CityGmlWriterSink {
    fn name(&self) -> &str {
        "CityGML Writer"
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = ctx.feature;

        if let GeometryValue::CityGmlGeometry(ref geom) = feature.geometry.value {
            if let Some(env) = compute_envelope(geom) {
                match &mut self.envelope {
                    Some(existing) => existing.merge(&env),
                    None => self.envelope = Some(env),
                }
            }
        }

        self.buffer.push(feature);
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let path = self.params.output.as_str();
        let out = crate::SinkOutput::new(&ctx.sandbox_root, path, &ctx.storage_resolver)
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        write_citygml_to_storage(
            out.uri(),
            &ctx.sandbox_root,
            &self.buffer,
            &self.lod_mask,
            self.params.epsg_code,
            self.params.pretty_print.unwrap_or(true),
            &ctx.storage_resolver,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod sandbox_tests {
    use reearth_flow_common::uri::Uri;
    use reearth_flow_storage::resolve::StorageResolver;
    use std::str::FromStr;
    use tempfile::tempdir;

    /// Texture dst paths outside the configured sandbox_root must be rejected
    /// by SinkOutput::new — this catches any regression that would
    /// reintroduce the direct dst_storage.put_sync bypass.
    #[test]
    fn texture_dst_write_outside_sandbox_root_is_rejected() {
        let tmp = tempdir().unwrap();
        let inside = tmp.path().join("inside");
        std::fs::create_dir(&inside).unwrap();

        let sandbox_root = Uri::from_str(&format!("file://{}", inside.display())).unwrap();
        let resolver = StorageResolver::new();

        // A traversal path that would escape sandbox_root must be rejected by SinkOutput::new.
        let result = crate::SinkOutput::new(&sandbox_root, "../../outside/texture.png", &resolver);
        assert!(
            result.is_err(),
            "Texture dst outside sandbox root must be rejected; got: {:?}",
            result.ok().map(|s| s.uri().clone())
        );
    }

    /// Coverage for `stage_textures`, which only exists in the legacy build:
    /// it reads `GeometryValue::CityGmlGeometry`, a type the unified build's
    /// `Feature` cannot hold.
    ///
    /// Everything runs over `ram://` storage, so no test touches the filesystem.
    /// Each test builds its own `StorageResolver`, and an OpenDAL memory backend
    /// is per-operator, so the in-memory stores are isolated from each other.
    #[cfg(not(feature = "new-geometry"))]
    mod staging {
        use std::str::FromStr;
        use std::sync::Arc;

        use bytes::Bytes;
        use reearth_flow_common::uri::Uri;
        use reearth_flow_storage::resolve::StorageResolver;
        use reearth_flow_types::geometry::{CityGmlGeometry, Geometry, GeometryValue};
        use reearth_flow_types::material::Texture;
        use reearth_flow_types::Feature;
        use url::Url;

        use super::super::stage_textures;

        fn resolver() -> Arc<StorageResolver> {
            Arc::new(StorageResolver::new())
        }

        fn put(resolver: &StorageResolver, uri: &str, bytes: &'static [u8]) {
            let uri = Uri::from_str(uri).unwrap();
            resolver
                .resolve(&uri)
                .unwrap()
                .put_sync(uri.path().as_path(), Bytes::from_static(bytes))
                .unwrap();
        }

        fn read(resolver: &StorageResolver, uri: &str) -> Bytes {
            let uri = Uri::from_str(uri).unwrap();
            resolver
                .resolve(&uri)
                .unwrap()
                .get_sync(uri.path().as_path())
                .unwrap_or_else(|e| panic!("expected a staged file at {uri}: {e}"))
        }

        fn exists(resolver: &StorageResolver, uri: &str) -> bool {
            let uri = Uri::from_str(uri).unwrap();
            resolver
                .resolve(&uri)
                .unwrap()
                .get_sync(uri.path().as_path())
                .is_ok()
        }

        /// One feature whose CityGML geometry references `uris`, in order.
        fn textured_feature(uris: &[&str]) -> Feature {
            let textures = uris
                .iter()
                .map(|uri| Texture {
                    uri: Url::parse(uri).unwrap(),
                })
                .collect();
            let geometry = CityGmlGeometry::new(Vec::new(), Vec::new(), textures);
            Feature::from(Geometry::with_value(GeometryValue::CityGmlGeometry(
                geometry,
            )))
        }

        /// The collision regression: dedup is by source URI, so two distinct
        /// sources sharing a basename used to stage to one destination and the
        /// second silently overwrote the first.
        #[test]
        fn two_sources_sharing_a_basename_stage_to_distinct_files() {
            let resolver = resolver();
            put(&resolver, "ram:///src/a/wall.png", b"first");
            put(&resolver, "ram:///src/b/wall.png", b"second");

            let sandbox_root = Uri::from_str("ram:///jobs/collision").unwrap();
            let output = Uri::from_str("ram:///jobs/collision/city.gml").unwrap();
            let features = vec![textured_feature(&[
                "ram:///src/a/wall.png",
                "ram:///src/b/wall.png",
            ])];

            let remap = stage_textures(&features, &output, &sandbox_root, &resolver).unwrap();

            assert_eq!(
                remap.get("ram:///src/a/wall.png").map(String::as_str),
                Some("city_appearance/wall.png")
            );
            assert_eq!(
                remap.get("ram:///src/b/wall.png").map(String::as_str),
                Some("city_appearance/wall_1.png")
            );
            assert_eq!(
                read(&resolver, "ram:///jobs/collision/city_appearance/wall.png"),
                Bytes::from_static(b"first")
            );
            assert_eq!(
                read(
                    &resolver,
                    "ram:///jobs/collision/city_appearance/wall_1.png"
                ),
                Bytes::from_static(b"second")
            );
        }

        /// Dedup stays keyed on the source URI: one image referenced twice — here
        /// from two features — is staged once and claims one destination name.
        #[test]
        fn the_same_source_twice_stages_once() {
            let resolver = resolver();
            put(&resolver, "ram:///src/wall.png", b"only");

            let sandbox_root = Uri::from_str("ram:///jobs/dedup").unwrap();
            let output = Uri::from_str("ram:///jobs/dedup/city.gml").unwrap();
            let features = vec![
                textured_feature(&["ram:///src/wall.png"]),
                textured_feature(&["ram:///src/wall.png"]),
            ];

            let remap = stage_textures(&features, &output, &sandbox_root, &resolver).unwrap();

            assert_eq!(remap.len(), 1, "one source URI, one remap entry");
            assert_eq!(
                remap.get("ram:///src/wall.png").map(String::as_str),
                Some("city_appearance/wall.png")
            );
            assert!(
                !exists(&resolver, "ram:///jobs/dedup/city_appearance/wall_1.png"),
                "a repeated source must not claim a second destination name"
            );
        }

        /// A URI with no path segments yields no file name to stage under, so it
        /// is warned about and skipped rather than aborting the whole write.
        #[test]
        fn segment_less_uri_is_skipped() {
            let resolver = resolver();
            let sandbox_root = Uri::from_str("ram:///jobs/segmentless").unwrap();
            let output = Uri::from_str("ram:///jobs/segmentless/city.gml").unwrap();
            let features = vec![textured_feature(&["data:image/png;base64,AAAA"])];

            let remap = stage_textures(&features, &output, &sandbox_root, &resolver).unwrap();

            assert!(
                remap.is_empty(),
                "a skipped texture keeps its original app:imageURI; got {remap:?}"
            );
        }

        /// The staged path is physically under the GML's own directory, while the
        /// remap value stays relative to the GML file — that asymmetry is what
        /// keeps `app:imageURI` resolvable and appearance dirs from colliding
        /// across output groups.
        #[test]
        fn staged_path_is_beside_the_gml_and_remap_stays_gml_relative() {
            let resolver = resolver();
            put(&resolver, "ram:///src/wall.png", b"bytes");

            let sandbox_root = Uri::from_str("ram:///jobs/nested").unwrap();
            let output = Uri::from_str("ram:///jobs/nested/group/city.gml").unwrap();
            let features = vec![textured_feature(&["ram:///src/wall.png"])];

            let remap = stage_textures(&features, &output, &sandbox_root, &resolver).unwrap();

            assert_eq!(
                remap.get("ram:///src/wall.png").map(String::as_str),
                Some("city_appearance/wall.png")
            );
            assert_eq!(
                read(
                    &resolver,
                    "ram:///jobs/nested/group/city_appearance/wall.png"
                ),
                Bytes::from_static(b"bytes")
            );
        }
    }
}
