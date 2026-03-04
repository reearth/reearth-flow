use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Cursor},
    str::FromStr,
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;
use nusamai_citygml::{
    CityGmlElement, CityGmlReader, Envelope, GeometryStore, ParseError, SubTreeReader,
};
use nusamai_plateau::{
    appearance::AppearanceStore, models, Entity, FlattenTreeTransform, GeometricMergedownTransform,
};
use quick_xml::NsReader;
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    executor_operation::{Context, ExecutorContext},
    forwarder::ProcessorChannelForwarder,
    node::DEFAULT_PORT,
};
use reearth_flow_types::{
    conversion::nusamai::entity_to_geometry, geometry::Geometry, lod::LodMask, metadata::Metadata,
    Attribute, AttributeValue, Feature,
};
use url::Url;

use crate::feature::errors::FeatureProcessorError;

/// Resolve geometry refs, build per-child attributes, and emit a single pre-flattened entity.
/// Both the main emit loop and the cross-file feature-ref path call this so attribute keys
/// only need to be maintained in one place.
#[allow(clippy::too_many_arguments)]
fn emit_flat_entity(
    ctx: &Context,
    fw: &ProcessorChannelForwarder,
    mut ent: Entity,
    root_needs_reconstruction: bool,
    // Already contains base_attributes + gmlRootId + root featureType + gmlId + maxLod.
    // This function overrides the child-specific fields (featureType, gmlName, lod, gmlId).
    mut attrs: HashMap<Attribute, AttributeValue>,
    root_metadata: &Metadata,
    transformer: &mut GeometricMergedownTransform,
    flatten: bool,
    parent_lod: LodMask,
    geom_registry: &HashMap<Url, Arc<RwLock<GeometryStore>>>,
    app_registry: &HashMap<Url, Arc<RwLock<AppearanceStore>>>,
) -> Result<(), BoxedError> {
    // Resolve geometry refs
    {
        let mut geom_store = ent.geometry_store.write().unwrap();
        let nusamai_citygml::Value::Object(obj) = &mut ent.root else {
            return Ok(());
        };
        if let nusamai_citygml::object::ObjectStereotype::Feature {
            ref mut geometries, ..
        } = obj.stereotype
        {
            geom_store.resolve_refs(geometries);

            let has_cross_file = geometries
                .iter()
                .any(|r| r.unresolved_refs.iter().any(|(f, _, _)| f.is_some()));
            if has_cross_file {
                let old_ring_count = geom_store.ring_ids.len();
                let old_span_count = geom_store.surface_spans.len();
                let src_app_stores: Vec<Arc<RwLock<AppearanceStore>>> = {
                    let mut seen = std::collections::HashSet::new();
                    geometries
                        .iter()
                        .flat_map(|r| r.unresolved_refs.iter())
                        .filter(|(f, _, _)| f.is_some())
                        .filter_map(|(f, href, _)| {
                            let mut url = f.clone().unwrap();
                            url.set_fragment(Some(&href.0));
                            app_registry.get(&url).map(Arc::clone)
                        })
                        .filter(|a| seen.insert(Arc::as_ptr(a) as usize))
                        .collect()
                };
                geom_store.resolve_cross_file_refs(geometries, geom_registry);
                if !src_app_stores.is_empty() {
                    let new_ring_ids = geom_store.ring_ids[old_ring_count..].to_vec();
                    let new_surface_spans = geom_store.surface_spans[old_span_count..].to_vec();
                    let mut app_store = ent.appearance_store.write().unwrap();
                    for src_app_arc in src_app_stores {
                        let mut src_app = (*src_app_arc.read().unwrap()).clone();
                        app_store.merge_global(&mut src_app, &new_ring_ids, &new_surface_spans);
                    }
                }
            }
        }
    }

    let nusamai_citygml::Value::Object(obj) = &ent.root else {
        return Ok(());
    };
    let mut child_lod = LodMask::default();
    if let nusamai_citygml::object::ObjectStereotype::Feature { geometries, .. } = &obj.stereotype {
        for geom in geometries {
            child_lod.add_lod(geom.lod);
        }
    } else {
        return Ok(());
    }
    transformer.transform(&mut ent);

    let child_id = ent.id.clone();
    let child_typename = ent.typename.clone();
    if flatten {
        if let Some(typename) = &child_typename {
            if typename != "uro:DmGeometricAttribute" {
                attrs.insert(
                    Attribute::new("featureType"),
                    AttributeValue::String(typename.to_string()),
                );
                attrs.insert(
                    Attribute::new("gmlName"),
                    AttributeValue::String(typename.to_string()),
                );
            }
        }
        let effective_lod = child_lod.highest_lod().or_else(|| parent_lod.highest_lod());
        if let Some(max_lod) = effective_lod {
            attrs.insert(
                Attribute::new("lod"),
                AttributeValue::String(max_lod.to_string()),
            );
        }
    }

    let citygml_attributes =
        AttributeValue::Map(AttributeValue::from_nusamai_citygml_value(&ent.root));
    let geometry: Geometry = entity_to_geometry(ent, root_needs_reconstruction)
        .map_err(|e| FeatureProcessorError::FileCityGmlReader(format!("{e:?}")))?;
    let mut feature: Feature = geometry.into();
    feature.extend(attrs);
    feature.insert("cityGmlAttributes", citygml_attributes);
    let mut child_metadata = root_metadata.clone();
    if flatten {
        if child_lod.highest_lod().is_some() {
            child_metadata.lod = Some(child_lod);
        }
        child_metadata.feature_id = child_id;
        child_metadata.feature_type = child_typename;
    }
    feature.metadata = child_metadata;
    fw.send(ExecutorContext::new_with_context_feature_and_port(
        ctx,
        feature,
        DEFAULT_PORT.clone(),
    ));
    Ok(())
}

#[derive(Debug, Clone)]
pub(super) struct BufferedEntity {
    pub entity: Entity,
    pub base_attributes: IndexMap<Attribute, AttributeValue>,
    pub flatten: bool,
    pub base_url: Url,
    pub root_needs_reconstruction: bool,
}

/// Pass 1: parse a CityGML file, populate geometry/appearance registries, buffer entities.
/// Does not emit any features.
#[allow(clippy::uninlined_format_args, clippy::too_many_arguments)]
pub(super) fn parse_and_register(
    ctx: Context,
    feature: Feature,
    dataset: rhai::AST,
    original_dataset: reearth_flow_types::Expr,
    flatten: Option<bool>,
    global_params: Option<HashMap<String, serde_json::Value>>,
    codelists_url: Option<Url>,
    geom_registry: &mut HashMap<Url, Arc<RwLock<GeometryStore>>>,
    app_registry: &mut HashMap<Url, Arc<RwLock<AppearanceStore>>>,
    buffered: &mut Vec<BufferedEntity>,
) -> Result<(), FeatureProcessorError> {
    let code_resolver = if let Some(codelists_path) = codelists_url {
        nusamai_plateau::codelist::Resolver::with_fallback(vec![codelists_path])
    } else {
        nusamai_plateau::codelist::Resolver::new()
    };
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let scope = feature.new_scope(expr_engine.clone(), &global_params);
    let city_gml_path = scope
        .eval_ast::<String>(&dataset)
        .unwrap_or_else(|_| original_dataset.to_string());
    let input_path = Uri::from_str(city_gml_path.as_str()).map_err(|e| {
        FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    let storage = storage_resolver.resolve(&input_path).map_err(|e| {
        FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    let byte = storage.get_sync(input_path.path().as_path()).map_err(|e| {
        FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    let cursor = Cursor::new(byte);
    let buf_reader = BufReader::new(cursor);
    let base_url: Url = input_path.into();
    let mut xml_reader = NsReader::from_reader(buf_reader);
    let context = nusamai_citygml::ParseContext::new(base_url.clone(), &code_resolver);
    let mut citygml_reader = CityGmlReader::new(context);
    let mut st = citygml_reader.start_root(&mut xml_reader).map_err(|e| {
        FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;

    collect_entities(
        &mut st,
        &feature.attributes,
        flatten.unwrap_or(false),
        base_url,
        geom_registry,
        app_registry,
        buffered,
    )
    .map_err(|e| FeatureProcessorError::FileCityGmlReader(format!("{e:?}")))
}

#[allow(clippy::uninlined_format_args)]
fn collect_entities<R: BufRead>(
    st: &mut SubTreeReader<'_, '_, R>,
    base_attributes: &IndexMap<Attribute, AttributeValue>,
    flatten: bool,
    base_url: Url,
    geom_registry: &mut HashMap<Url, Arc<RwLock<GeometryStore>>>,
    app_registry: &mut HashMap<Url, Arc<RwLock<AppearanceStore>>>,
    buffered: &mut Vec<BufferedEntity>,
) -> Result<(), FeatureProcessorError> {
    let mut entities = Vec::new();
    let mut global_appearances = AppearanceStore::default();
    let mut envelope = Envelope::default();

    st.parse_children(|st| {
        let path: &[u8] = &st.current_path();
        match path {
            b"gml:boundedBy" => Ok(()),
            b"gml:boundedBy/gml:Envelope" => {
                envelope.parse(st)?;
                Ok(())
            }
            b"core:cityObjectMember" => {
                let mut cityobj: models::TopLevelCityObject = Default::default();
                cityobj.parse(st)?;
                let geometry_store = st.collect_geometries(envelope.crs_uri.clone());
                let feature_hrefs = st.collect_feature_hrefs();
                let id = cityobj.id();
                let typename = cityobj.name();
                if let Some(root) = cityobj.into_object() {
                    let entity = Entity {
                        id: Some(id.to_string()),
                        typename: Some(typename.to_string()),
                        root,
                        base_url: base_url.clone(),
                        geometry_store: RwLock::new(geometry_store).into(),
                        appearance_store: Default::default(),
                        cross_file_feature_refs: feature_hrefs,
                    };
                    entities.push(entity);
                }
                Ok(())
            }
            b"app:appearanceMember" => {
                let mut appearance_prop: models::appearance::AppearanceProperty =
                    Default::default();
                match appearance_prop.parse(st) {
                    Ok(()) => {
                        let models::appearance::AppearanceProperty::Appearance(appearance) =
                            appearance_prop
                        else {
                            unreachable!();
                        };
                        global_appearances.update(appearance);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Skipping appearance due to parse error (file: {}): {:?}",
                            base_url,
                            e
                        );
                    }
                }
                Ok(())
            }
            other => Err(ParseError::SchemaViolation(format!(
                "Unrecognized element {}",
                String::from_utf8_lossy(other)
            ))),
        }
    })
    .map_err(|e| FeatureProcessorError::FileCityGmlReader(format!("{e:?}")))?;

    for entity in entities {
        // Merge file-level global appearances
        {
            let geom_store = entity.geometry_store.read().unwrap();
            entity.appearance_store.write().unwrap().merge_global(
                &mut global_appearances,
                &geom_store.ring_ids,
                &geom_store.surface_spans,
            );
        }
        // Swap lat/lon coordinate order
        {
            let mut geom_store = entity.geometry_store.write().unwrap();
            geom_store.vertices.iter_mut().for_each(|v| {
                (v[0], v[1], v[2]) = (v[1], v[0], v[2]);
            });
        }
        // Populate registries keyed by polygon URL
        {
            let geom_store = entity.geometry_store.read().unwrap();
            for span in &geom_store.surface_spans {
                let mut poly_url = base_url.clone();
                poly_url.set_fragment(Some(&span.id.0));
                geom_registry.insert(poly_url.clone(), Arc::clone(&entity.geometry_store));
                app_registry.insert(poly_url, Arc::clone(&entity.appearance_store));
            }
        }
        // Check if geometry refs are empty (needs reconstruction from store)
        let root_needs_reconstruction = {
            let nusamai_citygml::Value::Object(obj) = &entity.root else {
                continue;
            };
            matches!(
                &obj.stereotype,
                nusamai_citygml::object::ObjectStereotype::Feature { geometries, .. }
                    if geometries.is_empty()
            )
        };
        buffered.push(BufferedEntity {
            entity,
            base_attributes: base_attributes.clone(),
            flatten,
            base_url: base_url.clone(),
            root_needs_reconstruction,
        });
    }
    Ok(())
}

/// Pass 2: resolve cross-file geometry refs and emit features.
#[allow(clippy::uninlined_format_args)]
pub(super) fn emit_buffered(
    ctx: Context,
    fw: &ProcessorChannelForwarder,
    buffered: Vec<BufferedEntity>,
    geom_registry: &HashMap<Url, Arc<RwLock<GeometryStore>>>,
    app_registry: &HashMap<Url, Arc<RwLock<AppearanceStore>>>,
) -> Result<(), BoxedError> {
    // Build flat feature registry: (file_url, gml_id) -> (Entity, root_needs_reconstruction)
    // Pre-flatten all entities so child IDs (e.g. tran:TrafficArea inside tran:Road) are indexed.
    let flat_feature_registry: HashMap<(Url, String), (Entity, bool)> = buffered
        .iter()
        .flat_map(|be| {
            let flat = if be.flatten {
                FlattenTreeTransform::transform(be.entity.clone())
            } else {
                vec![be.entity.clone()]
            };
            let file_url = be.entity.base_url.clone();
            let rnr = be.root_needs_reconstruction;
            flat.into_iter().filter_map(move |ent| {
                ent.id
                    .clone()
                    .map(|id| ((file_url.clone(), id), (ent, rnr)))
            })
        })
        .collect();

    let mut transformer = GeometricMergedownTransform::new();

    for BufferedEntity {
        entity,
        base_attributes,
        flatten,
        base_url,
        root_needs_reconstruction,
    } in buffered.iter().cloned()
    {
        let gml_id = entity.root.id();
        let name = entity.root.typename();
        let lod = LodMask::find_lods_by_citygml_value(&entity.root);
        let metadata = Metadata {
            feature_id: gml_id.map(|id| id.to_string()),
            feature_type: name.map(|name| name.to_string()),
            lod: Some(lod),
        };
        let mut attributes = HashMap::<Attribute, AttributeValue>::from([
            (
                Attribute::new("featureType"),
                name.map(|s| AttributeValue::String(s.to_string()))
                    .unwrap_or(AttributeValue::Null),
            ),
            (
                Attribute::new("gmlId"),
                gml_id
                    .map(|s| AttributeValue::String(s.to_string()))
                    .unwrap_or(AttributeValue::Null),
            ),
            (
                Attribute::new("gmlRootId"),
                AttributeValue::String(format!("root_{}", to_hash(base_url.as_str()))),
            ),
        ]);
        if let Some(max_lod) = lod.highest_lod() {
            attributes.insert(
                Attribute::new("maxLod"),
                AttributeValue::String(max_lod.to_string()),
            );
            attributes.insert(
                Attribute::new("lod"),
                AttributeValue::String(max_lod.to_string()),
            );
        }
        attributes.extend(base_attributes.clone());

        let cross_file_feature_refs = entity.cross_file_feature_refs.clone();
        let flat_entities = if flatten {
            FlattenTreeTransform::transform(entity)
        } else {
            vec![entity]
        };

        for ent in flat_entities {
            emit_flat_entity(
                &ctx,
                fw,
                ent,
                root_needs_reconstruction,
                attributes.clone(),
                &metadata,
                &mut transformer,
                flatten,
                lod,
                geom_registry,
                app_registry,
            )?;
        }

        // Emit cross-file feature refs using the referencing entity's attributes/metadata
        if flatten && !cross_file_feature_refs.is_empty() {
            for (ref_file_url, ref_id) in &cross_file_feature_refs {
                let key = (ref_file_url.clone(), ref_id.clone());
                if let Some((ref_ent, ref_rnr)) = flat_feature_registry.get(&key) {
                    emit_flat_entity(
                        &ctx,
                        fw,
                        ref_ent.clone(),
                        *ref_rnr,
                        attributes.clone(),
                        &metadata,
                        &mut transformer,
                        flatten,
                        lod,
                        geom_registry,
                        app_registry,
                    )?;
                }
            }
        }
    }
    Ok(())
}
