pub mod attribute_keys;
pub mod classes;
pub mod generic_attributes;
/// The converter→writer seam. Shared and unconditional.
pub mod model;
pub mod placement;
pub mod properties;
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
use reearth_flow_citygml::schema::{AdeSource, SchemaSet};
use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::ErrorCode;
use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::conversion::CrsCoverage;
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{AttributeValue, Attributes, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use attribute_keys::{AttributeKeys, ResolvedKeys};
use classes::{CityClass, Prefixes};
use generic_attributes::GenericAttribute;
use model::{AppearanceBundle, BoundingEnvelope, ConvertedCityObject, TextureRef, TextureSource};
use placement::Placed;
use properties::XmlProperty;
use writer::CityGmlXmlWriter;

/// What a writable feature is written as: its class, its source `gml:id` and
/// its placed properties.
type Identity = (CityClass, Option<String>, Vec<Placed>);

/// Write `features` as CityGML 2.0 to `output`, with their CityGML properties
/// and the `generic` computed values, staging the texture images and codelists
/// the document references alongside.
///
/// Shared by the `CityGmlWriter` sink and the `Feature Writer` processor.
///
/// Every feature is converted before the header is written: `srsName` and
/// `gml:boundedBy` are folded over the coordinates that actually reach the file,
/// at the cost of holding converted geometry alongside the buffered features.
///
/// Classes resolve against `schema` through `prefixes`, and properties are
/// built and placed through the same two; `ade_locations` pairs
/// each ADE namespace with the location the output declares for it. A writer
/// with no ADEs passes `SchemaSet::core()`, `Prefixes::citygml2()` and none.
#[allow(clippy::too_many_arguments)]
pub fn write_citygml_to_storage(
    output: &Uri,
    sandbox_root: &Uri,
    features: &[Feature],
    lod_mask: &LodMask,
    keys: &ResolvedKeys,
    generic: &[GenericAttribute],
    epsg_code: Option<u32>,
    pretty_print: bool,
    storage_resolver: &Arc<StorageResolver>,
    diagnostics: Option<&NodeDiagnosticsHandle>,
    schema: &SchemaSet,
    prefixes: &Prefixes,
    ade_locations: &[(String, String)],
) -> Result<(), SinkError> {
    if features.is_empty() {
        return Ok(());
    }

    // The writer's own inputs are top-level pipeline data, not properties, and a
    // top-level source of a computed value is written once, as that value. A map
    // named by `cityGmlAttributes` holds only the source's properties, so none of
    // those top-level names may remove anything from it.
    let excluded: HashSet<&str> = match keys.city_gml_attributes {
        Some(_) => HashSet::new(),
        None => [
            keys.feature_type.as_str(),
            keys.gml_id.as_str(),
            keys.lod.as_str(),
            keys.gml_property_name.as_str(),
        ]
        .into_iter()
        .chain(generic.iter().map(|entry| entry.attribute.as_str()))
        .collect(),
    };

    let mut converted: Vec<ConvertedCityObject> = Vec::with_capacity(features.len());
    // `None` for a feature with no geometry, which is not written.
    let mut objects: Vec<Option<Identity>> = Vec::with_capacity(features.len());
    let mut classes = ClassResolver::new(prefixes, schema);
    let mut envelope: Option<BoundingEnvelope> = None;
    let mut crs = CrsCoverage::default();
    let mut textures: Vec<TextureRef> = Vec::new();
    let mut texture_keys: HashSet<String> = HashSet::new();
    let mut code_spaces: Vec<String> = Vec::new();

    for feature in features {
        let object = converter::convert_city_object(feature, lod_mask, keys, diagnostics)?;

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

        // A feature with no geometry is not written, so nothing of it is
        // resolved, placed, reported or staged.
        let identity = (!object.geometries.is_empty()).then(|| {
            // Only a source `gml:id` is offered as a candidate. The engine's own
            // feature id is a per-run UUID, so it would make the output
            // nondeterministic, and a minted id would be indistinguishable from
            // a real one like `bldg_<uuid>`, which is precisely the shape PLATEAU
            // uses. `claim_gml_id` mints a stable `<prefix>_<n>` instead.
            let (class, gml_id) = city_object_identity(feature, keys, &mut classes, diagnostics);
            let placed = place_properties(
                feature,
                &class,
                keys,
                generic,
                &excluded,
                schema,
                prefixes,
                diagnostics,
                &mut code_spaces,
            );
            (class, gml_id, placed)
        });

        converted.push(object);
        objects.push(identity);
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
    let code_space_remap = stage_codelists(
        &code_spaces,
        output,
        sandbox_root,
        storage_resolver,
        diagnostics,
    )?;
    for (_, _, placed) in objects.iter_mut().flatten() {
        for placed in placed {
            remap_code_spaces(&mut placed.property, &code_space_remap);
        }
    }

    let buffer_size = (features.len() * 4096).clamp(32 * 1024, 512 * 1024);
    let mut xml_buffer = Vec::with_capacity(buffer_size);
    {
        let buf_writer = BufWriter::with_capacity(buffer_size, &mut xml_buffer);
        let mut xml_writer = CityGmlXmlWriter::new(buf_writer, pretty_print, srs_name);
        xml_writer.set_uri_remap(uri_remap);
        xml_writer.set_schema(schema, prefixes.clone());

        xml_writer.write_header(envelope.as_ref(), ade_locations)?;

        for ((feature, object), identity) in features.iter().zip(converted).zip(objects) {
            // Drops are reported by the world's own converter, which is the
            // only side that can tell an unusable geometry from an empty one.
            let Some((class, gml_id, placed)) = identity else {
                continue;
            };

            let appearance: Option<&AppearanceBundle> = if object.appearance.has_content() {
                Some(&object.appearance)
            } else {
                None
            };
            let dropped_before = xml_writer.dropped_lod1_surfaces();
            xml_writer.write_city_object(
                &class,
                object.geometries,
                placed,
                gml_id.as_deref(),
                appearance,
            )?;
            // The XSD permits both `lod1Solid` and `lod1MultiSurface`, so the
            // schema gate cannot see this one and it is reported here instead of
            // dropped in silence.
            if xml_writer.dropped_lod1_surfaces() > dropped_before {
                if let Some(diagnostics) = diagnostics {
                    diagnostics.report_drop(
                        ErrorCode::CitygmlLod1ShellConflict,
                        Some(feature.id),
                        Some(true),
                    );
                }
            }
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

/// One feature's properties placed in `class`'s content model, with every skip
/// reported against the feature. Codelist URLs the placed tree references are
/// added to `code_spaces`, first seen first.
///
/// Properties are built and placed through the writer's own `schema` and
/// `prefixes`, so an ADE listed in `adeSchemas` makes its properties placeable
/// wherever its schema allows them.
#[allow(clippy::too_many_arguments)]
fn place_properties(
    feature: &Feature,
    class: &CityClass,
    keys: &ResolvedKeys,
    generic: &[GenericAttribute],
    excluded: &HashSet<&str>,
    schema: &SchemaSet,
    prefixes: &Prefixes,
    diagnostics: Option<&NodeDiagnosticsHandle>,
    code_spaces: &mut Vec<String>,
) -> Vec<Placed> {
    let no_attributes = Attributes::new();
    let source = match &keys.city_gml_attributes {
        None => feature.attributes.as_ref(),
        // A configured map that is missing, or is not a map, holds no properties.
        Some(key) => match feature.get(key) {
            Some(AttributeValue::Map(map)) => map,
            _ => &no_attributes,
        },
    };
    let converted = properties::from_attributes(source, excluded, prefixes);
    // Computed values are named by the feature's own attributes, wherever the
    // CityGML properties come from.
    let (computed, computed_skips) =
        generic_attributes::to_properties(&feature.attributes, generic);
    let mut properties = converted.properties;
    generic_attributes::merge(&mut properties, computed);
    let (placed, placement_skips) = placement::place(schema, prefixes, &class.element, properties);

    // The run summary counts each report as one affected feature, so a feature
    // is reported once per code however many of its attributes were left out.
    if let Some(diagnostics) = diagnostics {
        let codes: HashSet<ErrorCode> = converted
            .skipped
            .iter()
            .chain(&computed_skips)
            .chain(&placement_skips)
            .map(|skip| skip.code())
            .collect();
        for code in codes {
            diagnostics.report_warn(code, Some(feature.id));
        }
    }
    for placed in &placed {
        collect_code_spaces(&placed.property, &converted.code_spaces, code_spaces);
    }
    placed
}

/// Add each `codeSpace` in `property`'s tree that is one of `stageable` to
/// `out`, once. A property left out by placement never gets here, so its list
/// is not copied for nothing.
fn collect_code_spaces(property: &XmlProperty, stageable: &[String], out: &mut Vec<String>) {
    if let Some(url) = property.attr("codeSpace") {
        if stageable.iter().any(|candidate| candidate == url) && !out.iter().any(|seen| seen == url)
        {
            out.push(url.to_owned());
        }
    }
    for child in &property.children {
        collect_code_spaces(child, stageable, out);
    }
}

/// Point every staged `codeSpace` in `property`'s tree at its copy.
fn remap_code_spaces(property: &mut XmlProperty, remap: &HashMap<String, String>) {
    for (name, value) in &mut property.attrs {
        if name == "codeSpace" {
            if let Some(staged) = remap.get(value.as_str()) {
                value.clone_from(staged);
            }
        }
    }
    for child in &mut property.children {
        remap_code_spaces(child, remap);
    }
}

/// A directory files are staged into beside the GML file.
struct StagingDir {
    /// `{gml_stem}_{suffix}`, the GML-relative prefix the document names
    /// staged files by.
    name: String,
    /// The same directory relative to the sandbox root, which is where it is
    /// written.
    path: String,
}

fn staging_dir(output: &Uri, sandbox_root: &Uri, suffix: &str) -> Result<StagingDir, SinkError> {
    let gml_stem = output
        .path()
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = format!("{gml_stem}_{suffix}");

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
                 refusing to fall back to a flat {suffix} directory"
            ))
        })?;
    let path = match gml_rel_path.rsplit_once('/') {
        Some((parent, _)) if !parent.is_empty() => format!("{parent}/{name}"),
        _ => name.clone(),
    };
    Ok(StagingDir { name, path })
}

/// Copy every codelist in `code_spaces` into `{gml_stem}_codelists/` beside the
/// GML file and return the URL → relative-path remap for `codeSpace`.
///
/// A per-output directory keeps two outputs in one folder from overwriting each
/// other's same-named lists, and clashing basenames get a numbered suffix as
/// textures do. Unlike a texture, a codelist that cannot be copied never fails
/// the write in either world: its absolute URL is kept, which still names the
/// list for anything that can reach it, and the miss is reported.
fn stage_codelists(
    code_spaces: &[String],
    output: &Uri,
    sandbox_root: &Uri,
    storage_resolver: &Arc<StorageResolver>,
    diagnostics: Option<&NodeDiagnosticsHandle>,
) -> Result<HashMap<String, String>, SinkError> {
    let dir = staging_dir(output, sandbox_root, "codelists")?;
    let mut remap: HashMap<String, String> = HashMap::new();
    let mut staged_names: HashSet<String> = HashSet::new();
    for url in code_spaces {
        let staged = load_codelist(url, storage_resolver).and_then(|(filename, bytes)| {
            let staged_name = unique_staged_name(&filename, &staged_names);
            let rel_path = format!("{}/{}", dir.path, staged_name);
            crate::SinkOutput::new(sandbox_root, &rel_path, storage_resolver)
                .map_err(|e| {
                    format!(
                        "failed to acquire sandboxed SinkOutput for codelist destination \
                         '{rel_path}': {e}"
                    )
                })?
                .write(bytes)
                .map_err(|e| format!("failed to write codelist file '{rel_path}': {e}"))?;
            Ok(staged_name)
        });
        match staged {
            Ok(staged_name) => {
                remap.insert(url.clone(), format!("{}/{}", dir.name, staged_name));
                staged_names.insert(staged_name);
            }
            Err(reason) => {
                warn_not_staged(&reason);
                if let Some(diagnostics) = diagnostics {
                    diagnostics.report_warn(ErrorCode::CitygmlCodelistNotStaged, None);
                }
            }
        }
    }
    Ok(remap)
}

/// A codelist's file name, its URL's last path segment, and its bytes.
fn load_codelist(
    url: &str,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<(String, Bytes), String> {
    let uri =
        Uri::from_str(url).map_err(|e| format!("failed to parse codelist URL '{url}': {e}"))?;
    let filename = uri
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| format!("codelist URL has no file name, skipping copy: {url}"))?;
    let storage = storage_resolver
        .resolve(&uri)
        .map_err(|e| format!("failed to resolve storage for codelist '{url}': {e}"))?;
    let bytes = storage
        .get_sync(uri.path().as_path())
        .map_err(|e| format!("failed to read codelist file '{url}': {e}"))?;
    Ok((filename, bytes))
}

/// The embedded CityGML 2.0 schemas, which a write with no ADEs resolves against.
pub fn core_schema() -> Result<&'static SchemaSet, SinkError> {
    SchemaSet::core().map_err(|e| {
        SinkError::CityGmlWriter(format!("the CityGML 2.0 schemas failed to load: {e}"))
    })
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
    let dir = staging_dir(output, sandbox_root, "appearance")?;

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
        let texture_rel_path = format!("{}/{}", dir.path, staged_name);
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
        uri_remap.insert(texture.key.clone(), format!("{}/{}", dir.name, staged_name));
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
    warn_not_staged(&reason);
    Ok(())
}

/// Log why a file the document references was not staged. Diagnostic messages
/// are discarded in production, so this is the only place the reason survives.
fn warn_not_staged(reason: &str) {
    tracing::warn!("{reason}");
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
        "Writes features to CityGML 2.0 files, with the CityGML properties they were read with placed in schema order. Properties of an ADE such as uro: are written when the ADE is listed in ADE Schemas and reported otherwise. Nested city objects such as boundary surfaces are reported rather than written."
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
        let keys = params
            .attribute_keys
            .clone()
            .unwrap_or_default()
            .resolve()
            .map_err(SinkError::CityGmlWriterFactory)?;
        let generic_attributes = params.generic_attributes.clone().unwrap_or_default();
        generic_attributes::validate(&generic_attributes)
            .map_err(SinkError::CityGmlWriterFactory)?;
        let ade_schemas = params.ade_schemas.clone().unwrap_or_default();
        let ade = bind_ade_schemas(&ade_schemas, &ctx).map_err(SinkError::CityGmlWriterFactory)?;
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
                ade,
            },
            lod_mask,
            keys,
            generic_attributes,
            buffer: Vec::new(),
        }))
    }
}

/// Evaluate each entry's `schemaLocation`, check the entries and normalize them.
fn bind_ade_schemas(entries: &[AdeSchema], ctx: &NodeContext) -> Result<Vec<AdeBinding>, String> {
    let locations = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            entry
                .schema_location
                .compile()
                .map_err(|e| format!("Failed to compile `adeSchemas[{i}].schemaLocation`: {e}"))?
                .eval_string_variables_only(ctx.variables.clone())
                .map_err(|e| format!("Failed to evaluate `adeSchemas[{i}].schemaLocation`: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_ade_schemas(entries, &locations)?;
    Ok(entries
        .iter()
        .zip(&locations)
        .map(|(entry, location)| AdeBinding::new(entry, location))
        .collect())
}

/// The storage schemes an ADE schema may be read from.
const ADE_SCHEMA_SCHEMES: [&str; 3] = ["file", "gs", "ram"];

/// Check `adeSchemas`, given each entry's evaluated `schemaLocation`, before
/// anything is read, naming the offending field.
///
/// `schemaLocation` must be an absolute URL, so a relative path never resolves
/// against whatever directory the worker happens to run in, and its scheme must
/// be one of [`ADE_SCHEMA_SCHEMES`]. In particular it must not be `http`/`https`,
/// because the writer never fetches from the network.
fn validate_ade_schemas(entries: &[AdeSchema], locations: &[String]) -> Result<(), String> {
    let built_in = Prefixes::citygml2();
    let mut prefixes: HashSet<&str> = HashSet::new();
    let mut namespaces: HashSet<&str> = HashSet::new();
    for (i, (entry, schema_location)) in entries.iter().zip(locations).enumerate() {
        let field = |name: &str| format!("`adeSchemas[{i}].{name}`");
        for (name, value) in [
            ("prefix", &entry.prefix),
            ("namespace", &entry.namespace),
            ("schemaLocation", schema_location),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{} is blank", field(name)));
            }
        }
        // Both are written into `xsi:schemaLocation`, a whitespace-separated list
        // of pairs, so whitespace inside either would shift every pair after it.
        for (name, value) in [
            ("namespace", Some(&entry.namespace)),
            ("declaredLocation", entry.declared_location.as_ref()),
        ] {
            if value.is_some_and(|value| {
                !value.trim().is_empty() && value.chars().any(char::is_whitespace)
            }) {
                return Err(format!(
                    "{} must not contain whitespace; encode a space as `%20`",
                    field(name)
                ));
            }
        }
        if !writer::is_ncname(&entry.prefix) {
            return Err(format!(
                "{} `{}` is not a valid XML prefix; use letters, digits, `_`, `-` and `.`, starting with a letter or `_`",
                field("prefix"),
                entry.prefix
            ));
        }
        if built_in.namespace(&entry.prefix).is_some() {
            return Err(format!(
                "{} `{}` is reserved by the writer; choose another",
                field("prefix"),
                entry.prefix
            ));
        }
        // Reserved by XML Namespaces, so binding one would make the output malformed.
        if matches!(entry.prefix.as_str(), "xml" | "xmlns") {
            return Err(format!(
                "{} `{}` is reserved by XML; choose another",
                field("prefix"),
                entry.prefix
            ));
        }
        if !prefixes.insert(entry.prefix.as_str()) {
            return Err(format!(
                "{} `{}` is used by an earlier entry",
                field("prefix"),
                entry.prefix
            ));
        }
        if !namespaces.insert(entry.namespace.as_str()) {
            return Err(format!(
                "{} `{}` is used by an earlier entry",
                field("namespace"),
                entry.namespace
            ));
        }
        let location = url::Url::parse(schema_location).map_err(|e| {
            format!(
                "{} `{schema_location}` is not an absolute URL ({e}); give a `file://` or `gs://` path",
                field("schemaLocation"),
            )
        })?;
        if !ADE_SCHEMA_SCHEMES.contains(&location.scheme()) {
            return Err(format!(
                "{} `{schema_location}` uses the `{}` scheme; the writer reads schemas only from {} (it never fetches from the network), so give a copy of the XSD there and put any public URL in `declaredLocation`",
                field("schemaLocation"),
                location.scheme(),
                ADE_SCHEMA_SCHEMES.map(|scheme| format!("`{scheme}://`")).join(", "),
            ));
        }
        if entry
            .declared_location
            .as_ref()
            .is_some_and(|declared| declared.trim().is_empty())
        {
            return Err(format!(
                "{} is blank; give a location or leave it out to use `schemaLocation`",
                field("declaredLocation")
            ));
        }
    }
    Ok(())
}

/// Resolves feature-type values to classes, once per distinct value in a write.
struct ClassResolver<'a> {
    prefixes: &'a Prefixes,
    schema: &'a SchemaSet,
    resolved: HashMap<String, Option<CityClass>>,
}

impl<'a> ClassResolver<'a> {
    fn new(prefixes: &'a Prefixes, schema: &'a SchemaSet) -> Self {
        Self {
            prefixes,
            schema,
            resolved: HashMap::new(),
        }
    }

    /// `None` when `value` names no class.
    fn class_of(&mut self, value: &str) -> Option<CityClass> {
        if let Some(class) = self.resolved.get(value) {
            return class.clone();
        }
        let class = classes::resolve(value, self.prefixes, self.schema);
        self.resolved.insert(value.to_string(), class.clone());
        class
    }
}

/// The CityGML class and the source `gml:id` a feature records under the
/// configured keys. An unresolved or missing class becomes
/// `gen:GenericCityObject`; a missing id is minted by the writer.
///
/// Only an unresolved value is reported. A missing or non-text one claims no
/// class, so the fallback is what was asked for.
fn city_object_identity(
    feature: &Feature,
    keys: &ResolvedKeys,
    classes: &mut ClassResolver,
    diagnostics: Option<&NodeDiagnosticsHandle>,
) -> (CityClass, Option<String>) {
    let class = match feature
        .get(keys.feature_type.as_str())
        .and_then(|value| value.as_string())
    {
        None => CityClass::generic(),
        Some(value) => classes.class_of(&value).unwrap_or_else(|| {
            if let Some(diagnostics) = diagnostics {
                diagnostics.report_warn(ErrorCode::CitygmlUnknownFeatureType, Some(feature.id));
            }
            CityClass::generic()
        }),
    };
    let gml_id = feature
        .get(keys.gml_id.as_str())
        .and_then(|value| value.as_string());
    (class, gml_id)
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
    /// # Attribute Keys
    /// Names of the attributes the writer reads its CityGML inputs from, for data that did not come from a CityGML reader. Any key left out uses the reader's name.
    #[serde(default)]
    pub attribute_keys: Option<AttributeKeys>,
    /// # Generic Attributes
    /// Computed values to write as CityGML generic attributes (`gen:`), each naming the feature attribute to read. Attributes read from a CityGML source are written back on their own and need no entry here.
    #[serde(default)]
    pub generic_attributes: Option<Vec<GenericAttribute>>,
    /// # ADE Schemas
    /// Application Domain Extensions (ADEs) to write classes from, such as PLATEAU i-UR. Each entry names the prefix used in feature types and attributes (for example `uro`), its namespace, and where to read its XSD. A feature type such as `uro:UndergroundBuilding` is written as itself when its ADE is listed here.
    #[serde(default)]
    pub ade_schemas: Option<Vec<AdeSchema>>,
}

/// # ADE Schema
/// An Application Domain Extension the writer can write classes from.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdeSchema {
    /// # Prefix
    /// Prefix for the ADE, such as `uro`. It cannot reuse a CityGML 2.0 prefix.
    pub prefix: String,
    /// # Namespace
    /// The ADE's namespace, as declared by its XSD's `targetNamespace`.
    pub namespace: String,
    /// # Schema Location
    /// Path or expression for where to read the ADE's XSD, as a `file://` or `gs://` path. Schemas it imports by relative path are read from beside it. Imports of the OGC schemas come from the copy built into Flow.
    pub schema_location: Code,
    /// # Declared Location
    /// Schema location written into the output's `xsi:schemaLocation` for this namespace, such as the official URL or a path relative to the output. Defaults to `schemaLocation`.
    #[serde(default)]
    pub declared_location: Option<String>,
}

fn default_pretty_print() -> Option<bool> {
    Some(true)
}

/// An `adeSchemas` entry once validated, with its location normalized.
#[derive(Debug, Clone)]
struct AdeBinding {
    prefix: String,
    namespace: String,
    location: String,
    declared_location: String,
}

impl AdeBinding {
    /// Only for an entry `validate_ade_schemas` accepted, with its evaluated
    /// `schema_location`, which therefore parses.
    fn new(entry: &AdeSchema, schema_location: &str) -> Self {
        let location = url::Url::parse(schema_location)
            .map(|url| url.to_string())
            .unwrap_or_else(|_| schema_location.to_string());
        Self {
            prefix: entry.prefix.clone(),
            namespace: entry.namespace.clone(),
            declared_location: entry
                .declared_location
                .clone()
                .unwrap_or_else(|| location.clone()),
            location,
        }
    }
}

/// The Flow storage URI for a location the schema loader asks for.
///
/// The loader keys documents by `Url`'s normalized form, which percent-encodes
/// the path, while Flow storage takes the path as it is. Left encoded, a
/// directory such as `a b` or `スキーマ` would be looked up as `a%20b`.
fn storage_uri(location: &str) -> Result<Uri, String> {
    let url = url::Url::parse(location).map_err(|e| format!("`{location}`: {e}"))?;
    if url.scheme() == "file" {
        let path = url
            .to_file_path()
            .map_err(|()| format!("`{location}` is not a local file path"))?;
        return Uri::try_from(path).map_err(|e| e.to_string());
    }
    let path = percent_encoding::percent_decode_str(url.path())
        .decode_utf8()
        .map_err(|e| format!("`{location}`: {e}"))?;
    Uri::from_str(&format!(
        "{}://{}{path}",
        url.scheme(),
        url.host_str().unwrap_or_default()
    ))
    .map_err(|e| e.to_string())
}

/// The core CityGML 2.0 schemas plus each ADE's, read through Flow storage.
fn load_ade_schemas(
    ade: &[AdeBinding],
    storage_resolver: &Arc<StorageResolver>,
) -> Result<SchemaSet, SinkError> {
    let sources: Vec<AdeSource> = ade
        .iter()
        .map(|binding| AdeSource {
            namespace: binding.namespace.clone(),
            location: binding.location.clone(),
        })
        .collect();
    let read = |location: &str| -> Result<Vec<u8>, String> {
        // Relative joins stay on the entry's scheme, but the storage resolver
        // fetches over the network for others, so refuse anything unlisted.
        let scheme = url::Url::parse(location)
            .map(|url| url.scheme().to_owned())
            .map_err(|e| format!("`{location}`: {e}"))?;
        if !ADE_SCHEMA_SCHEMES.contains(&scheme.as_str()) {
            return Err(format!(
                "`{location}` uses the `{scheme}` scheme; ADE schemas are read only from {}",
                ADE_SCHEMA_SCHEMES.join(", ")
            ));
        }
        let uri = storage_uri(location)?;
        let storage = storage_resolver.resolve(&uri).map_err(|e| e.to_string())?;
        storage
            .get_sync(uri.path().as_path())
            .map(|bytes| bytes.to_vec())
            .map_err(|e| e.to_string())
    };
    SchemaSet::with_ade_from(&sources, &read)
        .map_err(|e| SinkError::CityGmlWriter(format!("adeSchemas: {e}")))
}

#[derive(Debug, Clone)]
struct CityGmlWriterCompiledParam {
    output: String,
    epsg_code: Option<u32>,
    pretty_print: Option<bool>,
    ade: Vec<AdeBinding>,
}

#[derive(Debug, Clone)]
struct CityGmlWriterSink {
    params: CityGmlWriterCompiledParam,
    lod_mask: LodMask,
    keys: ResolvedKeys,
    generic_attributes: Vec<GenericAttribute>,
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

        // The core set is loaded once per process; a set with ADEs, once per finish.
        let loaded;
        let schema: &SchemaSet = if self.params.ade.is_empty() {
            core_schema()?
        } else {
            loaded = load_ade_schemas(&self.params.ade, &ctx.storage_resolver)?;
            &loaded
        };
        let bindings: Vec<(String, String)> = self
            .params
            .ade
            .iter()
            .map(|binding| (binding.prefix.clone(), binding.namespace.clone()))
            .collect();
        let prefixes = Prefixes::with_ade(&bindings);
        let ade_locations: Vec<(String, String)> = self
            .params
            .ade
            .iter()
            .map(|binding| (binding.namespace.clone(), binding.declared_location.clone()))
            .collect();

        write_citygml_to_storage(
            out.uri(),
            &ctx.sandbox_root,
            &self.buffer,
            &self.lod_mask,
            &self.keys,
            &self.generic_attributes,
            self.params.epsg_code,
            self.params.pretty_print.unwrap_or(true),
            &ctx.storage_resolver,
            ctx.diagnostics.as_deref(),
            schema,
            &prefixes,
            &ade_locations,
        )?;

        Ok(())
    }
}

#[cfg(all(test, not(feature = "new-geometry")))]
mod diagnostics_tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use indexmap::IndexMap;
    use reearth_flow_common::uri::Uri;
    use reearth_flow_diagnostics::{
        Disposition, DispositionPolicy, ErrorCode, OverrideInput, PolicyInput,
    };
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::node::NodeHandle;
    use reearth_flow_storage::resolve::StorageResolver;
    use reearth_flow_types::{AttributeValue, GeometryValue};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn non_citygml_geometry_feature_is_reported_not_silently_dropped() {
        let tmp = tempdir().unwrap();
        let sandbox_root = Uri::from_str(&format!("file://{}", tmp.path().display())).unwrap();
        let output = Uri::from_str(&format!("file://{}/out.gml", tmp.path().display())).unwrap();
        let storage_resolver = Arc::new(StorageResolver::new());

        let handle = Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "writer".into(),
            "CityGML Writer".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            true,
        ));

        // `Feature::from` on an empty attribute map carries `GeometryValue::None`,
        // which does not match `GeometryValue::CityGmlGeometry` — this exercises
        // the non-CityGML drop lane without fabricating CityGML geometry.
        let features = vec![Feature::from(IndexMap::<String, AttributeValue>::new())];

        write_citygml_to_storage(
            &output,
            &sandbox_root,
            &features,
            &LodMask::all(),
            &ResolvedKeys::default(),
            &[],
            None,
            false,
            &storage_resolver,
            Some(&handle),
            core_schema().unwrap(),
            &Prefixes::citygml2(),
            &[],
        )
        .unwrap();

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        assert!(summaries[0]
            .message
            .contains("citygml.non_citygml_geometry"));
    }

    /// A `citygml.empty_geometry` drop promoted to `reject` by an
    /// `errorPolicy` override must also write a side-file shard row, not
    /// just the aggregation bucket — exercised end to end via
    /// `write_citygml_to_storage`, not just the isolated
    /// `NodeDiagnosticsHandle` API.
    #[test]
    fn empty_geometry_feature_reject_override_captures_a_side_file_row_matching_the_bucket() {
        let tmp = tempdir().unwrap();
        let sandbox_root = Uri::from_str(&format!("file://{}", tmp.path().display())).unwrap();
        let output = Uri::from_str(&format!("file://{}/out.gml", tmp.path().display())).unwrap();
        let storage_resolver = Arc::new(StorageResolver::new());

        let policy = DispositionPolicy::compile(PolicyInput {
            side_file: true,
            overrides: vec![OverrideInput {
                node: Some("n1".to_string()),
                code: Some("citygml.empty_geometry".to_string()),
                category: None,
                disposition: Disposition::Reject,
            }],
            ..Default::default()
        })
        .expect("policy should compile");

        let handle = Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "writer".into(),
            "CityGML Writer".into(),
            Arc::default(),
            Arc::new(policy),
            true,
        ));

        // A feature whose geometry IS a `CityGmlGeometry` (so it clears the
        // non-CityGML-geometry lane above) but has zero `gml_geometries`, so
        // `convert_citygml_geometry` yields zero entries -- this hits the
        // *second* report_drop call site (empty-converted), not the first.
        let geometry = reearth_flow_types::Geometry::new_with(
            4326,
            GeometryValue::CityGmlGeometry(reearth_flow_types::CityGmlGeometry::default()),
        );
        let features = vec![Feature::new_with_attributes_and_geometry(
            reearth_flow_types::Attributes::new(),
            geometry,
        )];
        let feature_id = features[0].id;

        write_citygml_to_storage(
            &output,
            &sandbox_root,
            &features,
            &LodMask::all(),
            &ResolvedKeys::default(),
            &[],
            None,
            false,
            &storage_resolver,
            Some(&handle),
            core_schema().unwrap(),
            &Prefixes::citygml2(),
            &[],
        )
        .unwrap();

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::Reject)
        );
        let bucket_count = summaries[0].aggregated.as_ref().unwrap().count;
        assert_eq!(bucket_count, 1);

        let (rows, overflow) = handle
            .drain_reject_rows()
            .expect("a reject row should have been captured");
        assert_eq!(overflow, 0);
        assert_eq!(
            rows.len() as u64,
            bucket_count,
            "side-file row count must match the aggregation bucket count"
        );
        assert_eq!(rows[0].feature_id, Some(feature_id));
        // The source geometry IS present (a CityGmlGeometry, just one that
        // converts to zero entries), so has_geometry is honestly `true`.
        assert_eq!(rows[0].has_geometry, Some(true));
        assert_eq!(rows[0].code, ErrorCode::CitygmlEmptyGeometry);
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
        /// document and carries on.
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

#[cfg(test)]
mod attribute_keys_tests {
    use std::collections::HashMap;

    use reearth_flow_runtime::event::EventHub;
    use reearth_flow_runtime::executor_operation::NodeContext;
    use reearth_flow_runtime::node::SinkFactory;
    use reearth_flow_types::{AttributeValue, Feature};
    use serde_json::{json, Value};

    use super::attribute_keys::{AttributeKeys, ResolvedKeys};
    use super::classes::{CityClass, Prefixes};
    use super::{city_object_identity, CityGmlWriterFactory, ClassResolver, SchemaSet};

    fn resolved(json: Value) -> ResolvedKeys {
        serde_json::from_value::<AttributeKeys>(json)
            .unwrap()
            .resolve()
            .unwrap()
    }

    fn feature_with(pairs: &[(&str, &str)]) -> Feature {
        let mut feature = Feature::new_with_attributes(Default::default());
        for (key, value) in pairs {
            feature.insert(*key, AttributeValue::String((*value).to_string()));
        }
        feature
    }

    fn identity(feature: &Feature, keys: &ResolvedKeys) -> (CityClass, Option<String>) {
        let prefixes = Prefixes::citygml2();
        let mut classes = ClassResolver::new(&prefixes, SchemaSet::core().unwrap());
        city_object_identity(feature, keys, &mut classes, None)
    }

    fn class(value: &str) -> CityClass {
        super::classes::resolve(value, &Prefixes::citygml2(), SchemaSet::core().unwrap()).unwrap()
    }

    #[test]
    fn default_keys_read_the_readers_feature_attributes() {
        let feature = feature_with(&[
            ("__citygml_feature_type", "bldg:Building"),
            ("__citygml_gml_id", "bldg_1"),
        ]);

        let (city_type, gml_id) = identity(&feature, &ResolvedKeys::default());

        assert_eq!(city_type, class("bldg:Building"));
        assert_eq!(gml_id.as_deref(), Some("bldg_1"));
    }

    #[test]
    fn custom_keys_read_the_named_attributes() {
        let feature = feature_with(&[
            ("kind", "Road"),
            ("id", "r1"),
            ("__citygml_feature_type", "bldg:Building"),
        ]);

        let (city_type, gml_id) = identity(
            &feature,
            &resolved(json!({ "featureType": "kind", "gmlId": "id" })),
        );

        assert_eq!(city_type, class("tran:Road"));
        assert_eq!(gml_id.as_deref(), Some("r1"));
    }

    #[test]
    fn a_missing_feature_type_falls_back_to_a_generic_city_object() {
        let (city_type, gml_id) = identity(&feature_with(&[]), &ResolvedKeys::default());

        assert_eq!(city_type, CityClass::generic());
        assert_eq!(gml_id, None);
    }

    fn build(attribute_keys: Option<Value>) -> Result<(), String> {
        build_with(
            attribute_keys
                .map(|keys| ("attributeKeys", keys))
                .into_iter()
                .collect(),
        )
    }

    pub(super) fn build_with(parameters: Vec<(&str, Value)>) -> Result<(), String> {
        let mut with: HashMap<String, Value> = [(
            "output".to_string(),
            json!({"type": "string", "value": "out.gml"}),
        )]
        .into();
        for (name, value) in parameters {
            with.insert(name.to_string(), value);
        }
        CityGmlWriterFactory
            .build(
                NodeContext::default(),
                EventHub::new(10),
                "CityGML Writer".to_string(),
                Some(with),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[test]
    fn the_writer_builds_without_attribute_keys() {
        assert!(build(None).is_ok());
    }

    #[test]
    fn the_writer_builds_with_attribute_keys() {
        assert!(build(Some(json!({ "lod": "level", "featureType": "kind" }))).is_ok());
    }

    #[test]
    fn a_blank_attribute_key_fails_the_build() {
        let message = build(Some(json!({ "gmlId": " " }))).unwrap_err();

        assert!(message.contains("attributeKeys.gmlId"), "{message}");
    }
}

/// Attributes, computed values and codelists through `write_citygml_to_storage`.
/// Each world builds the same one-solid LOD1 building its own way; everything
/// after conversion is shared, so every test runs in both.
#[cfg(test)]
mod attributes_tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use reearth_flow_common::uri::Uri;
    use reearth_flow_diagnostics::ErrorCode;
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::node::NodeHandle;
    use reearth_flow_storage::resolve::StorageResolver;
    use reearth_flow_types::lod::LodMask;
    use reearth_flow_types::{AttributeValue, Attributes, Feature};
    use serde_json::{json, Value};
    use tempfile::{tempdir, TempDir};

    use super::attribute_keys::{AttributeKeys, ResolvedKeys};
    use super::attribute_keys_tests::build_with;
    use super::classes::Prefixes;
    use super::generic_attributes::GenericAttribute;
    use super::{core_schema, write_citygml_to_storage, SchemaSet};

    fn attributes(value: Value) -> Attributes {
        match AttributeValue::from(value) {
            AttributeValue::Map(map) => map,
            other => panic!("expected a map, got {other:?}"),
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    fn lod1_building(attributes: Attributes) -> Feature {
        use reearth_flow_geometry::types::coordinate::Coordinate3D;
        use reearth_flow_geometry::types::line_string::LineString3D;
        use reearth_flow_geometry::types::polygon::Polygon3D;
        use reearth_flow_types::geometry::{
            CityGmlGeometry, GeometryType, GeometryValue, GmlGeometry,
        };
        use reearth_flow_types::Geometry;

        let face = Polygon3D::new(
            LineString3D::new(vec![
                Coordinate3D::new__(139.0, 35.0, 0.0),
                Coordinate3D::new__(139.1, 35.0, 0.0),
                Coordinate3D::new__(139.0, 35.1, 0.0),
                Coordinate3D::new__(139.0, 35.0, 0.0),
            ]),
            vec![],
        );
        let solid = GmlGeometry {
            polygons: vec![face],
            len: 1,
            ..GmlGeometry::new(GeometryType::Solid, Some(1))
        };
        let geometry = CityGmlGeometry {
            gml_geometries: vec![solid],
            ..Default::default()
        };
        Feature::new_with_attributes_and_geometry(
            attributes,
            Geometry::new_with(6697, GeometryValue::CityGmlGeometry(geometry)),
        )
    }

    #[cfg(feature = "new-geometry")]
    fn lod1_building(attributes: Attributes) -> Feature {
        use reearth_flow_citygml::pipeline::{MEMBER_GML_PROPERTY_NAME_KEY, MEMBER_LOD_KEY};
        use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
        use reearth_flow_geometry::polygon_mesh::PolygonMesh3DData;
        use reearth_flow_geometry::solid::{Shell, Solid};
        use reearth_flow_geometry::{Euclidean3DGeometry, Geometry, GeometryCollection};
        use reearth_flow_types::Attribute;

        let exterior = Shell::PolygonMesh(
            PolygonMesh3DData::from_parts(
                vec![
                    [35.0, 139.0, 0.0],
                    [35.1, 139.0, 0.0],
                    [35.0, 139.1, 0.0],
                    [35.1, 139.1, 0.0],
                ],
                [[0u32, 1, 2], [1, 3, 2]],
            )
            .unwrap(),
        );
        let solid = Euclidean3DGeometry::Solid(Box::new(Solid::new(
            CoordinateFrame::Crs(EpsgCode::new(6697)),
            exterior,
            Vec::new(),
        )));
        let mut member = Attributes::new();
        member.insert(
            Attribute::new(MEMBER_LOD_KEY),
            AttributeValue::Number(1.into()),
        );
        member.insert(
            Attribute::new(MEMBER_GML_PROPERTY_NAME_KEY),
            AttributeValue::String("lod1Solid".to_string()),
        );
        Feature::new_with_attributes_and_geometry(
            attributes,
            Geometry::GeometryCollection(
                GeometryCollection::with_attributes(
                    vec![Geometry::Euclidean3D(solid)],
                    vec![member],
                )
                .unwrap(),
            ),
        )
    }

    fn diagnostics() -> NodeDiagnosticsHandle {
        NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "writer".into(),
            "CityGML Writer".into(),
            Arc::default(),
            Arc::default(),
            true,
        )
    }

    /// How many times `code` was reported, across the handle's summaries.
    fn reported(handle: &NodeDiagnosticsHandle, code: ErrorCode) -> u64 {
        handle
            .inner
            .drain_summaries()
            .iter()
            .filter(|summary| summary.code == code)
            .map(|summary| summary.aggregated.as_ref().map_or(1, |info| info.count))
            .sum()
    }

    fn position(gml: &str, needle: &str) -> usize {
        gml.find(needle)
            .unwrap_or_else(|| panic!("no `{needle}` in {gml}"))
    }

    /// A temp directory whose `out/` is the sandbox, with `out/city.gml` the
    /// output, so files outside the sandbox can stand in for source codelists.
    struct Job {
        dir: TempDir,
        sandbox_root: Uri,
        output: Uri,
        resolver: Arc<StorageResolver>,
    }

    impl Job {
        fn new() -> Self {
            let dir = tempdir().unwrap();
            let root = dir.path().display().to_string();
            Self {
                sandbox_root: Uri::from_str(&format!("file://{root}/out")).unwrap(),
                output: Uri::from_str(&format!("file://{root}/out/city.gml")).unwrap(),
                resolver: Arc::new(StorageResolver::new()),
                dir,
            }
        }

        fn url(&self, relative: &str) -> String {
            format!("file://{}/{relative}", self.dir.path().display())
        }

        fn write(
            &self,
            feature: Feature,
            keys: &ResolvedKeys,
            generic: &[GenericAttribute],
            diagnostics: Option<&NodeDiagnosticsHandle>,
        ) -> String {
            self.write_with(
                feature,
                keys,
                generic,
                diagnostics,
                core_schema().unwrap(),
                &Prefixes::citygml2(),
            )
        }

        /// `write` against `schema` and `prefixes`, declaring each ADE at a
        /// placeholder location.
        fn write_with(
            &self,
            feature: Feature,
            keys: &ResolvedKeys,
            generic: &[GenericAttribute],
            diagnostics: Option<&NodeDiagnosticsHandle>,
            schema: &SchemaSet,
            prefixes: &Prefixes,
        ) -> String {
            let ade_locations: Vec<(String, String)> = prefixes
                .ade()
                .map(|(prefix, namespace)| (namespace.to_string(), format!("{prefix}.xsd")))
                .collect();
            write_citygml_to_storage(
                &self.output,
                &self.sandbox_root,
                &[feature],
                &LodMask::all(),
                keys,
                generic,
                None,
                false,
                &self.resolver,
                diagnostics,
                schema,
                prefixes,
                &ade_locations,
            )
            .unwrap();
            std::fs::read_to_string(self.dir.path().join("out/city.gml")).unwrap()
        }
    }

    fn building_with_source_properties(extra: &[(&str, Value)]) -> Feature {
        let mut value = json!({
            "__citygml_feature_type": "bldg:Building",
            "bldg:measuredHeight": { "@uom": "m", "$": "10.5" },
            "gen:stringAttribute": { "@name": "a", "gen:value": "x" },
        });
        for (key, extra) in extra {
            value[*key] = extra.clone();
        }
        lod1_building(attributes(value))
    }

    #[test]
    fn source_properties_are_written_in_schema_order_before_the_geometry() {
        let job = Job::new();

        let gml = job.write(
            building_with_source_properties(&[]),
            &ResolvedKeys::default(),
            &[],
            None,
        );

        let generic = position(&gml, r#"<gen:stringAttribute name="a">"#);
        let height = position(
            &gml,
            r#"<bldg:measuredHeight uom="m">10.5</bldg:measuredHeight>"#,
        );
        let solid = position(&gml, "<bldg:lod1Solid");
        assert!(generic < height && height < solid, "{gml}");
    }

    #[test]
    fn a_computed_value_with_a_unit_is_written_as_a_generic_measure() {
        let job = Job::new();
        let generic: Vec<GenericAttribute> = serde_json::from_value(json!([
            { "name": "日射量", "attribute": "solar", "uom": "kWh" }
        ]))
        .unwrap();

        let gml = job.write(
            building_with_source_properties(&[("solar", json!(1234.5))]),
            &ResolvedKeys::default(),
            &generic,
            None,
        );

        assert!(
            gml.contains(
                r#"<gen:measureAttribute name="日射量"><gen:value uom="kWh">1234.5</gen:value></gen:measureAttribute>"#
            ),
            "{gml}"
        );
    }

    #[test]
    fn a_computed_source_with_an_unknown_prefix_is_written_once_and_not_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let generic: Vec<GenericAttribute> = serde_json::from_value(json!([
            { "name": "x", "attribute": "solar:x", "uom": "kWh" }
        ]))
        .unwrap();

        let gml = job.write(
            building_with_source_properties(&[("solar:x", json!(1.5))]),
            &ResolvedKeys::default(),
            &generic,
            Some(&handle),
        );

        assert_eq!(gml.matches("<gen:measureAttribute").count(), 1, "{gml}");
        assert!(
            gml.contains(r#"<gen:value uom="kWh">1.5</gen:value>"#),
            "{gml}"
        );
        assert_eq!(reported(&handle, ErrorCode::CitygmlAttributeNotPlaced), 0);
    }

    #[test]
    fn a_configured_map_is_read_instead_of_the_features_own_attributes() {
        let job = Job::new();
        let keys = serde_json::from_value::<AttributeKeys>(json!({ "cityGmlAttributes": "cga" }))
            .unwrap()
            .resolve()
            .unwrap();
        let feature = lod1_building(attributes(json!({
            "__citygml_feature_type": "bldg:Building",
            "cga": { "bldg:class": "3001" },
            "bldg:class": "9999",
        })));

        let gml = job.write(feature, &keys, &[], None);

        assert!(gml.contains("<bldg:class>3001</bldg:class>"), "{gml}");
        assert!(!gml.contains("9999"), "{gml}");
    }

    #[test]
    fn a_top_level_computed_source_leaves_the_configured_maps_property_alone() {
        let job = Job::new();
        let keys = serde_json::from_value::<AttributeKeys>(json!({ "cityGmlAttributes": "cga" }))
            .unwrap()
            .resolve()
            .unwrap();
        let generic: Vec<GenericAttribute> =
            serde_json::from_value(json!([{ "name": "n", "attribute": "bldg:class" }])).unwrap();
        let feature = lod1_building(attributes(json!({
            "__citygml_feature_type": "bldg:Building",
            "cga": { "bldg:class": "3001" },
            "bldg:class": 5,
        })));

        let gml = job.write(feature, &keys, &generic, None);

        assert!(gml.contains("<bldg:class>3001</bldg:class>"), "{gml}");
        assert!(
            gml.contains(
                r#"<gen:intAttribute name="n"><gen:value>5</gen:value></gen:intAttribute>"#
            ),
            "{gml}"
        );
    }

    #[test]
    fn a_feature_is_reported_once_per_code_however_many_attributes_are_left_out() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[
            (
                "uro:buildingIDAttribute",
                json!({ "uro:buildingID": "13104-bldg-1" }),
            ),
            (
                "uro:buildingDetailAttribute",
                json!({ "uro:totalFloorArea": "100" }),
            ),
        ]);

        job.write(feature, &ResolvedKeys::default(), &[], Some(&handle));

        assert_eq!(reported(&handle, ErrorCode::CitygmlAttributeNotPlaced), 1);
    }

    #[test]
    fn a_property_outside_the_known_namespaces_is_reported_and_the_feature_still_written() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[(
            "uro:buildingIDAttribute",
            json!({ "uro:buildingID": "13104-bldg-1" }),
        )]);

        let gml = job.write(feature, &ResolvedKeys::default(), &[], Some(&handle));

        assert!(gml.contains("<bldg:lod1Solid"), "{gml}");
        assert!(!gml.contains("buildingIDAttribute"), "{gml}");
        assert_eq!(reported(&handle, ErrorCode::CitygmlAttributeNotPlaced), 1);
    }

    fn classified_building(code_space: &str) -> Feature {
        building_with_source_properties(&[
            ("bldg:class", json!("普通建物")),
            ("bldg:class_code", json!("3001")),
            ("bldg:class_codeSpace", json!(code_space)),
        ])
    }

    #[test]
    fn a_referenced_codelist_is_staged_beside_the_output() {
        let job = Job::new();
        let codelist = "<gml:Dictionary>3001</gml:Dictionary>\n";
        std::fs::create_dir(job.dir.path().join("codelists")).unwrap();
        std::fs::write(
            job.dir.path().join("codelists/Building_class.xml"),
            codelist,
        )
        .unwrap();
        let handle = diagnostics();

        let gml = job.write(
            classified_building(&job.url("codelists/Building_class.xml")),
            &ResolvedKeys::default(),
            &[],
            Some(&handle),
        );

        assert!(
            gml.contains(
                r#"<bldg:class codeSpace="city_codelists/Building_class.xml">3001</bldg:class>"#
            ),
            "{gml}"
        );
        assert_eq!(
            std::fs::read_to_string(job.dir.path().join("out/city_codelists/Building_class.xml"))
                .unwrap(),
            codelist
        );
        assert_eq!(reported(&handle, ErrorCode::CitygmlCodelistNotStaged), 0);
    }

    #[test]
    fn an_unreadable_codelist_keeps_its_url_and_is_reported() {
        let job = Job::new();
        let url = job.url("codelists/Building_class.xml");
        let handle = diagnostics();

        let gml = job.write(
            classified_building(&url),
            &ResolvedKeys::default(),
            &[],
            Some(&handle),
        );

        assert!(
            gml.contains(&format!(
                r#"<bldg:class codeSpace="{url}">3001</bldg:class>"#
            )),
            "{gml}"
        );
        assert_eq!(reported(&handle, ErrorCode::CitygmlCodelistNotStaged), 1);
    }

    #[test]
    fn a_blank_generic_attribute_name_fails_the_build() {
        let message = build_with(vec![(
            "genericAttributes",
            json!([{ "name": " ", "attribute": "x" }]),
        )])
        .unwrap_err();

        assert!(message.contains("genericAttributes[0].name"), "{message}");
    }
}

#[cfg(test)]
mod ade_schemas_tests {
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Arc;

    use reearth_flow_common::uri::Uri;
    use reearth_flow_diagnostics::ErrorCode;
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::event::EventHub;
    use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
    use reearth_flow_runtime::node::{NodeHandle, SinkFactory, FEATURES_PORT};
    use reearth_flow_types::{AttributeValue, Feature};
    use serde_json::{json, Value};
    use tempfile::tempdir;

    use super::CityGmlWriterFactory;

    const URO: &str = "https://www.geospatial.jp/iur/uro/3.1";

    /// A feature with one LOD0 triangle, which the writer always writes.
    #[cfg(not(feature = "new-geometry"))]
    fn writable() -> Feature {
        use reearth_flow_geometry::types::coordinate::Coordinate3D;
        use reearth_flow_geometry::types::line_string::LineString3D;
        use reearth_flow_geometry::types::polygon::Polygon3D;
        use reearth_flow_types::geometry::{
            CityGmlGeometry, GeometryType, GeometryValue, GmlGeometry,
        };
        use reearth_flow_types::{Attributes, Geometry};

        let ring = LineString3D::new(vec![
            Coordinate3D::new__(139.0, 35.0, 0.0),
            Coordinate3D::new__(139.1, 35.0, 0.0),
            Coordinate3D::new__(139.0, 35.1, 0.0),
            Coordinate3D::new__(139.0, 35.0, 0.0),
        ]);
        let mut surface = GmlGeometry::new(GeometryType::Surface, Some(0));
        surface.polygons.push(Polygon3D::new(ring, vec![]));
        Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::new_with(
                6697,
                GeometryValue::CityGmlGeometry(CityGmlGeometry::new(vec![surface], vec![], vec![])),
            ),
        )
    }

    /// A feature with one LOD0 triangle, which the writer always writes.
    #[cfg(feature = "new-geometry")]
    fn writable() -> Feature {
        use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
        use reearth_flow_geometry::polygon::Polygon3D;
        use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};

        let ring = vec![
            [35.0, 139.0, 0.0],
            [35.1, 139.0, 0.0],
            [35.0, 139.1, 0.0],
            [35.0, 139.0, 0.0],
        ];
        let polygon = Polygon3D::from_rings(
            CoordinateFrame::Crs(EpsgCode::new(6697)),
            ring,
            Vec::<Vec<[f64; 3]>>::new(),
        );
        Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(
            Box::new(polygon),
        )))
    }

    fn typed(feature_type: &str) -> Feature {
        let mut feature = writable();
        feature.insert(
            "__citygml_feature_type",
            AttributeValue::String(feature_type.to_string()),
        );
        feature
    }

    fn params(ade_schemas: Option<Value>) -> HashMap<String, Value> {
        let mut with: HashMap<String, Value> = [(
            "output".to_string(),
            json!({"type": "string", "value": "out.gml"}),
        )]
        .into();
        if let Some(ade_schemas) = ade_schemas {
            with.insert("adeSchemas".to_string(), ade_schemas);
        }
        with
    }

    fn build(ade_schemas: Value) -> Result<(), String> {
        CityGmlWriterFactory
            .build(
                NodeContext::default(),
                EventHub::new(10),
                "CityGML Writer".to_string(),
                Some(params(Some(ade_schemas))),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn entry(prefix: &str, namespace: &str, location: &str) -> Value {
        json!({ "prefix": prefix, "namespace": namespace, "schemaLocation": literal(location) })
    }

    fn literal(value: &str) -> Value {
        json!({ "type": "string", "value": value })
    }

    fn context(
        sandbox: &std::path::Path,
        diagnostics: Option<Arc<NodeDiagnosticsHandle>>,
    ) -> NodeContext {
        NodeContext {
            sandbox_root: Uri::from_str(&format!("file://{}", sandbox.display())).unwrap(),
            diagnostics,
            ..NodeContext::default()
        }
    }

    /// Build the writer, feed it `features` and finish it.
    fn run(
        ctx: &NodeContext,
        ade_schemas: Option<Value>,
        features: Vec<Feature>,
    ) -> Result<(), String> {
        let mut sink = CityGmlWriterFactory
            .build(
                ctx.clone(),
                EventHub::new(10),
                "CityGML Writer".to_string(),
                Some(params(ade_schemas)),
            )
            .map_err(|e| e.to_string())?;
        for feature in features {
            sink.process(ExecutorContext::new_with_node_context_feature_and_port(
                ctx,
                feature,
                FEATURES_PORT.clone(),
            ))
            .map_err(|e| e.to_string())?;
        }
        sink.finish(ctx.clone()).map_err(|e| e.to_string())
    }

    /// The written document and how many times `CitygmlUnknownFeatureType`
    /// was reported.
    fn write(ade_schemas: Option<Value>, features: Vec<Feature>) -> (String, u64) {
        let tmp = tempdir().unwrap();
        let handle = Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "writer".into(),
            "CityGML Writer".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            true,
        ));
        let ctx = context(tmp.path(), Some(handle.clone()));
        run(&ctx, ade_schemas, features).unwrap();

        let xml = std::fs::read_to_string(tmp.path().join("out.gml")).unwrap();
        let unknown = handle
            .inner
            .drain_summaries()
            .iter()
            .filter(|summary| summary.code == ErrorCode::CitygmlUnknownFeatureType)
            .map(|summary| summary.aggregated.as_ref().unwrap().count)
            .sum();
        (xml, unknown)
    }

    /// The PLATEAU fixture's i-UR 3.1 schema, as a `file://` URL.
    fn iur_location() -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../testing/data/fixtures/plateau-citymodel/15202_nagaoka-shi_city_2024_citygml_1_op/schemas/iur/uro/3.1/urbanObject.xsd")
            .canonicalize()
            .unwrap();
        url::Url::from_file_path(path).unwrap().to_string()
    }

    #[test]
    fn the_writer_builds_with_an_ade_schema() {
        let result = build(json!([entry("uro", URO, "file:///schemas/uro.xsd")]));
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn a_blank_prefix_fails_the_build() {
        let message = build(json!([entry(" ", URO, "file:///schemas/uro.xsd")])).unwrap_err();
        assert!(message.contains("adeSchemas[0].prefix"), "{message}");
    }

    #[test]
    fn a_prefix_that_is_not_an_ncname_fails_the_build() {
        let message = build(json!([entry("1uro", URO, "file:///schemas/uro.xsd")])).unwrap_err();
        assert!(message.contains("adeSchemas[0].prefix"), "{message}");
    }

    #[test]
    fn a_built_in_prefix_fails_the_build() {
        let message = build(json!([entry("bldg", URO, "file:///schemas/uro.xsd")])).unwrap_err();
        assert!(message.contains("adeSchemas[0].prefix"), "{message}");
    }

    #[test]
    fn a_duplicate_prefix_fails_the_build() {
        let message = build(json!([
            entry("uro", URO, "file:///schemas/uro.xsd"),
            entry(
                "uro",
                "https://example.com/other",
                "file:///schemas/other.xsd"
            ),
        ]))
        .unwrap_err();
        assert!(message.contains("adeSchemas[1].prefix"), "{message}");
    }

    #[test]
    fn a_duplicate_namespace_fails_the_build() {
        let message = build(json!([
            entry("uro", URO, "file:///schemas/uro.xsd"),
            entry("uro2", URO, "file:///schemas/other.xsd"),
        ]))
        .unwrap_err();
        assert!(message.contains("adeSchemas[1].namespace"), "{message}");
    }

    #[test]
    fn a_blank_namespace_fails_the_build() {
        let message = build(json!([entry("uro", "", "file:///schemas/uro.xsd")])).unwrap_err();
        assert!(message.contains("adeSchemas[0].namespace"), "{message}");
    }

    #[test]
    fn an_http_schema_location_fails_the_build_and_points_at_declared_location() {
        let message = build(json!([entry(
            "uro",
            URO,
            "https://www.geospatial.jp/iur/schemas/uro/3.1/urbanObject.xsd"
        )]))
        .unwrap_err();
        assert!(
            message.contains("adeSchemas[0].schemaLocation"),
            "{message}"
        );
        assert!(message.contains("declaredLocation"), "{message}");
        assert!(message.contains("`file://`"), "{message}");
    }

    #[test]
    fn an_s3_schema_location_fails_the_build() {
        let message = build(json!([entry("uro", URO, "s3://bucket/uro.xsd")])).unwrap_err();
        assert!(
            message.contains("adeSchemas[0].schemaLocation"),
            "{message}"
        );
        assert!(message.contains("`s3`"), "{message}");
        assert!(message.contains("`gs://`"), "{message}");
    }

    #[test]
    fn whitespace_in_the_namespace_fails_the_build() {
        let message = build(json!([entry(
            "uro",
            "https://example.com/a b",
            "file:///schemas/uro.xsd"
        )]))
        .unwrap_err();
        assert!(message.contains("adeSchemas[0].namespace"), "{message}");
    }

    #[test]
    fn whitespace_in_the_declared_location_fails_the_build() {
        let message = build(json!([{
            "prefix": "uro",
            "namespace": URO,
            "schemaLocation": literal("file:///schemas/uro.xsd"),
            "declaredLocation": "../schemas/a b/uro.xsd",
        }]))
        .unwrap_err();
        assert!(
            message.contains("adeSchemas[0].declaredLocation"),
            "{message}"
        );
    }

    #[test]
    fn a_relative_schema_location_fails_the_build() {
        let message = build(json!([entry("uro", URO, "schemas/uro.xsd")])).unwrap_err();
        assert!(
            message.contains("adeSchemas[0].schemaLocation"),
            "{message}"
        );
    }

    #[test]
    fn a_blank_declared_location_fails_the_build() {
        let message = build(json!([{
            "prefix": "uro",
            "namespace": URO,
            "schemaLocation": literal("file:///schemas/uro.xsd"),
            "declaredLocation": " ",
        }]))
        .unwrap_err();
        assert!(
            message.contains("adeSchemas[0].declaredLocation"),
            "{message}"
        );
    }

    #[test]
    fn an_unknown_class_is_reported_once_per_feature() {
        let features = (0..3).map(|_| typed("my_building_export_v2")).collect();

        let (xml, unknown) = write(None, features);

        assert_eq!(xml.matches("<gen:GenericCityObject ").count(), 3, "{xml}");
        assert_eq!(unknown, 3);
    }

    #[test]
    fn a_missing_feature_type_is_generic_and_not_reported() {
        let (xml, unknown) = write(None, vec![writable()]);

        assert_eq!(xml.matches("<gen:GenericCityObject ").count(), 1, "{xml}");
        assert_eq!(unknown, 0);
    }

    #[test]
    fn an_ade_class_is_written_when_its_schema_is_listed() {
        let ade_schemas = json!([{
            "prefix": "uro",
            "namespace": URO,
            "schemaLocation": literal(&iur_location()),
            "declaredLocation": "../schemas/iur/uro/3.1/urbanObject.xsd",
        }]);

        let (xml, unknown) = write(Some(ade_schemas), vec![typed("uro:UndergroundBuilding")]);

        assert!(xml.contains("<uro:UndergroundBuilding "), "{xml}");
        assert!(xml.contains(&format!(r#"xmlns:uro="{URO}""#)), "{xml}");
        assert!(
            xml.contains(&format!("{URO} ../schemas/iur/uro/3.1/urbanObject.xsd")),
            "{xml}"
        );
        assert_eq!(unknown, 0);
    }

    /// A copy of the i-UR schema under a directory whose name has a space and
    /// Japanese, as PLATEAU data often does. Returns the dir guard and the path.
    fn iur_copy_under_awkward_names() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("a b").join("スキーマ");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("urbanObject.xsd");
        let source = url::Url::parse(&iur_location())
            .unwrap()
            .to_file_path()
            .unwrap();
        std::fs::copy(source, &path).unwrap();
        (tmp, path)
    }

    fn writes_underground_building_from(location: String) {
        let ade_schemas = json!([entry("uro", URO, &location)]);

        let (xml, unknown) = write(Some(ade_schemas), vec![typed("uro:UndergroundBuilding")]);

        assert!(xml.contains("<uro:UndergroundBuilding "), "{xml}");
        assert_eq!(unknown, 0);
        // With no `declaredLocation`, the root declares the normalized `schemaLocation`.
        let normalized = url::Url::parse(&location).unwrap();
        assert!(xml.contains(&format!("{URO} {normalized}")), "{xml}");
    }

    #[test]
    fn an_ade_schema_under_a_percent_encoded_path_is_read() {
        let (_tmp, path) = iur_copy_under_awkward_names();
        let location = url::Url::from_file_path(&path).unwrap().to_string();
        assert!(location.contains("%20"), "{location}");

        writes_underground_building_from(location);
    }

    #[test]
    fn an_ade_schema_under_a_raw_path_with_a_space_and_japanese_is_read() {
        let (_tmp, path) = iur_copy_under_awkward_names();

        writes_underground_building_from(format!("file://{}", path.display()));
    }

    #[test]
    fn a_schema_location_expression_is_evaluated_against_the_variables() {
        let tmp = tempdir().unwrap();
        let ctx = NodeContext {
            variables: Arc::new(
                [("uroSchema".to_string(), Value::String(iur_location()))]
                    .into_iter()
                    .collect(),
            ),
            ..context(tmp.path(), None)
        };
        let ade_schemas = json!([{
            "prefix": "uro",
            "namespace": URO,
            "schemaLocation": { "type": "flowExpr", "value": "variables[\"uroSchema\"]" },
        }]);

        run(
            &ctx,
            Some(ade_schemas),
            vec![typed("uro:UndergroundBuilding")],
        )
        .unwrap();

        let xml = std::fs::read_to_string(tmp.path().join("out.gml")).unwrap();
        assert!(xml.contains("<uro:UndergroundBuilding "), "{xml}");
    }

    #[test]
    fn an_object_storage_location_is_read_with_its_path_decoded() {
        let uri = super::storage_uri("gs://bucket/a%20b/%E3%82%B9/uro.xsd").unwrap();

        assert_eq!(uri.as_str(), "gs://bucket/a b/ス/uro.xsd");
        assert_eq!(uri.path(), std::path::PathBuf::from("/a b/ス/uro.xsd"));
    }

    #[test]
    fn an_ade_class_without_its_schema_is_generic_and_reported() {
        let (xml, unknown) = write(None, vec![typed("uro:UndergroundBuilding")]);

        assert!(!xml.contains("uro:"), "{xml}");
        assert_eq!(xml.matches("<gen:GenericCityObject ").count(), 1, "{xml}");
        assert_eq!(unknown, 1);
    }

    #[test]
    fn an_unreadable_ade_schema_fails_the_write() {
        let tmp = tempdir().unwrap();
        let ctx = context(tmp.path(), None);
        let missing = format!("file://{}/absent.xsd", tmp.path().display());

        let message = run(
            &ctx,
            Some(json!([entry("uro", URO, &missing)])),
            vec![typed("bldg:Building")],
        )
        .unwrap_err();

        assert!(message.contains("adeSchemas"), "{message}");
        assert!(message.contains("absent.xsd"), "{message}");
    }
}
