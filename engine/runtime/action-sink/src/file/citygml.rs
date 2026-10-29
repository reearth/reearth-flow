pub mod attribute_keys;
pub mod classes;
pub mod hierarchy;
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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::BufWriter;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
#[cfg(feature = "new-geometry")]
use reearth_flow_citygml::schema::QName;
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
use reearth_flow_types::{AttributeValue, Attributes, Code, Feature, CITYGML_ROOT_GML_ID_KEY};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use attribute_keys::{AttributeKeys, ResolvedKeys};
use classes::{CityClass, ClassResolver, Prefixes};
use hierarchy::{
    build_feature_objects, ContainmentProperties, FeatureObjects, FeatureParts, TopLevel,
    TreeObject,
};
use model::{
    AppearanceBundle, BoundingEnvelope, ConvertedCityObject, ObjectNode, TextureRef, TextureSource,
};
use placement::Placed;
use properties::{Skip, XmlProperty};
use writer::CityGmlXmlWriter;

/// Write `features` as CityGML 2.0 to `output`, with their CityGML properties,
/// staging the texture images and codelists the document references alongside.
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

    // The writer's own inputs are top-level pipeline data, not properties. A map
    // named by `cityGmlAttributes` holds only the source's properties, so none of
    // those top-level names may remove anything from it.
    let excluded: HashSet<&str> = match keys.city_gml_attributes {
        Some(_) => HashSet::new(),
        None => [
            keys.feature_type.as_str(),
            keys.gml_id.as_str(),
            keys.lod.as_str(),
            keys.gml_property_name.as_str(),
            keys.parent_gml_id.as_str(),
        ]
        .into_iter()
        .collect(),
    };

    let mut converted: Vec<ConvertedCityObject> = Vec::with_capacity(features.len());
    let mut objects: Vec<FeatureObjects> = Vec::with_capacity(features.len());
    // The run summary counts each report as one affected feature, so each
    // feature is reported once per code, wherever in its tree the code came
    // from.
    let mut feature_codes: Vec<HashSet<ErrorCode>> = Vec::with_capacity(features.len());
    // Per feature, the codelist URLs its properties may reference that can be
    // staged.
    let mut stageable: Vec<Vec<String>> = Vec::with_capacity(features.len());
    let mut classes = ClassResolver::new(prefixes, schema);
    let mut containment = ContainmentProperties::default();
    let mut envelope: Option<BoundingEnvelope> = None;
    let mut crs = CrsCoverage::default();
    let mut textures: Vec<TextureRef> = Vec::new();
    let mut texture_keys: HashSet<String> = HashSet::new();

    for (source, feature) in features.iter().enumerate() {
        let mut object = converter::convert_city_object(feature, lod_mask, keys, diagnostics)?;

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

        // Every feature is built, as one without geometry of its own may
        // still be written as the parent of features joined into it.
        let mut codes: HashSet<ErrorCode> = HashSet::new();
        // Only a source `gml:id` is offered as a candidate. The engine's own
        // feature id is a per-run UUID, so it would make the output
        // nondeterministic, and a minted id would be indistinguishable from
        // a real one like `bldg_<uuid>`, which is precisely the shape PLATEAU
        // uses. `claim_gml_id` mints a stable `<prefix>_<n>` instead.
        let (class, gml_id) = city_object_identity(feature, keys, &mut classes, &mut codes);
        let properties = place_properties(
            feature,
            &class,
            keys,
            &excluded,
            schema,
            prefixes,
            &mut containment,
            &mut codes,
        );
        let parts = FeatureParts {
            source,
            class,
            // The root's id is written as given, so it is only ever the
            // feature's own resolved `gml:id`.
            gml_id,
            parent_key: parent_key(feature, keys),
            document_root: feature
                .get(CITYGML_ROOT_GML_ID_KEY)
                .and_then(|value| value.as_string()),
            geometries: std::mem::take(&mut object.geometries),
            properties: properties.placed,
            tree: properties.tree,
            referenced: properties.referenced,
            data_owners: properties.data_owners,
        };
        let mut built = build_feature_objects(parts, schema, prefixes, &mut classes);
        codes.extend(std::mem::take(&mut built.reports));

        converted.push(object);
        objects.push(built);
        feature_codes.push(codes);
        stageable.push(properties.code_spaces);
    }

    let joined = hierarchy::join(objects, schema);

    // A feature's own object is written when the tree it ends up in has
    // geometry; its orphans are always written.
    let mut root_written = vec![false; features.len()];
    let mut written = vec![false; features.len()];
    let mut top: Vec<ObjectNode> = Vec::new();
    for TopLevel {
        object,
        orphan,
        roots,
    } in joined.top
    {
        if !orphan && !object.has_geometry() {
            continue;
        }
        for source in roots {
            root_written[source] = true;
            written[source] = true;
        }
        written[object.source] = true;
        top.push(object);
    }

    if let Some(diagnostics) = diagnostics {
        for (source, code) in joined.reports {
            feature_codes[source].insert(code);
        }
        for (source, (feature, codes)) in features.iter().zip(feature_codes).enumerate() {
            // A feature left out whole as a copy is reported as that, and
            // nothing else: what it holds is written with the first copy. One
            // that only lost a nested copy is still reported when it is not
            // written.
            #[cfg(feature = "new-geometry")]
            let copy = joined.copies[source];
            for code in codes {
                // A feature that is not written reports only that, as before
                // the join; a copy left out is reported either way, as it can
                // be why.
                if written[source] || code == ErrorCode::CitygmlDuplicateObjectNotWritten {
                    diagnostics.report_warn(code, Some(feature.id));
                }
            }
            // The legacy converter reports this itself, as only it can tell
            // geometry that is not CityGML from none at all.
            #[cfg(feature = "new-geometry")]
            if !root_written[source] && !copy {
                diagnostics.report_drop(
                    ErrorCode::CitygmlEmptyGeometry,
                    Some(feature.id),
                    Some(feature.has_geometry()),
                );
            }
        }
    }

    // Every feature was filtered out or held nothing writable: no document.
    if top.is_empty() {
        return Ok(());
    }

    let mut code_spaces: Vec<String> = Vec::new();
    for object in &top {
        for_each_property(object, &mut |source, property| {
            collect_code_spaces(property, &stageable[source], &mut code_spaces)
        });
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
    for object in &mut top {
        for_each_property_mut(object, &mut |property| {
            remap_code_spaces(property, &code_space_remap)
        });
    }
    // Indexed by feature, which is each object's source.
    let appearances: Vec<Option<&AppearanceBundle>> = converted
        .iter()
        .map(|object| {
            object
                .appearance
                .has_content()
                .then_some(&object.appearance)
        })
        .collect();
    let mut lod1_surfaces_dropped: BTreeMap<usize, usize> = BTreeMap::new();
    let mut geometry_not_written: BTreeMap<usize, usize> = BTreeMap::new();

    let buffer_size = (features.len() * 4096).clamp(32 * 1024, 512 * 1024);
    let mut xml_buffer = Vec::with_capacity(buffer_size);
    {
        let buf_writer = BufWriter::with_capacity(buffer_size, &mut xml_buffer);
        let mut xml_writer = CityGmlXmlWriter::new(buf_writer, pretty_print, srs_name);
        xml_writer.set_uri_remap(uri_remap);
        xml_writer.set_schema(schema, prefixes.clone());

        xml_writer.write_header(envelope.as_ref(), ade_locations)?;

        for object in top {
            let outcome = xml_writer.write_object_tree(object, &appearances)?;
            for (source, dropped) in outcome.lod1_surfaces_dropped {
                *lod1_surfaces_dropped.entry(source).or_default() += dropped;
            }
            for (source, left_out) in outcome.geometry_not_written {
                *geometry_not_written.entry(source).or_default() += left_out;
            }
        }

        xml_writer.write_footer()?;
    }

    if let Some(diagnostics) = diagnostics {
        // Once per feature, however many of its geometries, its nested
        // objects' included, had no property: the row names the feature, which
        // is what a user acts on.
        for source in geometry_not_written.into_keys() {
            diagnostics.report_warn(
                ErrorCode::CitygmlGeometryNotWritten,
                Some(features[source].id),
            );
        }
        // The XSD permits both `lod1Solid` and `lod1MultiSurface`, so the
        // schema gate cannot see this one and it is reported here instead of
        // dropped in silence, against the feature each dropped surface came
        // from.
        for source in lod1_surfaces_dropped.into_keys() {
            diagnostics.report_drop(
                ErrorCode::CitygmlLod1ShellConflict,
                Some(features[source].id),
                Some(true),
            );
        }
    }

    let storage = storage_resolver
        .resolve(output)
        .map_err(SinkError::citygml_writer)?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(xml_buffer))
        .map_err(SinkError::citygml_writer)?;

    Ok(())
}

/// One feature's own properties, and the nested city objects taken out of them.
struct FeatureProperties {
    /// Placed in the feature's class.
    placed: Vec<Placed>,
    /// Placed in their own classes once the object tree is built.
    tree: Vec<TreeObject>,
    /// The keys of the objects held through a reference property.
    referenced: HashSet<String>,
    /// The city object holding each keyed object that is not one.
    data_owners: HashMap<String, Option<String>>,
    /// The codelist URLs any of them may reference that can be staged.
    code_spaces: Vec<String>,
}

/// One feature's properties placed in `class`'s content model, with the code
/// of every skip added to `codes`.
///
/// Under `new-geometry` the nested city objects are taken out first, to be
/// written as objects of their own. The legacy world writes their geometry
/// onto the feature, so it leaves them in, and placement reports them.
///
/// Properties are built and placed through the writer's own `schema` and
/// `prefixes`, so an ADE listed in `adeSchemas` makes its properties placeable
/// wherever its schema allows them.
#[allow(clippy::too_many_arguments)]
fn place_properties(
    feature: &Feature,
    class: &CityClass,
    keys: &ResolvedKeys,
    excluded: &HashSet<&str>,
    schema: &SchemaSet,
    prefixes: &Prefixes,
    containment: &mut ContainmentProperties,
    codes: &mut HashSet<ErrorCode>,
) -> FeatureProperties {
    let no_attributes = Attributes::new();
    let source = match &keys.city_gml_attributes {
        None => feature.attributes.as_ref(),
        // A configured map that is missing, or is not a map, holds no properties.
        Some(key) => match feature.get(key) {
            Some(AttributeValue::Map(map)) => map,
            _ => &no_attributes,
        },
    };
    let converted = properties::from_attributes(source, excluded, prefixes, &|prefix, local| {
        placement::is_geometry_property(schema, prefixes, &class.element, prefix, local)
    });
    #[cfg(feature = "new-geometry")]
    let (properties, tree) =
        hierarchy::split_nested_objects(schema, prefixes, converted.properties);
    #[cfg(feature = "new-geometry")]
    let referenced = hierarchy::referenced_keys(source, prefixes);
    #[cfg(feature = "new-geometry")]
    let data_owners = hierarchy::data_owners(source, schema, prefixes);
    #[cfg(not(feature = "new-geometry"))]
    let (properties, tree, referenced, data_owners) = (
        converted.properties,
        Vec::new(),
        HashSet::new(),
        HashMap::new(),
    );
    let (placed, placement_skips) = placement::place(schema, prefixes, &class.element, properties);
    // An unprefixed key cannot be bound, so it is never written. A configured
    // map holds only what the reader read, so each of its unprefixed keys is a
    // source element in a default namespace, and is reported. A feature's own
    // unprefixed attributes, even `name` or `class`, are Flow data, and are
    // never written.
    let unprefixed_property =
        keys.city_gml_attributes.is_some() && !converted.unprefixed.is_empty();
    // A reader that extracts a nested city object leaves the property that
    // held it behind as an empty string, and the object is joined back into
    // that property. Such an empty property is absent, not a lost value.
    #[cfg(feature = "new-geometry")]
    let mut wrappers = converted
        .blank_elements
        .iter()
        .filter(|blank| {
            let parent = match &blank.parent {
                None => Some(class.element.clone()),
                Some((prefix, local)) => prefixes
                    .namespace(prefix)
                    .map(|namespace| QName::new(namespace, local.as_str())),
            };
            let property = prefixes
                .namespace(&blank.prefix)
                .map(|namespace| QName::new(namespace, blank.local.as_str()));
            matches!(
                (parent, property),
                (Some(parent), Some(property)) if containment.holds_objects(schema, &parent, &property)
            )
        })
        .count();
    // The legacy writer joins nothing, so every empty property is reported.
    #[cfg(not(feature = "new-geometry"))]
    let mut wrappers = {
        let _ = containment;
        0
    };
    // Each wrapper was counted once as an empty value.
    let reported_skips = converted.skipped.iter().filter(|skip| {
        let wrapper = **skip == Skip::EmptyValue && wrappers > 0;
        if wrapper {
            wrappers -= 1;
        }
        !wrapper
    });

    codes.extend(
        reported_skips
            .chain(&placement_skips)
            .map(|skip| skip.code()),
    );
    if unprefixed_property {
        codes.insert(ErrorCode::CitygmlAttributeNotPlaced);
    }
    FeatureProperties {
        placed,
        tree,
        referenced,
        data_owners,
        code_spaces: converted.code_spaces,
    }
}

/// Call `f` on every placed property of `object` and of the objects nested
/// in it, parents first, with the source of the object holding it.
fn for_each_property(object: &ObjectNode, f: &mut impl FnMut(usize, &XmlProperty)) {
    for placed in &object.properties {
        f(object.source, &placed.property);
    }
    for nested in &object.children {
        for_each_property(&nested.object, f);
    }
}

/// [`for_each_property`], changing each property.
fn for_each_property_mut(object: &mut ObjectNode, f: &mut impl FnMut(&mut XmlProperty)) {
    for placed in &mut object.properties {
        f(&mut placed.property);
    }
    for nested in &mut object.children {
        for_each_property_mut(&mut nested.object, f);
    }
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
        "Writes features to CityGML 2.0 files with the CityGML properties they were read with in schema order, and nested city objects such as boundary surfaces, openings and building parts under their parents. Properties of an ADE such as uro: are written only when ADE Schemas lists it, and reported otherwise."
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

/// The CityGML class and the source `gml:id` a feature records under the
/// configured keys. An unresolved or missing class becomes
/// `gen:GenericCityObject`; a missing id is minted by the writer.
///
/// Only an unresolved value is reported, by adding its code to `codes`. A
/// missing or non-text one claims no class, so the fallback is what was asked
/// for.
fn city_object_identity(
    feature: &Feature,
    keys: &ResolvedKeys,
    classes: &mut ClassResolver,
    codes: &mut HashSet<ErrorCode>,
) -> (CityClass, Option<String>) {
    let class = match feature
        .get(keys.feature_type.as_str())
        .and_then(|value| value.as_string())
    {
        None => CityClass::generic(),
        Some(value) => classes.class_of(&value).unwrap_or_else(|| {
            codes.insert(ErrorCode::CitygmlUnknownFeatureType);
            CityClass::generic()
        }),
    };
    let gml_id = feature
        .get(keys.gml_id.as_str())
        .and_then(|value| value.as_string());
    (class, gml_id)
}

/// The `gml:id` of the object `feature` nests under.
#[cfg(feature = "new-geometry")]
fn parent_key(feature: &Feature, keys: &ResolvedKeys) -> Option<String> {
    feature
        .get(keys.parent_gml_id.as_str())
        .and_then(|value| value.as_string())
}

/// The legacy writer flattens nested objects onto their feature, so it nests
/// no feature in another either.
#[cfg(not(feature = "new-geometry"))]
fn parent_key(_feature: &Feature, _keys: &ResolvedKeys) -> Option<String> {
    None
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
    /// # ADE Schemas
    /// Application Domain Extensions (ADEs) to write classes from, such as PLATEAU i-UR. Each entry names the prefix used in feature types and attributes (for example `uro`), its namespace, and where to read its XSD. A feature type such as `uro:UndergroundBuilding` is written as itself when its ADE is listed here. List every ADE your features use: a class from an ADE left out is written as `gen:GenericCityObject`, and the objects nested in it are written as top-level objects.
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
        city_object_identity(
            feature,
            keys,
            &mut classes,
            &mut std::collections::HashSet::new(),
        )
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

    fn build_with(parameters: Vec<(&str, Value)>) -> Result<(), String> {
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

/// Attributes and codelists through `write_citygml_to_storage`.
/// Each world builds the same one-solid LOD1 building its own way; everything
/// after conversion is shared, so every test runs in both.
#[cfg(test)]
mod attributes_tests {
    use std::collections::HashMap;
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
    use super::classes::Prefixes;
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
            diagnostics: Option<&NodeDiagnosticsHandle>,
        ) -> String {
            self.write_with(
                feature,
                keys,
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

    /// A workflow writes a value it computed as a generic attribute by giving
    /// it the shape the reader gives one: a map with `@name` and `gen:value`,
    /// and `gen:value_uom` for a measure.
    #[test]
    fn hand_built_generic_attribute_maps_are_written() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[
            (
                "gen:stringAttribute",
                json!({ "@name": "用途", "gen:value": "住宅" }),
            ),
            (
                "gen:intAttribute",
                json!({ "@name": "階数", "gen:value": 3 }),
            ),
            (
                "gen:measureAttribute",
                json!({ "@name": "高さ", "gen:value": 12.5, "gen:value_uom": "m" }),
            ),
        ]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        for written in [
            r#"<gen:stringAttribute name="用途"><gen:value>住宅</gen:value></gen:stringAttribute>"#,
            r#"<gen:intAttribute name="階数"><gen:value>3</gen:value></gen:intAttribute>"#,
            r#"<gen:measureAttribute name="高さ"><gen:value uom="m">12.5</gen:value></gen:measureAttribute>"#,
        ] {
            assert!(gml.contains(written), "{written} in {gml}");
        }
        assert_eq!(reported(&handle, ErrorCode::CitygmlAttributeNotPlaced), 0);
        assert_eq!(reported(&handle, ErrorCode::CitygmlEmptyValueNotWritten), 0);
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

        let gml = job.write(feature, &keys, None);

        assert!(gml.contains("<bldg:class>3001</bldg:class>"), "{gml}");
        assert!(!gml.contains("9999"), "{gml}");
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

        job.write(feature, &ResolvedKeys::default(), Some(&handle));

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

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(gml.contains("<bldg:lod1Solid"), "{gml}");
        assert!(!gml.contains("buildingIDAttribute"), "{gml}");
        assert_eq!(reported(&handle, ErrorCode::CitygmlAttributeNotPlaced), 1);
    }

    /// Every code reported, with its count, from one drain of the handle.
    fn report_counts(handle: &NodeDiagnosticsHandle) -> HashMap<ErrorCode, u64> {
        let mut counts = HashMap::new();
        for summary in handle.inner.drain_summaries() {
            *counts.entry(summary.code).or_default() +=
                summary.aggregated.as_ref().map_or(1, |info| info.count);
        }
        counts
    }

    /// The codes the writer reports for attributes it could not place, summed.
    fn placement_reports(counts: &HashMap<ErrorCode, u64>) -> u64 {
        [
            ErrorCode::CitygmlAttributeNotPlaced,
            ErrorCode::CitygmlAttributeOverLimit,
            ErrorCode::CitygmlNestedObjectNotWritten,
        ]
        .iter()
        .filter_map(|code| counts.get(code))
        .sum()
    }

    fn attribute_reports(handle: &NodeDiagnosticsHandle) -> u64 {
        placement_reports(&report_counts(handle))
    }

    fn empty_reports(counts: &HashMap<ErrorCode, u64>) -> u64 {
        counts
            .get(&ErrorCode::CitygmlEmptyValueNotWritten)
            .copied()
            .unwrap_or(0)
    }

    /// As in FME, an empty string is no value: nothing is written for it, and
    /// it is reported so the gap can be filled upstream.
    #[test]
    fn an_empty_value_writes_nothing_and_is_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[("bldg:roofType", json!(""))]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(gml.contains("<bldg:measuredHeight"), "{gml}");
        assert!(!gml.contains("roofType"), "{gml}");
        let counts = report_counts(&handle);
        assert_eq!(empty_reports(&counts), 1);
        assert_eq!(placement_reports(&counts), 0);
    }

    /// `null` is no value: nothing was there, so nothing is written and
    /// nothing is reported.
    #[test]
    fn a_null_value_writes_nothing_and_is_not_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[
            ("bldg:function", json!(null)),
            ("bldg:class", json!({ "@codeSpace": null, "$": "3001" })),
        ]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(!gml.contains("bldg:function"), "{gml}");
        assert!(gml.contains("<bldg:class>3001</bldg:class>"), "{gml}");
        assert!(report_counts(&handle).is_empty());
    }

    /// `uom` describes a value and never stands for one, so a measure without
    /// a value writes nothing, and is reported.
    #[test]
    fn an_element_with_only_a_unit_is_not_written_and_is_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature =
            building_with_source_properties(&[("bldg:measuredHeight", json!({ "@uom": "m" }))]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(!gml.contains("measuredHeight"), "{gml}");
        assert_eq!(empty_reports(&report_counts(&handle)), 1);
    }

    /// An `xlink:href` is a reference, content of its own, so the element is
    /// written with nothing else and nothing is reported.
    #[test]
    fn an_element_holding_only_a_reference_is_written_and_not_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature =
            building_with_source_properties(&[("bldg:address", json!({ "@xlink:href": "#a1" }))]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(
            gml.contains(r##"<bldg:address xlink:href="#a1"/>"##),
            "{gml}"
        );
        assert!(report_counts(&handle).is_empty());
    }

    /// An empty XML attribute is left off; the value it described is still
    /// written.
    #[test]
    fn an_empty_code_space_is_left_off_the_value_and_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[(
            "bldg:class",
            json!({ "@codeSpace": "", "$": "3001" }),
        )]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(gml.contains("<bldg:class>3001</bldg:class>"), "{gml}");
        assert!(!gml.contains("codeSpace"), "{gml}");
        assert_eq!(empty_reports(&report_counts(&handle)), 1);
    }

    /// `gen:value` is required, so a generic attribute whose value is empty or
    /// null is dropped whole, as is a set left with no members. Each had a
    /// name, so each is reported.
    #[test]
    fn a_generic_attribute_with_an_empty_or_null_value_is_dropped_and_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[
            (
                "gen:stringAttribute",
                json!([
                    { "@name": "note", "gen:value": "" },
                    { "@name": "a", "gen:value": "x" },
                    { "@name": "gone", "gen:value": null },
                ]),
            ),
            (
                "gen:genericAttributeSet",
                json!({ "@name": "set", "gen:stringAttribute": { "@name": "inner", "gen:value": "" } }),
            ),
        ]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(
            gml.contains(
                r#"<gen:stringAttribute name="a"><gen:value>x</gen:value></gen:stringAttribute>"#
            ),
            "{gml}"
        );
        assert_eq!(gml.matches("<gen:").count(), 2, "{gml}");
        for gone in ["note", "gone", "set", "inner"] {
            assert!(!gml.contains(&format!(r#"name="{gone}""#)), "{gml}");
        }
        let counts = report_counts(&handle);
        assert_eq!(empty_reports(&counts), 1);
        assert_eq!(placement_reports(&counts), 0);
    }

    /// `core:Address` requires `core:xalAddress`, so an address whose only
    /// content was empty is dropped with its `bldg:address`, and reported.
    #[test]
    fn an_address_left_without_its_required_content_is_dropped_and_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[(
            "bldg:address",
            json!({ "core:Address": {
                "@gml:id": "addr1",
                "core:xalAddress": { "xAL:AddressDetails": { "xAL:Country": { "xAL:CountryName": "" } } }
            } }),
        )]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(gml.contains("<bldg:measuredHeight"), "{gml}");
        assert!(!gml.contains("address"), "{gml}");
        assert!(!gml.contains("addr1"), "{gml}");
        let counts = report_counts(&handle);
        assert_eq!(empty_reports(&counts), 1);
        assert_eq!(placement_reports(&counts), 0);
    }

    /// Placement drops an address that still has other content once an empty
    /// value took its required `core:xalAddress`, and its `bldg:address` with
    /// it, and reports that.
    #[test]
    fn an_address_with_other_content_is_dropped_without_its_required_part_and_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[(
            "bldg:address",
            json!({ "core:Address": {
                "@gml:id": "addr1",
                "core:xalAddress": { "xAL:AddressDetails": "" },
                "core:multiPoint": { "@xlink:href": "#p1" }
            } }),
        )]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(gml.contains("<bldg:measuredHeight"), "{gml}");
        assert!(!gml.contains("address"), "{gml}");
        assert!(!gml.contains("#p1"), "{gml}");
        let counts = report_counts(&handle);
        assert_eq!(empty_reports(&counts), 1);
        assert_eq!(placement_reports(&counts), 0);
    }

    /// The run summary counts each report as one affected feature, so however
    /// many values a feature loses, it is reported once, by its id.
    #[test]
    fn an_empty_value_is_reported_once_per_feature_with_its_id() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[
            ("bldg:roofType", json!("")),
            ("bldg:class", json!({ "@codeSpace": "", "$": "3001" })),
            ("bldg:storeysAboveGround", json!({ "@uom": "m" })),
            (
                "gen:stringAttribute",
                json!([{ "@name": "a", "gen:value": "x" }, { "@name": "b", "gen:value": "" }]),
            ),
        ]);
        let id = feature.id;

        job.write(feature, &ResolvedKeys::default(), Some(&handle));

        let summaries: Vec<_> = handle
            .inner
            .drain_summaries()
            .into_iter()
            .filter(|summary| summary.code == ErrorCode::CitygmlEmptyValueNotWritten)
            .collect();
        assert_eq!(summaries.len(), 1);
        let aggregated = summaries[0].aggregated.as_ref().unwrap();
        assert_eq!(aggregated.count, 1);
        assert_eq!(aggregated.sample_feature_ids, [id]);
    }

    /// Unprefixed top-level attributes are Flow data, never written, even when
    /// one is named like a property of the class.
    #[test]
    fn unprefixed_top_level_attributes_are_not_written_or_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let feature = building_with_source_properties(&[
            ("path", json!("a.gml")),
            ("totalSolarRadiation", json!(1.5)),
            ("name", json!("本館")),
            ("class", json!("3001")),
            ("measuredHeight", json!("12")),
        ]);

        let gml = job.write(feature, &ResolvedKeys::default(), Some(&handle));

        assert!(!gml.contains("本館"), "{gml}");
        assert!(!gml.contains(">3001<"), "{gml}");
        assert!(!gml.contains(">12<"), "{gml}");
        assert_eq!(attribute_reports(&handle), 0);
    }

    /// A configured map holds only what the reader read, so every unprefixed
    /// key in it is a source element.
    #[test]
    fn an_unprefixed_key_in_a_configured_map_is_reported() {
        let job = Job::new();
        let handle = diagnostics();
        let keys = serde_json::from_value::<AttributeKeys>(json!({ "cityGmlAttributes": "cga" }))
            .unwrap()
            .resolve()
            .unwrap();
        let feature = lod1_building(attributes(json!({
            "__citygml_feature_type": "bldg:Building",
            "cga": { "bldg:class": "3001", "buildingID": "13104-bldg-1" },
        })));

        let gml = job.write(feature, &keys, Some(&handle));

        assert!(gml.contains("<bldg:class>3001</bldg:class>"), "{gml}");
        assert_eq!(reported(&handle, ErrorCode::CitygmlAttributeNotPlaced), 1);
    }

    const URO: &str = "https://www.geospatial.jp/iur/uro/3.1";

    /// The PLATEAU fixture's i-UR 3.1 schema, with `uro` bound as `adeSchemas`
    /// binds it.
    fn iur() -> (SchemaSet, Prefixes) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../testing/data/fixtures/plateau-citymodel/15202_nagaoka-shi_city_2024_citygml_1_op/schemas/iur/uro/3.1/urbanObject.xsd");
        let set = SchemaSet::with_ade(&[path]).unwrap();
        let prefixes = Prefixes::with_ade(&[("uro".into(), URO.into())]);
        (set, prefixes)
    }

    /// A `uro:UndergroundBuilding` carrying an i-UR attribute, as the reader
    /// hands it over.
    fn underground_building_with_an_iur_attribute() -> Feature {
        lod1_building(attributes(json!({
            "__citygml_feature_type": "uro:UndergroundBuilding",
            "uro:buildingIDAttribute": {
                "uro:BuildingIDAttribute": {
                    "uro:buildingID": "13104-bldg-1",
                    "uro:city": "13104",
                },
            },
        })))
    }

    #[test]
    fn an_ade_attribute_is_placed_under_its_namespace_when_its_ade_is_listed() {
        let job = Job::new();
        let handle = diagnostics();
        let (schema, prefixes) = iur();

        let gml = job.write_with(
            underground_building_with_an_iur_attribute(),
            &ResolvedKeys::default(),
            Some(&handle),
            &schema,
            &prefixes,
        );

        assert!(gml.contains(&format!(r#"xmlns:uro="{URO}""#)), "{gml}");
        assert!(gml.contains("<uro:UndergroundBuilding "), "{gml}");
        let attribute = position(
            &gml,
            "<uro:buildingIDAttribute><uro:BuildingIDAttribute>\
             <uro:buildingID>13104-bldg-1</uro:buildingID><uro:city>13104</uro:city>\
             </uro:BuildingIDAttribute></uro:buildingIDAttribute>",
        );
        // `bldg:_GenericApplicationPropertyOfAbstractBuilding`, the hook i-UR
        // substitutes into, closes the inherited sequence, so it follows the
        // geometry rather than arriving first.
        let solid = position(&gml, "</bldg:lod1Solid>");
        assert!(solid < attribute, "{gml}");
        assert_eq!(reported(&handle, ErrorCode::CitygmlAttributeNotPlaced), 0);
    }

    #[test]
    fn an_ade_attribute_is_not_placed_when_its_ade_is_not_listed() {
        let job = Job::new();
        let handle = diagnostics();

        let gml = job.write(
            underground_building_with_an_iur_attribute(),
            &ResolvedKeys::default(),
            Some(&handle),
        );

        assert!(!gml.contains("uro:"), "{gml}");
        assert!(gml.contains("<gen:GenericCityObject "), "{gml}");
        let summaries = handle.inner.drain_summaries();
        let count = |code: ErrorCode| {
            summaries
                .iter()
                .filter(|summary| summary.code == code)
                .map(|summary| summary.aggregated.as_ref().map_or(1, |info| info.count))
                .sum::<u64>()
        };
        assert_eq!(count(ErrorCode::CitygmlAttributeNotPlaced), 1);
        assert_eq!(count(ErrorCode::CitygmlUnknownFeatureType), 1);
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

    pub(super) fn context(
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
        run_with(ctx, params(ade_schemas), features)
    }

    /// [`run`] with the writer's whole `with` map.
    pub(super) fn run_with(
        ctx: &NodeContext,
        with: HashMap<String, Value>,
        features: Vec<Feature>,
    ) -> Result<(), String> {
        let mut sink = CityGmlWriterFactory
            .build(
                ctx.clone(),
                EventHub::new(10),
                "CityGML Writer".to_string(),
                Some(with),
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
        let (xml, summaries) = write_reporting(ade_schemas, features);
        let unknown = summaries
            .iter()
            .filter(|summary| summary.code == ErrorCode::CitygmlUnknownFeatureType)
            .map(|summary| summary.aggregated.as_ref().unwrap().count)
            .sum();
        (xml, unknown)
    }

    /// The written document and every diagnostic summary the write produced.
    fn write_reporting(
        ade_schemas: Option<Value>,
        features: Vec<Feature>,
    ) -> (String, Vec<reearth_flow_diagnostics::Diagnostic>) {
        let tmp = tempdir().unwrap();
        let handle = diagnostics();
        let ctx = context(tmp.path(), Some(handle.clone()));
        run(&ctx, ade_schemas, features).unwrap();

        let xml = std::fs::read_to_string(tmp.path().join("out.gml")).unwrap();
        (xml, handle.inner.drain_summaries())
    }

    /// A diagnostics handle that keeps every report for the test to drain.
    pub(super) fn diagnostics() -> Arc<NodeDiagnosticsHandle> {
        Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "writer".into(),
            "CityGML Writer".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            true,
        ))
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

    /// `writable()` carries one LOD0 surface with no source property name, and
    /// `building.xsd` declares neither `lod0MultiSurface` nor `lod0Geometry`.
    /// The building is written without it, and the drop is reported against
    /// the feature.
    #[test]
    fn geometry_its_class_has_no_property_for_is_reported_with_the_feature_id() {
        let feature = typed("bldg:Building");
        let id = feature.id;

        let (xml, summaries) = write_reporting(None, vec![feature]);

        assert!(xml.contains("<bldg:Building "), "{xml}");
        assert!(!xml.contains("lod0MultiSurface"), "{xml}");
        assert!(!xml.contains("<gml:MultiSurface"), "{xml}");
        let reported: Vec<_> = summaries
            .iter()
            .filter(|summary| summary.code == ErrorCode::CitygmlGeometryNotWritten)
            .collect();
        assert_eq!(reported.len(), 1, "{summaries:?}");
        let aggregated = reported[0].aggregated.as_ref().unwrap();
        assert_eq!(aggregated.count, 1);
        assert_eq!(aggregated.sample_feature_ids, vec![id]);
    }

    #[test]
    fn geometry_its_class_declares_is_not_reported() {
        let (xml, summaries) = write_reporting(None, vec![typed("gen:GenericCityObject")]);

        assert!(xml.contains("<gen:lod0Geometry>"), "{xml}");
        assert!(
            !summaries
                .iter()
                .any(|summary| summary.code == ErrorCode::CitygmlGeometryNotWritten),
            "{summaries:?}"
        );
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

/// CityGML read by the reader and written back, nested objects included.
#[cfg(all(test, feature = "new-geometry"))]
mod hierarchy_tests {
    use std::collections::{HashMap, HashSet};

    use reearth_flow_citygml::parser::{CityGmlVersion, Parser};
    use reearth_flow_citygml::pipeline::build_features;
    use reearth_flow_diagnostics::{Diagnostic, ErrorCode};
    use reearth_flow_types::Feature;
    use serde_json::{json, Value};
    use tempfile::tempdir;
    use url::Url;

    use super::ade_schemas_tests::{context, diagnostics, run_with};

    const HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:gml="http://www.opengis.net/gml" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0">
<gml:boundedBy><gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3"><gml:lowerCorner>35.0 135.0 0</gml:lowerCorner><gml:upperCorner>35.001 135.001 10</gml:upperCorner></gml:Envelope></gml:boundedBy>
<core:cityObjectMember>"#;

    const FOOTER: &str = "</core:cityObjectMember>\n</core:CityModel>\n";

    /// A `property` holding one triangle at height `z`.
    fn surface(property: &str, z: u32) -> String {
        format!(
            "<bldg:{property}><gml:MultiSurface><gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing>\
             <gml:posList>35.0 135.0 {z} 35.0 135.001 {z} 35.001 135.0 {z} 35.0 135.0 {z}</gml:posList>\
             </gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember></gml:MultiSurface></bldg:{property}>"
        )
    }

    /// Building `b1`: wall `w1` with a name and LOD2 geometry, holding window
    /// `win1` with a name and LOD3 geometry, and building part `p1`.
    fn building() -> String {
        format!(
            r#"{HEADER}<bldg:Building gml:id="b1">
<bldg:boundedBy><bldg:WallSurface gml:id="w1"><gml:name>north wall</gml:name>{wall}
<bldg:opening><bldg:Window gml:id="win1"><gml:name>window</gml:name>{window}</bldg:Window></bldg:opening>
</bldg:WallSurface></bldg:boundedBy>
<bldg:consistsOfBuildingPart><bldg:BuildingPart gml:id="p1">{part}</bldg:BuildingPart></bldg:consistsOfBuildingPart>
</bldg:Building>{FOOTER}"#,
            wall = surface("lod2MultiSurface", 0),
            window = surface("lod3MultiSurface", 1),
            part = surface("lod2MultiSurface", 2),
        )
    }

    /// Read `files`, each a name and its content, with the CityGML 2 reader's
    /// pipeline, as the CityGML reader action does.
    fn read(
        files: &[(&str, &str)],
        tags: &[&str],
        keep_attributes: bool,
        attributes_key: Option<&str>,
    ) -> Vec<Feature> {
        let tags: HashSet<String> = tags.iter().map(|tag| tag.to_string()).collect();
        let mut parser = Parser::with_extract_tags(CityGmlVersion::V2, tags.clone());
        for (name, content) in files {
            let url = Url::parse(&format!("file:///data/{name}")).unwrap();
            parser.parse(content.as_bytes(), &url).unwrap();
        }
        build_features(
            parser,
            &tags,
            &HashMap::new(),
            attributes_key,
            keep_attributes,
            false,
            &[],
            false,
        )
    }

    /// Write `features` with the writer's `params`, unindented, and return the
    /// document and what was reported.
    fn write(features: Vec<Feature>, params: Value) -> (String, Vec<Diagnostic>) {
        let tmp = tempdir().unwrap();
        let handle = diagnostics();
        let ctx = context(tmp.path(), Some(handle.clone()));
        let mut with: HashMap<String, Value> = HashMap::from([
            (
                "output".to_string(),
                json!({"type": "string", "value": "out.gml"}),
            ),
            ("prettyPrint".to_string(), json!(false)),
        ]);
        if let Value::Object(params) = params {
            with.extend(params);
        }
        run_with(&ctx, with, features).unwrap();
        let xml = std::fs::read_to_string(tmp.path().join("out.gml")).unwrap();
        (xml, handle.inner.drain_summaries())
    }

    /// How many features `code` was reported for.
    fn reported(diagnostics: &[Diagnostic], code: ErrorCode) -> u64 {
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == code)
            .map(|diagnostic| diagnostic.aggregated.as_ref().map_or(1, |info| info.count))
            .sum()
    }

    /// The text inside the first element whose start tag begins with `start`,
    /// up to the first `end` after it.
    fn inside<'x>(xml: &'x str, start: &str, end: &str) -> &'x str {
        let from = xml.find(start).unwrap_or_else(|| panic!("{start}: {xml}"));
        let body = &xml[from..];
        let open = body.find('>').unwrap() + 1;
        let close = body.find(end).unwrap_or_else(|| panic!("{end}: {xml}"));
        &body[open..close]
    }

    fn assert_nested(xml: &str) {
        assert!(
            xml.contains(r#"<bldg:boundedBy><bldg:WallSurface gml:id="w1">"#),
            "{xml}"
        );
        assert!(
            inside(xml, "<bldg:WallSurface ", "</bldg:WallSurface>")
                .starts_with("<gml:name>north wall</gml:name>"),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<bldg:opening><bldg:Window gml:id="win1">"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<bldg:consistsOfBuildingPart><bldg:BuildingPart gml:id="p1">"#),
            "{xml}"
        );
        // One in the wall and one in the part: none was flattened onto the
        // building.
        assert_eq!(xml.matches("<bldg:lod2MultiSurface>").count(), 2, "{xml}");
        assert!(
            inside(xml, "<bldg:WallSurface ", "</bldg:WallSurface>")
                .contains("<bldg:lod2MultiSurface>"),
            "{xml}"
        );
        assert!(
            inside(xml, "<bldg:BuildingPart ", "</bldg:BuildingPart>")
                .contains("<bldg:lod2MultiSurface>"),
            "{xml}"
        );
    }

    #[test]
    fn a_read_building_is_written_with_walls_window_and_part_nested() {
        let features = read(&[("hierarchy.gml", &building())], &[], true, None);

        let (xml, diagnostics) = write(features, json!({}));

        assert_nested(&xml);
        assert!(
            inside(&xml, "<bldg:Window ", "</bldg:Window>")
                .starts_with("<gml:name>window</gml:name><bldg:lod3MultiSurface>"),
            "{xml}"
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn a_nested_object_whose_geometry_is_filtered_out_keeps_its_place_and_data() {
        let features = read(&[("hierarchy.gml", &building())], &[], true, None);

        let (xml, _) = write(features, json!({"lodFilter": [2]}));

        assert!(
            xml.contains(
                r#"<bldg:opening><bldg:Window gml:id="win1"><gml:name>window</gml:name></bldg:Window></bldg:opening>"#
            ),
            "{xml}"
        );
        assert!(!xml.contains("lod3"), "{xml}");
    }

    #[test]
    fn nested_objects_are_read_from_a_configured_attribute_map() {
        let features = read(&[("hierarchy.gml", &building())], &[], true, Some("cga"));

        let (xml, diagnostics) = write(
            features,
            json!({"attributeKeys": {"cityGmlAttributes": "cga"}}),
        );

        assert_nested(&xml);
        assert!(
            inside(&xml, "<bldg:Window ", "</bldg:Window>")
                .starts_with("<gml:name>window</gml:name><bldg:lod3MultiSurface>"),
            "{xml}"
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    /// With `keepAttributes` off the reader drops each nested object's
    /// `gml:id`, so nothing matches the geometry to its object: it is written
    /// on the feature and reported. The objects keep their data, with minted
    /// ids, and an object left with no data is not written and is reported.
    #[test]
    fn with_keep_attributes_off_nested_geometry_goes_on_the_feature_and_is_reported() {
        let features = read(&[("hierarchy.gml", &building())], &[], false, None);
        let id = features[0].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            xml.contains(
                r#"<bldg:WallSurface gml:id="wallsurface_1"><gml:name>north wall</gml:name><bldg:opening><bldg:Window gml:id="window_1"><gml:name>window</gml:name></bldg:Window></bldg:opening></bldg:WallSurface>"#
            ),
            "{xml}"
        );
        assert!(!xml.contains("<bldg:BuildingPart"), "{xml}");
        let building = inside(&xml, r#"<bldg:Building gml:id="b1""#, "</bldg:Building>");
        assert!(building.starts_with("<bldg:lod2MultiSurface>"), "{xml}");
        assert!(building.contains("<bldg:lod3MultiSurface>"), "{xml}");
        assert!(!xml.contains(r#"gml:id="w1""#), "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlNestedDataNotMatched),
            std::slice::from_ref(&id)
        );
        // The part, read with nothing but its dropped id, is an empty value.
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlEmptyValueNotWritten),
            [id]
        );
        assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    }

    /// Objects are matched by `gml:id` alone, so an id-less surface keeps its
    /// data, under a minted id, while its geometry goes on the feature and is
    /// reported. No reader-internal key is ever written.
    #[test]
    fn an_id_less_surfaces_geometry_goes_on_the_feature_and_is_reported() {
        let gml = format!(
            r#"{HEADER}<bldg:Building gml:id="b1">
<bldg:boundedBy><bldg:WallSurface><gml:name>id-less wall</gml:name>{wall}</bldg:WallSurface></bldg:boundedBy>
</bldg:Building>{FOOTER}"#,
            wall = surface("lod2MultiSurface", 0),
        );
        let features = read(&[("hierarchy.gml", &gml)], &[], true, None);
        let id = features[0].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            xml.contains(r#"<bldg:Building gml:id="b1"><bldg:lod2MultiSurface>"#),
            "{xml}"
        );
        assert!(
            xml.contains(
                r#"<bldg:boundedBy><bldg:WallSurface gml:id="wallsurface_1"><gml:name>id-less wall</gml:name></bldg:WallSurface></bldg:boundedBy>"#
            ),
            "{xml}"
        );
        assert!(!xml.contains("_synid"), "{xml}");
        assert!(!xml.contains("__citygml_synthetic"), "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlNestedDataNotMatched),
            [id]
        );
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    }

    #[test]
    fn a_nested_parts_lod1_shell_conflict_is_reported_against_its_feature() {
        let face = |a: &str, b: &str, c: &str| {
            format!(
                "<gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing>\
                 <gml:posList>{a} {b} {c} {a}</gml:posList>\
                 </gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>"
            )
        };
        let (o, x, y, z) = (
            "35.0 135.0 0",
            "35.001 135.0 0",
            "35.0 135.001 0",
            "35.0 135.0 10",
        );
        let solid = format!(
            "<bldg:lod1Solid><gml:Solid><gml:exterior><gml:Shell>{}{}{}{}</gml:Shell></gml:exterior></gml:Solid></bldg:lod1Solid>",
            face(o, y, x),
            face(o, x, z),
            face(o, z, y),
            face(x, y, z),
        );
        let gml = format!(
            r#"{HEADER}<bldg:Building gml:id="b1">
<bldg:consistsOfBuildingPart><bldg:BuildingPart gml:id="p1">{solid}{surface}</bldg:BuildingPart></bldg:consistsOfBuildingPart>
</bldg:Building>{FOOTER}"#,
            surface = surface("lod1MultiSurface", 0),
        );
        let features = read(&[("hierarchy.gml", &gml)], &[], true, None);

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            inside(&xml, "<bldg:BuildingPart ", "</bldg:BuildingPart>")
                .starts_with("<bldg:lod1Solid>"),
            "{xml}"
        );
        assert!(!xml.contains("lod1MultiSurface"), "{xml}");
        assert_eq!(
            reported(&diagnostics, ErrorCode::CitygmlLod1ShellConflict),
            1
        );
    }

    /// A nested installation's `lod2MultiSurface`, which its class does not
    /// declare, falls back to its `lod2Geometry` and is not reported. A wall's
    /// `lod1MultiSurface` has no property at all, so it is left out and
    /// reported once, against the wall's own feature rather than the
    /// building's.
    #[test]
    fn nested_geometry_falls_back_or_is_reported_against_its_own_feature() {
        let gml = format!(
            r#"{HEADER}<bldg:Building gml:id="b1">
<bldg:outerBuildingInstallation><bldg:BuildingInstallation gml:id="i1">{installation}</bldg:BuildingInstallation></bldg:outerBuildingInstallation>
<bldg:boundedBy><bldg:WallSurface gml:id="w1">{lod1}{lod2}</bldg:WallSurface></bldg:boundedBy>
</bldg:Building>{FOOTER}"#,
            installation = surface("lod2MultiSurface", 4),
            lod1 = surface("lod1MultiSurface", 3),
            lod2 = surface("lod2MultiSurface", 0),
        );
        let features = read(
            &[("hierarchy.gml", &gml)],
            &["Building", "WallSurface"],
            true,
            None,
        );
        let wall = features
            .iter()
            .find(|feature| gml_id_of(feature).as_deref() == Some("w1"))
            .unwrap()
            .id
            .to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            xml.contains(
                r#"<bldg:outerBuildingInstallation><bldg:BuildingInstallation gml:id="i1"><bldg:lod2Geometry><gml:MultiSurface"#
            ),
            "{xml}"
        );
        assert!(
            inside(&xml, "<bldg:WallSurface ", "</bldg:WallSurface>")
                .starts_with("<bldg:lod2MultiSurface>"),
            "{xml}"
        );
        assert_eq!(xml.matches("<bldg:lod2MultiSurface>").count(), 1, "{xml}");
        assert!(!xml.contains("lod1"), "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlGeometryNotWritten),
            vec![wall]
        );
    }

    #[test]
    fn a_reference_property_is_still_reported_as_not_written() {
        let gml = format!(
            r#"{HEADER}<bldg:Building gml:id="b1">{own}
<core:generalizesTo><bldg:Building gml:id="g1"><gml:name>generalized</gml:name></bldg:Building></core:generalizesTo>
</bldg:Building>{FOOTER}"#,
            own = surface("lod2MultiSurface", 0),
        );
        let features = read(&[("hierarchy.gml", &gml)], &[], true, None);

        let (xml, diagnostics) = write(features, json!({}));

        assert!(xml.contains(r#"<bldg:Building gml:id="b1">"#), "{xml}");
        assert!(!xml.contains("generalizesTo"), "{xml}");
        assert!(!xml.contains("g1"), "{xml}");
        assert_eq!(
            reported(&diagnostics, ErrorCode::CitygmlNestedObjectNotWritten),
            1
        );
    }

    /// The ids of the features `code` was reported for.
    fn reported_for(diagnostics: &[Diagnostic], code: ErrorCode) -> Vec<String> {
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == code)
            .flat_map(|diagnostic| {
                let samples = diagnostic
                    .aggregated
                    .as_ref()
                    .map(|info| info.sample_feature_ids.clone())
                    .unwrap_or_default();
                samples.into_iter().chain(diagnostic.feature_id)
            })
            .map(|id| id.to_string())
            .collect()
    }

    /// Every `gml:id` value in `xml`, in document order.
    fn gml_ids(xml: &str) -> Vec<&str> {
        xml.split("gml:id=\"")
            .skip(1)
            .map(|rest| &rest[..rest.find('"').unwrap()])
            .collect()
    }

    fn gml_id_of(feature: &Feature) -> Option<String> {
        feature
            .get(reearth_flow_types::CITYGML_GML_ID_KEY)
            .and_then(|value| value.as_string())
    }

    /// Extracting the building and the window but not the wall between them
    /// leaves the window naming the building as its parent, which cannot hold
    /// it: the window is written top-level and reported.
    #[test]
    fn extract_tags_skipping_the_wall_leave_the_window_top_level_and_reported() {
        let features = read(
            &[("hierarchy.gml", &building())],
            &["Building", "Window"],
            true,
            None,
        );
        assert_eq!(features.len(), 2);
        let window = features
            .iter()
            .find(|feature| gml_id_of(feature).as_deref() == Some("win1"))
            .unwrap()
            .id
            .to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            xml.contains(
                r#"<core:cityObjectMember><bldg:Window gml:id="win1"><gml:name>window</gml:name><bldg:lod3MultiSurface>"#
            ),
            "{xml}"
        );
        assert!(
            !inside(&xml, "<bldg:WallSurface ", "</bldg:WallSurface>").contains("<bldg:opening>"),
            "{xml}"
        );
        assert_eq!(xml.matches("<bldg:Building ").count(), 1, "{xml}");
        assert_eq!(xml.matches("<core:cityObjectMember>").count(), 2, "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlParentCannotHold),
            [window]
        );
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    }

    #[test]
    fn extract_tags_at_every_level_land_the_window_in_its_wall() {
        let features = read(
            &[("hierarchy.gml", &building())],
            &["Building", "WallSurface", "Window"],
            true,
            None,
        );
        assert_eq!(features.len(), 3);

        let (xml, diagnostics) = write(features, json!({}));

        assert_nested(&xml);
        assert!(
            inside(&xml, "<bldg:WallSurface ", "</bldg:WallSurface>").contains(
                r#"<bldg:opening><bldg:Window gml:id="win1"><gml:name>window</gml:name>"#
            ),
            "{xml}"
        );
        assert_eq!(xml.matches("<bldg:Building ").count(), 1, "{xml}");
        assert_eq!(xml.matches("<core:cityObjectMember>").count(), 1, "{xml}");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn an_extracted_feature_nests_under_its_parent_gml_id() {
        let features = read(
            &[("hierarchy.gml", &building())],
            &["Building", "WallSurface"],
            true,
            None,
        );
        let wall = features
            .iter()
            .find(|feature| gml_id_of(feature).as_deref() == Some("w1"))
            .unwrap();
        assert_eq!(
            wall.get(reearth_flow_types::CITYGML_PARENT_GML_ID_KEY)
                .and_then(|value| value.as_string())
                .as_deref(),
            Some("b1")
        );

        let (xml, diagnostics) = write(features, json!({}));

        assert_nested(&xml);
        assert_eq!(xml.matches("<core:cityObjectMember>").count(), 1, "{xml}");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn a_geometry_less_parent_of_joined_children_is_written_and_not_reported_empty() {
        let features = read(
            &[("hierarchy.gml", &building())],
            &["Building", "WallSurface", "BuildingPart"],
            true,
            None,
        );
        let b1 = features
            .iter()
            .find(|feature| gml_id_of(feature).as_deref() == Some("b1"))
            .unwrap();
        assert!(!b1.has_geometry());

        let (xml, diagnostics) = write(features, json!({}));

        assert!(xml.contains(r#"<bldg:Building gml:id="b1">"#), "{xml}");
        assert_nested(&xml);
        assert_eq!(reported(&diagnostics, ErrorCode::CitygmlEmptyGeometry), 0);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn a_geometry_less_feature_with_nothing_joined_is_still_reported_empty() {
        let bare = format!(
            r#"{HEADER}<bldg:Building gml:id="b2"><gml:name>bare</gml:name></bldg:Building>{FOOTER}"#
        );
        let features = read(
            &[("hierarchy.gml", &building()), ("bare.gml", &bare)],
            &[],
            true,
            None,
        );
        assert_eq!(features.len(), 2);
        let bare_id = features[1].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert_nested(&xml);
        assert!(!xml.contains(r#"gml:id="b2""#), "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlEmptyGeometry),
            [bare_id]
        );
    }

    #[test]
    fn the_same_document_read_twice_writes_one_complete_copy() {
        let doc = building();
        let features = read(&[("a.gml", &doc), ("a.gml", &doc)], &[], true, None);
        assert_eq!(features.len(), 2);
        let second = features[1].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert_nested(&xml);
        assert_eq!(xml.matches(r#"gml:id="w1""#).count(), 1, "{xml}");
        let ids = gml_ids(&xml);
        let unique: HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "{ids:?}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten),
            std::slice::from_ref(&second)
        );
        // Left with nothing to write once its copies are left out, the
        // second building is reported as a copy, not as empty.
        assert_eq!(reported(&diagnostics, ErrorCode::CitygmlEmptyGeometry), 0);
    }

    #[test]
    fn a_document_read_twice_writes_its_building_with_own_geometry_once() {
        let doc = building().replace(
            r#"<bldg:Building gml:id="b1">"#,
            &format!(
                r#"<bldg:Building gml:id="b1">{}"#,
                surface("lod2MultiSurface", 5)
            ),
        );
        let features = read(
            &[("a.gml", &doc), ("a.gml", &doc)],
            &["Building"],
            true,
            None,
        );
        assert_eq!(features.len(), 2);
        assert!(features.iter().all(|feature| feature.has_geometry()));
        let second = features[1].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            xml.contains(r#"<bldg:boundedBy><bldg:WallSurface gml:id="w1">"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<bldg:opening><bldg:Window gml:id="win1">"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<bldg:consistsOfBuildingPart><bldg:BuildingPart gml:id="p1">"#),
            "{xml}"
        );
        // The building's own, the wall's and the part's: one copy of each.
        assert_eq!(xml.matches("<bldg:lod2MultiSurface>").count(), 3, "{xml}");
        assert_eq!(xml.matches("<bldg:Building ").count(), 1, "{xml}");
        for id in ["b1", "w1", "win1", "p1"] {
            assert_eq!(
                xml.matches(&format!(r#"gml:id="{id}""#)).count(),
                1,
                "{id}: {xml}"
            );
        }
        let ids = gml_ids(&xml);
        let unique: HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "{ids:?}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten),
            [second]
        );
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    }

    #[test]
    fn an_emptied_wrapper_is_absent_but_an_empty_value_is_still_reported() {
        let other = format!(
            r#"{HEADER}<bldg:Building gml:id="b2">{own}</bldg:Building>{FOOTER}"#,
            own = surface("lod2MultiSurface", 0),
        );
        let mut features = read(
            &[("hierarchy.gml", &building()), ("other.gml", &other)],
            &["Building", "WallSurface", "Window"],
            true,
            None,
        );
        let wall = features
            .iter()
            .find(|feature| gml_id_of(feature).as_deref() == Some("w1"))
            .unwrap();
        // The reader leaves the extracted window's wrapper as an empty value.
        assert_eq!(
            wall.get("bldg:opening"),
            Some(&reearth_flow_types::AttributeValue::String(String::new())),
            "{wall:?}"
        );
        let b2 = features
            .iter_mut()
            .find(|feature| gml_id_of(feature).as_deref() == Some("b2"))
            .unwrap();
        b2.insert(
            "bldg:measuredHeight",
            reearth_flow_types::AttributeValue::String(String::new()),
        );
        let b2_id = b2.id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            xml.contains(r#"<bldg:opening><bldg:Window gml:id="win1">"#),
            "{xml}"
        );
        assert!(xml.contains(r#"<bldg:Building gml:id="b2">"#), "{xml}");
        assert!(!xml.contains("measuredHeight"), "{xml}");
        // Only the empty value, not the emptied wrapper of the wall.
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlEmptyValueNotWritten),
            [b2_id]
        );
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    }

    #[test]
    fn an_inlined_xlink_copy_is_written_once() {
        let gml = format!(
            r##"{HEADER}<bldg:Building gml:id="b1">
<bldg:consistsOfBuildingPart><bldg:BuildingPart gml:id="p1">{part}</bldg:BuildingPart></bldg:consistsOfBuildingPart>
</bldg:Building></core:cityObjectMember>
<core:cityObjectMember><bldg:Building gml:id="b2">{own}
<bldg:consistsOfBuildingPart xlink:href="#p1"/>
</bldg:Building>{FOOTER}"##,
            part = surface("lod2MultiSurface", 0),
            own = surface("lod2MultiSurface", 1),
        )
        .replace(
            "xmlns:gml=",
            r#"xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:gml="#,
        );
        let features = read(&[("xlink.gml", &gml)], &[], true, None);
        assert_eq!(features.len(), 2);
        let b2 = features[1].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert_eq!(xml.matches(r#"gml:id="p1""#).count(), 1, "{xml}");
        assert!(
            inside(&xml, r#"<bldg:Building gml:id="b1""#, "</bldg:Building>")
                .contains(r#"<bldg:BuildingPart gml:id="p1">"#),
            "{xml}"
        );
        assert!(xml.contains(r#"<bldg:Building gml:id="b2">"#), "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten),
            [b2]
        );
    }

    /// With every level extracted, the reader gives an extracted id-less
    /// wall a generated `gml:id` (`_synid<n>`) and its window that id as
    /// parent, so the window lands in its own document's wall. The writer
    /// takes the generated id as the wall's, as it takes any `gml:id`.
    #[test]
    fn an_extracted_id_less_wall_holds_its_window_by_the_readers_generated_id() {
        let doc = |building: &str, window: &str| {
            format!(
                r#"{HEADER}<bldg:Building gml:id="{building}">
<bldg:boundedBy><bldg:WallSurface>{wall}
<bldg:opening><bldg:Window gml:id="{window}">{glass}</bldg:Window></bldg:opening>
</bldg:WallSurface></bldg:boundedBy>
</bldg:Building>{FOOTER}"#,
                wall = surface("lod2MultiSurface", 0),
                glass = surface("lod3MultiSurface", 1),
            )
        };
        let features = read(
            &[
                ("x/a.gml", &doc("b1", "win1")),
                ("y/a.gml", &doc("b2", "win2")),
            ],
            &["Building", "WallSurface", "Window"],
            true,
            None,
        );
        assert_eq!(features.len(), 6);

        let (xml, diagnostics) = write(features, json!({}));

        for (building, window) in [("b1", "win1"), ("b2", "win2")] {
            let tree = inside(
                &xml,
                &format!(r#"<bldg:Building gml:id="{building}""#),
                "</bldg:Building>",
            );
            assert!(
                tree.starts_with(r#"<bldg:boundedBy><bldg:WallSurface gml:id="_synid"#),
                "{xml}"
            );
            assert!(
                tree.contains(&format!(r#"<bldg:opening><bldg:Window gml:id="{window}">"#)),
                "{xml}"
            );
        }
        assert_eq!(xml.matches("<core:cityObjectMember>").count(), 2, "{xml}");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    /// Extracting the building and the window but not the id-less wall
    /// between them: the window is top-level, and the wall, left with no
    /// data, is not written, its geometry going on the building. Both are
    /// reported.
    #[test]
    fn a_window_of_an_unextracted_id_less_wall_is_top_level_and_reported() {
        let gml = format!(
            r#"{HEADER}<bldg:Building gml:id="b1">
<bldg:boundedBy><bldg:WallSurface>{wall}
<bldg:opening><bldg:Window gml:id="win1">{glass}</bldg:Window></bldg:opening>
</bldg:WallSurface></bldg:boundedBy>
</bldg:Building>{FOOTER}"#,
            wall = surface("lod2MultiSurface", 0),
            glass = surface("lod3MultiSurface", 1),
        );
        let features = read(&[("a.gml", &gml)], &["Building", "Window"], true, None);
        let id_of = |gml_id: &str| {
            features
                .iter()
                .find(|feature| gml_id_of(feature).as_deref() == Some(gml_id))
                .unwrap()
                .id
                .to_string()
        };
        let (building, window) = (id_of("b1"), id_of("win1"));

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            xml.contains(r#"<bldg:Building gml:id="b1"><bldg:lod2MultiSurface>"#),
            "{xml}"
        );
        assert!(!xml.contains("<bldg:WallSurface"), "{xml}");
        assert!(
            xml.contains(r#"<core:cityObjectMember><bldg:Window gml:id="win1">"#),
            "{xml}"
        );
        assert!(!xml.contains("_synid"), "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlNestedDataNotMatched),
            [building]
        );
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlParentCannotHold),
            [window]
        );
        assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    }

    /// `HEADER` with the xlink and group namespaces declared too.
    fn header_with_groups() -> String {
        HEADER.replace(
            "xmlns:gml=",
            r#"xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xmlns:gml="#,
        )
    }

    #[test]
    fn a_group_listed_before_its_building_leaves_the_room_in_the_building() {
        // The PLATEAU underground building shape: a group naming a room of the
        // building and the building itself, both through xlinks, ahead of it.
        let gml = format!(
            r##"{header}<grp:CityObjectGroup gml:id="g1"><gml:name>floor</gml:name>
<grp:groupMember xlink:href="#r1"/>
<grp:parent xlink:href="#b1"/>
</grp:CityObjectGroup></core:cityObjectMember>
<core:cityObjectMember><bldg:Building gml:id="b1">{own}
<bldg:interiorRoom><bldg:Room gml:id="r1"><bldg:function>office</bldg:function>{room}</bldg:Room></bldg:interiorRoom>
</bldg:Building>{FOOTER}"##,
            header = header_with_groups(),
            own = surface("lod2MultiSurface", 0),
            room = surface("lod4MultiSurface", 1),
        );
        let features = read(&[("group.gml", &gml)], &[], true, None);
        assert_eq!(features.len(), 2);
        let group = features[0].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            inside(&xml, r#"<bldg:Building gml:id="b1""#, "</bldg:Building>").contains(
                r#"<bldg:interiorRoom><bldg:Room gml:id="r1"><bldg:function>office</bldg:function><bldg:lod4MultiSurface>"#
            ),
            "{xml}"
        );
        assert_eq!(xml.matches("<bldg:Room ").count(), 1, "{xml}");
        assert_eq!(xml.matches("<core:cityObjectMember>").count(), 1, "{xml}");
        assert_eq!(
            reported(&diagnostics, ErrorCode::CitygmlParentCannotHold),
            0
        );
        assert_eq!(
            reported(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten),
            0
        );
        // The group has no geometry of its own, so it is not written, and
        // that is reported.
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlEmptyGeometry),
            [group]
        );
    }

    #[test]
    fn walls_sharing_a_key_in_two_buildings_are_both_written() {
        let doc = |building: &str, z: u32| {
            format!(
                r#"{HEADER}<bldg:Building gml:id="{building}">
<bldg:boundedBy><bldg:WallSurface gml:id="w1">{wall}</bldg:WallSurface></bldg:boundedBy>
</bldg:Building>{FOOTER}"#,
                wall = surface("lod2MultiSurface", z),
            )
        };
        let features = read(
            &[("a.gml", &doc("b1", 0)), ("b.gml", &doc("b2", 3))],
            &[],
            true,
            None,
        );

        let (xml, diagnostics) = write(features, json!({}));

        // The second wall keeps its place and geometry under a generated id.
        for (building, wall, z) in [("b1", "w1", 0), ("b2", "wallsurface_1", 3)] {
            let body = inside(
                &xml,
                &format!(r#"<bldg:Building gml:id="{building}""#),
                "</bldg:Building>",
            );
            assert!(
                body.contains(&format!(
                    r#"<bldg:boundedBy><bldg:WallSurface gml:id="{wall}">"#
                )),
                "{xml}"
            );
            assert!(body.contains(&format!("<gml:posList>35 135 {z} ")), "{xml}");
        }
        assert_eq!(xml.matches(r#"gml:id="w1""#).count(), 1, "{xml}");
        let ids = gml_ids(&xml);
        let unique: HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "{ids:?}");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn a_document_read_twice_without_extract_tags_writes_its_building_once() {
        let doc = building().replace(
            r#"<bldg:Building gml:id="b1">"#,
            &format!(
                r#"<bldg:Building gml:id="b1">{}"#,
                surface("lod2MultiSurface", 5)
            ),
        );
        let features = read(&[("a.gml", &doc), ("a.gml", &doc)], &[], true, None);
        assert_eq!(features.len(), 2);
        let second = features[1].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        // The building's own, the wall's and the part's: one copy of each.
        assert_eq!(xml.matches("<bldg:lod2MultiSurface>").count(), 3, "{xml}");
        assert_eq!(xml.matches("<bldg:Building ").count(), 1, "{xml}");
        for id in ["b1", "w1", "win1", "p1"] {
            assert_eq!(
                xml.matches(&format!(r#"gml:id="{id}""#)).count(),
                1,
                "{id}: {xml}"
            );
        }
        let ids = gml_ids(&xml);
        let unique: HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "{ids:?}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten),
            [second]
        );
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    }

    #[test]
    fn a_geometry_less_feature_that_loses_only_a_nested_copy_is_reported_empty() {
        let gml = format!(
            r##"{HEADER}<bldg:Building gml:id="b1">
<bldg:consistsOfBuildingPart><bldg:BuildingPart gml:id="p1">{part}</bldg:BuildingPart></bldg:consistsOfBuildingPart>
</bldg:Building></core:cityObjectMember>
<core:cityObjectMember><bldg:Building gml:id="b2"><gml:name>no geometry</gml:name>
<bldg:consistsOfBuildingPart xlink:href="#p1"/>
</bldg:Building>{FOOTER}"##,
            part = surface("lod2MultiSurface", 0),
        )
        .replace(
            "xmlns:gml=",
            r#"xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:gml="#,
        );
        let features = read(&[("xlink.gml", &gml)], &[], true, None);
        assert_eq!(features.len(), 2);
        let b2 = features[1].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert_eq!(xml.matches(r#"gml:id="p1""#).count(), 1, "{xml}");
        assert!(!xml.contains(r#"gml:id="b2""#), "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten),
            std::slice::from_ref(&b2)
        );
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlEmptyGeometry),
            [b2]
        );
    }

    #[test]
    fn an_extracted_room_stays_in_its_building_when_a_group_comes_first() {
        // The PLATEAU underground building shape read with extractTags: the
        // reader also extracts the room and the building from the group's
        // xlinks, ahead of the real ones, and those copies name the group, or
        // the building copy reached through it, as their parent.
        let gml = format!(
            r##"{header}<grp:CityObjectGroup gml:id="g1"><gml:name>floor</gml:name>
<grp:groupMember xlink:href="#r1"/>
<grp:parent xlink:href="#b1"/>
</grp:CityObjectGroup></core:cityObjectMember>
<core:cityObjectMember><bldg:Building gml:id="b1">{own}
<bldg:interiorRoom><bldg:Room gml:id="r1"><bldg:function>office</bldg:function>{room}</bldg:Room></bldg:interiorRoom>
</bldg:Building>{FOOTER}"##,
            header = header_with_groups(),
            own = surface("lod2MultiSurface", 0),
            room = surface("lod4MultiSurface", 1),
        );
        let features = read(&[("group.gml", &gml)], &["Building", "Room"], true, None);
        let document = |feature: &Feature| {
            feature
                .get(reearth_flow_types::CITYGML_ROOT_GML_ID_KEY)
                .and_then(|value| value.as_string())
        };
        let (through_group, real): (Vec<&Feature>, Vec<&Feature>) = features
            .iter()
            .partition(|feature| document(feature).as_deref() == Some("g1"));
        // The real building and room come after their copies from the group.
        assert_eq!(real.len(), 2, "{features:?}");
        assert!(!through_group.is_empty());
        let copies: HashSet<String> = through_group.iter().map(|f| f.id.to_string()).collect();
        let real: Vec<String> = real.iter().map(|f| f.id.to_string()).collect();
        assert!(copies.contains(&features[0].id.to_string()));

        let (xml, diagnostics) = write(features, json!({}));

        assert!(
            inside(&xml, r#"<bldg:Building gml:id="b1""#, "</bldg:Building>").contains(
                r#"<bldg:interiorRoom><bldg:Room gml:id="r1"><bldg:function>office</bldg:function><bldg:lod4MultiSurface>"#
            ),
            "{xml}"
        );
        assert_eq!(xml.matches("<bldg:Room ").count(), 1, "{xml}");
        assert_eq!(xml.matches("<bldg:Building ").count(), 1, "{xml}");
        assert_eq!(xml.matches("<core:cityObjectMember>").count(), 1, "{xml}");
        assert_eq!(
            reported(&diagnostics, ErrorCode::CitygmlParentNotFound),
            0,
            "{diagnostics:?}"
        );
        // Only the copies reached through the group are reported as copies.
        let duplicates = reported_for(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten);
        assert!(
            duplicates.iter().all(|id| copies.contains(id)),
            "{duplicates:?} {real:?}"
        );
        for id in &real {
            assert!(!duplicates.contains(id), "{duplicates:?} {real:?}");
        }
    }

    /// Building `b1` with geometry of its own and one id-less wall, the shape
    /// of the `hierarchy_idless` input.
    fn building_with_an_id_less_wall() -> String {
        format!(
            r#"{HEADER}<bldg:Building gml:id="b1">{own}
<bldg:boundedBy><bldg:WallSurface>{wall}</bldg:WallSurface></bldg:boundedBy>
</bldg:Building>{FOOTER}"#,
            own = surface("lod2MultiSurface", 5),
            wall = surface("lod2MultiSurface", 0),
        )
    }

    #[test]
    fn a_document_with_id_less_objects_read_twice_writes_its_building_once() {
        let doc = building_with_an_id_less_wall();
        let features = read(&[("a.gml", &doc), ("a.gml", &doc)], &[], true, None);
        assert_eq!(features.len(), 2);
        let first = features[0].id.to_string();
        let second = features[1].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        // The id-less wall's geometry goes on the building, which leaves the
        // wall with nothing to write.
        assert_eq!(xml.matches("<bldg:Building ").count(), 1, "{xml}");
        assert!(!xml.contains("<bldg:WallSurface"), "{xml}");
        assert_eq!(xml.matches("<bldg:lod2MultiSurface>").count(), 1, "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten),
            [second]
        );
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlNestedDataNotMatched),
            std::slice::from_ref(&first)
        );
        // The wall, read with nothing but its geometry, is an empty value.
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlEmptyValueNotWritten),
            [first]
        );
        assert_eq!(diagnostics.len(), 3, "{diagnostics:?}");
    }

    #[test]
    fn an_inlined_xlink_copy_holding_an_id_less_object_is_written_once() {
        let gml = format!(
            r##"{HEADER}<bldg:Building gml:id="b1">
<bldg:consistsOfBuildingPart><bldg:BuildingPart gml:id="p1">{part}
<bldg:boundedBy><bldg:WallSurface>{wall}</bldg:WallSurface></bldg:boundedBy>
</bldg:BuildingPart></bldg:consistsOfBuildingPart>
</bldg:Building></core:cityObjectMember>
<core:cityObjectMember><bldg:Building gml:id="b2">{own}
<bldg:consistsOfBuildingPart xlink:href="#p1"/>
</bldg:Building>{FOOTER}"##,
            part = surface("lod2MultiSurface", 0),
            wall = surface("lod2MultiSurface", 2),
            own = surface("lod2MultiSurface", 1),
        )
        .replace(
            "xmlns:gml=",
            r#"xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:gml="#,
        );
        let features = read(&[("xlink.gml", &gml)], &[], true, None);
        assert_eq!(features.len(), 2);
        let b1 = features[0].id.to_string();
        let b2 = features[1].id.to_string();

        let (xml, diagnostics) = write(features, json!({}));

        assert_eq!(xml.matches("<bldg:BuildingPart ").count(), 1, "{xml}");
        // The id-less wall's geometry goes on its feature, b1, leaving the
        // wall nothing to write.
        assert!(!xml.contains("<bldg:WallSurface"), "{xml}");
        assert!(
            xml.contains(r#"<bldg:Building gml:id="b1"><bldg:lod2MultiSurface>"#),
            "{xml}"
        );
        assert!(
            inside(&xml, r#"<bldg:Building gml:id="b1""#, "</bldg:Building>")
                .contains(r#"<bldg:BuildingPart gml:id="p1">"#),
            "{xml}"
        );
        assert!(xml.contains(r#"<bldg:Building gml:id="b2">"#), "{xml}");
        assert_eq!(
            reported_for(&diagnostics, ErrorCode::CitygmlDuplicateObjectNotWritten),
            std::slice::from_ref(&b2)
        );
        // The inlined copy's id-less wall cannot be told from the original,
        // so its geometry goes on b2 as well, and is reported there too.
        assert_eq!(
            inside(&xml, r#"<bldg:Building gml:id="b2""#, "</bldg:Building>")
                .matches("<gml:Polygon>")
                .count(),
            2,
            "{xml}"
        );
        let mut unmatched = reported_for(&diagnostics, ErrorCode::CitygmlNestedDataNotMatched);
        unmatched.sort();
        let mut both = vec![b1, b2.clone()];
        both.sort();
        assert_eq!(unmatched, both);
    }
}
