/// The converter→writer seam. Shared and unconditional.
pub mod model;
pub mod writer;

// One module name, two files, so no call site needs a `cfg`.
#[cfg(not(feature = "new-geometry"))]
pub mod converter;
#[cfg(feature = "new-geometry")]
#[path = "citygml/converter_next.rs"]
pub mod converter;

// No legacy counterpart: the legacy reader flattens palettes itself.
#[cfg(feature = "new-geometry")]
mod appearance_next;

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
use reearth_flow_types::conversion::CrsCoverage;
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{CitygmlFeatureExt, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use model::{
    AppearanceBundle, BoundingEnvelope, CityObjectType, ConvertedCityObject, TextureRef,
    TextureSource,
};
use writer::CityGmlXmlWriter;

/// Write `features` as CityGML 2.0 to `output`, staging texture images alongside.
///
/// Shared by the `CityGmlWriter` sink and the `Feature Writer` processor.
///
/// Every feature is converted before the header is written: `srsName` and
/// `gml:boundedBy` are folded over the coordinates that actually reach the file,
/// at the cost of holding converted geometry alongside the buffered features.
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

    let mut converted: Vec<ConvertedCityObject> = Vec::with_capacity(features.len());
    let mut envelope: Option<BoundingEnvelope> = None;
    let mut crs = CrsCoverage::default();
    let mut textures: Vec<TextureRef> = Vec::new();
    let mut texture_keys: HashSet<String> = HashSet::new();

    for feature in features {
        let object = converter::convert_city_object(feature, lod_mask)?;

        if !object.omissions.is_empty() {
            let omitted = object
                .omissions
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ");
            tracing::warn!(
                feature_id = %feature.id,
                "CityGML Writer: {omitted}"
            );
        }

        if let Some(object_envelope) = &object.envelope {
            match &mut envelope {
                Some(existing) => existing.merge(object_envelope),
                None => envelope = Some(object_envelope.clone()),
            }
        }
        crs = crs.and(object.crs);
        for texture in &object.textures {
            if texture_keys.insert(texture.key.clone()) {
                textures.push(texture.clone());
            }
        }

        converted.push(object);
    }

    // Every feature was filtered out or held nothing writable: no document.
    if converted.iter().all(|object| object.geometries.is_empty()) {
        return Ok(());
    }

    let srs_name = converter::srs_name(features, epsg_code, crs)?;
    let uri_remap = stage_textures(
        &textures,
        output,
        sandbox_root,
        storage_resolver,
        converter::STRICT_TEXTURE_STAGING,
    )?;

    let buffer_size = (features.len() * 4096).clamp(32 * 1024, 512 * 1024);
    let mut xml_buffer = Vec::with_capacity(buffer_size);
    {
        let buf_writer = BufWriter::with_capacity(buffer_size, &mut xml_buffer);
        let mut xml_writer = CityGmlXmlWriter::new(buf_writer, pretty_print, srs_name);
        xml_writer.set_uri_remap(uri_remap);

        xml_writer.write_header(envelope.as_ref())?;

        for (feature, object) in features.iter().zip(&converted) {
            if object.geometries.is_empty() {
                continue;
            }

            let feature_type_str = feature
                .feature_type()
                .unwrap_or_else(|| "gen:GenericCityObject".to_string());
            let city_type = CityObjectType::from_feature_type(feature_type_str.as_str());

            let gml_id_str = feature
                .feature_id()
                .unwrap_or_else(|| feature.id.to_string());
            let appearance: Option<&AppearanceBundle> = if object.appearance.has_content() {
                Some(&object.appearance)
            } else {
                None
            };
            xml_writer.write_city_object(
                city_type,
                &object.geometries,
                Some(gml_id_str.as_str()),
                appearance,
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

/// Stage every referenced image into `{gml_stem}_appearance/` beside the GML file
/// and return the key → relative-path remap the writer rewrites `app:imageURI` with.
///
/// Destinations go through [`crate::SinkOutput`], so a hostile source URI cannot
/// escape the sandbox. Returned paths are relative to the GML file, which is what
/// `app:imageURI` means.
///
/// `textures` arrives deduplicated by [`TextureRef::key`]. What still needs
/// disambiguating is the destination *basename*: only a URI's last segment becomes
/// the file name, so `a/tex.png` and `b/tex.png` would both stage as `tex.png` and
/// the second would overwrite the first. The later one gets a numbered suffix.
///
/// `strict` is the compiled world's policy for an unstageable texture (see
/// `converter::STRICT_TEXTURE_STAGING`): the legacy path warns and leaves the
/// original URI in place, the unified path fails the write. A destination that
/// cannot be derived at all is fatal in both.
fn stage_textures(
    textures: &[TextureRef],
    output: &Uri,
    sandbox_root: &Uri,
    storage_resolver: &Arc<StorageResolver>,
    strict: bool,
) -> Result<HashMap<String, String>, SinkError> {
    let gml_stem = output
        .path()
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let appearance_dir_name = format!("{}_appearance", gml_stem);

    let sandbox_root_str = sandbox_root.as_str().trim_end_matches('/');
    let output_str = output.as_str();
    // `SinkOutput::new` builds `output` from `sandbox_root`, so a failed strip
    // means something upstream is broken. Fail rather than flatten the layout.
    let gml_rel_path: String = output_str
        .strip_prefix(sandbox_root_str)
        .map(|s| s.trim_start_matches('/').to_string())
        .ok_or_else(|| {
            SinkError::CityGmlWriter(format!(
                "output URI {output} is not under sandbox_root {sandbox_root_str}; \
                 refusing to fall back to a flat appearance directory"
            ))
        })?;
    let gml_rel_parent = gml_rel_path
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("");

    let mut uri_remap: HashMap<String, String> = HashMap::new();
    let mut staged_names: HashSet<String> = HashSet::new();
    for texture in textures {
        if uri_remap.contains_key(&texture.key) {
            continue;
        }
        let (filename, bytes) = match load_texture(texture, storage_resolver) {
            Ok(loaded) => loaded,
            Err(reason) => {
                report_texture_failure(strict, reason)?;
                continue;
            }
        };
        // Only a texture about to be written claims a name, so a skipped one
        // leaves the un-suffixed name free.
        let staged_name = unique_staged_name(&filename, &staged_names);
        let texture_rel_path = if gml_rel_parent.is_empty() {
            format!("{}/{}", appearance_dir_name, staged_name)
        } else {
            format!("{}/{}/{}", gml_rel_parent, appearance_dir_name, staged_name)
        };
        let dst_out =
            match crate::SinkOutput::new(sandbox_root, &texture_rel_path, storage_resolver) {
                Ok(o) => o,
                Err(e) => {
                    report_texture_failure(
                        strict,
                        format!(
                            "failed to acquire sandboxed SinkOutput for texture destination \
                             '{texture_rel_path}': {e}"
                        ),
                    )?;
                    continue;
                }
            };
        if let Err(e) = dst_out.write(bytes) {
            report_texture_failure(
                strict,
                format!("failed to write texture file '{texture_rel_path}': {e}"),
            )?;
            continue;
        }
        uri_remap.insert(
            texture.key.clone(),
            format!("{}/{}", appearance_dir_name, staged_name),
        );
        staged_names.insert(staged_name);
    }

    Ok(uri_remap)
}

/// The destination file name and bytes for one image, or why it could not be got.
///
/// A URI-backed raster keeps its source's last path segment; an in-memory one is
/// named after its content-hash key, filtered to file-name-safe characters so no
/// producer-invented key can steer a write out of the appearance directory.
fn load_texture(
    texture: &TextureRef,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<(String, Bytes), String> {
    match &texture.source {
        TextureSource::Uri(source) => {
            let src_str = source.to_string();
            let filename = source
                .path_segments()
                .and_then(|mut segments| segments.next_back())
                .filter(|name| !name.is_empty())
                .ok_or_else(|| {
                    format!("texture URI has no path segments, skipping copy: {src_str}")
                })?
                .to_string();
            let src_uri = Uri::from_str(&src_str)
                .map_err(|e| format!("failed to parse texture source URI '{src_str}': {e}"))?;
            let src_storage = storage_resolver.resolve(&src_uri).map_err(|e| {
                format!("failed to resolve storage for texture source '{src_str}': {e}")
            })?;
            let bytes = src_storage
                .get_sync(src_uri.path().as_path())
                .map_err(|e| format!("failed to read texture file '{src_str}': {e}"))?;
            Ok((filename, bytes))
        }
        TextureSource::InMemory { mime, bytes } => {
            let stem: String = texture
                .key
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if stem.is_empty() {
                return Err(format!(
                    "in-memory texture key '{}' yields no usable file name",
                    texture.key
                ));
            }
            Ok((
                format!("{stem}.{}", TextureSource::extension(*mime)),
                bytes.clone(),
            ))
        }
    }
}

/// Apply the compiled world's policy to one texture that could not be staged.
fn report_texture_failure(strict: bool, reason: String) -> Result<(), SinkError> {
    if strict {
        return Err(SinkError::CityGmlWriter(format!(
            "{reason}; the document would reference an image that was never written"
        )));
    }
    tracing::warn!("{reason}");
    Ok(())
}

/// `desired`, or the first free `{stem}_{n}{.ext}` variant. The suffix goes before
/// the extension so `mime_type_from_uri` can still sniff `app:mimeType`.
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
            .eval_string_variables_only(ctx.variables.clone())
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
}

impl Sink for CityGmlWriterSink {
    fn name(&self) -> &str {
        "CityGML Writer"
    }

    /// Buffering only; the envelope and CRS are folded during conversion in `finish`.
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        self.buffer.push(ctx.feature);
        Ok(())
    }

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

    /// `stage_textures` is shared, so this covers both worlds. Everything runs
    /// over `ram://`; a per-test resolver keeps the memory backends isolated.
    mod staging {
        use std::str::FromStr;
        use std::sync::Arc;

        use bytes::Bytes;
        use reearth_flow_common::image::MimeType;
        use reearth_flow_common::uri::Uri;
        use reearth_flow_storage::resolve::StorageResolver;
        use url::Url;

        use super::super::model::{TextureRef, TextureSource};
        use super::super::stage_textures;

        /// The legacy world's policy: a texture that cannot be staged warns and
        /// the write carries on.
        const LENIENT: bool = false;
        /// The unified world's policy: it aborts the write instead.
        const STRICT: bool = true;

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

        /// The manifest a conversion hands back for `uris`, in order — keyed on
        /// the source URI, which is what a URI-backed raster's key always is.
        fn manifest(uris: &[&str]) -> Vec<TextureRef> {
            uris.iter()
                .map(|uri| TextureRef {
                    key: uri.to_string(),
                    source: TextureSource::Uri(Url::parse(uri).unwrap()),
                })
                .collect()
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
            let textures = manifest(&["ram:///src/a/wall.png", "ram:///src/b/wall.png"]);

            let remap =
                stage_textures(&textures, &output, &sandbox_root, &resolver, LENIENT).unwrap();

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

        /// Dedup stays keyed on the source URI: one image referenced twice is
        /// staged once and claims one destination name.
        #[test]
        fn the_same_source_twice_stages_once() {
            let resolver = resolver();
            put(&resolver, "ram:///src/wall.png", b"only");

            let sandbox_root = Uri::from_str("ram:///jobs/dedup").unwrap();
            let output = Uri::from_str("ram:///jobs/dedup/city.gml").unwrap();
            let textures = manifest(&["ram:///src/wall.png", "ram:///src/wall.png"]);

            let remap =
                stage_textures(&textures, &output, &sandbox_root, &resolver, LENIENT).unwrap();

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
            let textures = manifest(&["data:image/png;base64,AAAA"]);

            let remap =
                stage_textures(&textures, &output, &sandbox_root, &resolver, LENIENT).unwrap();

            assert!(
                remap.is_empty(),
                "a skipped texture keeps its original app:imageURI; got {remap:?}"
            );
        }

        /// A raster that arrived as bytes has no URI to name it, so it is named
        /// after its key with the extension its MIME type fixes — one per value
        /// of the closed three-value enum.
        #[test]
        fn an_in_memory_raster_stages_under_its_mime_types_extension() {
            let resolver = resolver();
            let sandbox_root = Uri::from_str("ram:///jobs/inline").unwrap();
            let output = Uri::from_str("ram:///jobs/inline/city.gml").unwrap();
            let textures: Vec<TextureRef> = [
                (MimeType::ImageJpeg, "jpg", &b"jpeg-bytes"[..]),
                (MimeType::ImagePng, "png", &b"png-bytes"[..]),
                (MimeType::ImageWebp, "webp", &b"webp-bytes"[..]),
            ]
            .iter()
            .map(|(mime, ext, bytes)| TextureRef {
                key: format!("hash_{ext}"),
                source: TextureSource::InMemory {
                    mime: *mime,
                    bytes: Bytes::from_static(bytes),
                },
            })
            .collect();

            let remap = stage_textures(&textures, &output, &sandbox_root, &resolver, STRICT)
                .expect("in-memory rasters need no source to read");

            for (ext, bytes) in [
                ("jpg", &b"jpeg-bytes"[..]),
                ("png", &b"png-bytes"[..]),
                ("webp", &b"webp-bytes"[..]),
            ] {
                assert_eq!(
                    remap.get(&format!("hash_{ext}")).map(String::as_str),
                    Some(format!("city_appearance/hash_{ext}.{ext}").as_str())
                );
                assert_eq!(
                    read(
                        &resolver,
                        &format!("ram:///jobs/inline/city_appearance/hash_{ext}.{ext}")
                    ),
                    Bytes::copy_from_slice(bytes)
                );
            }
        }

        /// The two worlds' policies, on one input: a texture whose source does
        /// not exist. The legacy path leaves the original `app:imageURI` in the
        /// document and carries on, which is what it has always done.
        #[test]
        fn an_unreadable_texture_warns_in_the_legacy_path() {
            let resolver = resolver();
            let sandbox_root = Uri::from_str("ram:///jobs/missing").unwrap();
            let output = Uri::from_str("ram:///jobs/missing/city.gml").unwrap();
            let textures = manifest(&["ram:///src/absent.png"]);

            let remap = stage_textures(&textures, &output, &sandbox_root, &resolver, LENIENT)
                .expect("the legacy path tolerates an unreadable texture");

            assert!(remap.is_empty(), "nothing was staged; got {remap:?}");
        }

        /// The unified path fails the write instead, because it resolved the
        /// appearance itself and the document would name an image nobody wrote.
        #[test]
        fn an_unreadable_texture_fails_the_write_in_the_unified_path() {
            let resolver = resolver();
            let sandbox_root = Uri::from_str("ram:///jobs/missing-strict").unwrap();
            let output = Uri::from_str("ram:///jobs/missing-strict/city.gml").unwrap();
            let textures = manifest(&["ram:///src/absent.png"]);

            let message = stage_textures(&textures, &output, &sandbox_root, &resolver, STRICT)
                .unwrap_err()
                .to_string();

            assert!(message.contains("ram:///src/absent.png"), "{message}");
            assert!(message.contains("never written"), "{message}");
        }

        /// The staged path sits under the GML's directory while the remap stays
        /// GML-relative — what keeps `app:imageURI` resolvable across groups.
        #[test]
        fn staged_path_is_beside_the_gml_and_remap_stays_gml_relative() {
            let resolver = resolver();
            put(&resolver, "ram:///src/wall.png", b"bytes");

            let sandbox_root = Uri::from_str("ram:///jobs/nested").unwrap();
            let output = Uri::from_str("ram:///jobs/nested/group/city.gml").unwrap();
            let textures = manifest(&["ram:///src/wall.png"]);

            let remap =
                stage_textures(&textures, &output, &sandbox_root, &resolver, LENIENT).unwrap();

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
