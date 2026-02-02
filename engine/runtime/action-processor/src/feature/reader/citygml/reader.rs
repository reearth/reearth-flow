use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Cursor, Write},
    str::FromStr,
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;
use nusamai_citygml::{CityGmlElement, CityGmlReader, Envelope, ParseError, SubTreeReader};
use nusamai_plateau::{
    appearance::AppearanceStore, models, Entity, FlattenTreeTransform, GeometricMergedownTransform,
};
use quick_xml::NsReader;
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    executor_operation::{Context, ExecutorContext},
    forwarder::ProcessorChannelForwarder,
    node::DEFAULT_PORT,
};
use reearth_flow_types::{
    geometry::Geometry, lod::LodMask, metadata::Metadata, Attribute, AttributeValue, Feature,
};
use url::Url;

#[allow(clippy::uninlined_format_args, clippy::too_many_arguments)]
pub(super) fn read_citygml(
    ctx: Context,
    fw: ProcessorChannelForwarder,
    feature: Feature,
    dataset: rhai::AST,
    original_dataset: reearth_flow_types::Expr,
    flatten: Option<bool>,
    global_params: Option<HashMap<String, serde_json::Value>>,
    codelists_path: Option<Url>,
) -> Result<(), crate::feature::errors::FeatureProcessorError> {
    let code_resolver = if let Some(codelists_path) = codelists_path {
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
        crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    let storage = storage_resolver.resolve(&input_path).map_err(|e| {
        crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    let byte = storage.get_sync(input_path.path().as_path()).map_err(|e| {
        crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    let cursor = Cursor::new(byte);
    let buf_reader = BufReader::new(cursor);

    let base_url: Url = input_path.into();
    let mut xml_reader = NsReader::from_reader(buf_reader);
    let context = nusamai_citygml::ParseContext::new(base_url.clone(), &code_resolver);
    let mut citygml_reader = CityGmlReader::new(context);
    let mut st = citygml_reader.start_root(&mut xml_reader).map_err(|e| {
        crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    parse_tree_reader(
        &mut st,
        &feature.attributes,
        flatten.unwrap_or(false),
        base_url,
        &ctx,
        &fw,
    )
    .map_err(|e| {
        crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    Ok(())
}

#[allow(clippy::uninlined_format_args)]
fn parse_tree_reader<R: BufRead>(
    st: &mut SubTreeReader<'_, '_, R>,
    base_attributes: &IndexMap<Attribute, AttributeValue>,
    flatten: bool,
    base_url: Url,
    ctx: &Context,
    fw: &ProcessorChannelForwarder,
) -> Result<(), crate::feature::errors::FeatureProcessorError> {
    // Phase 1: Parse CityGML, build features, write them to a temp JSONL file.
    // This lets us drop all parsed entities/appearances/geometry before sending,
    // so backpressure on fw.send() doesn't trap large parsing data in memory.
    let temp_dir = reearth_flow_common::dir::project_temp_dir(
        uuid::Uuid::new_v4().to_string().as_str(),
    )
    .map_err(|e| {
        crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;
    let temp_path = temp_dir.join("features.jsonl");

    let feature_count = {
        let file = std::fs::File::create(&temp_path).map_err(|e| {
            crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
        })?;
        let mut writer = BufWriter::new(file);
        let count = parse_and_write_features(
            st,
            base_attributes,
            flatten,
            base_url,
            &mut writer,
        )?;
        writer.flush().map_err(|e| {
            crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
        })?;
        count
    };
    // All parsing data (entities, appearances, geometry stores) is now dropped.

    // Phase 2: Stream features from disk one at a time.
    if feature_count > 0 {
        let file = std::fs::File::open(&temp_path).map_err(|e| {
            crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
        })?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.map_err(|e| {
                crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            if line.is_empty() {
                continue;
            }
            let feature: Feature = serde_json::from_str(&line).map_err(|e| {
                crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
    }

    // Cleanup temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

/// Parses CityGML entities and appearances, transforms them into Features,
/// and writes each Feature as a JSON line. Returns the number of features written.
#[allow(clippy::uninlined_format_args)]
fn parse_and_write_features<R: BufRead, W: Write>(
    st: &mut SubTreeReader<'_, '_, R>,
    base_attributes: &IndexMap<Attribute, AttributeValue>,
    flatten: bool,
    base_url: Url,
    writer: &mut W,
) -> Result<usize, crate::feature::errors::FeatureProcessorError> {
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
    .map_err(|e| {
        crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
    })?;

    let mut transformer = GeometricMergedownTransform::new();
    let mut count = 0usize;
    for entity in entities {
        {
            let geom_store = entity.geometry_store.read().map_err(|e| {
                crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            entity.appearance_store.write().unwrap().merge_global(
                &mut global_appearances,
                &geom_store.ring_ids,
                &geom_store.surface_spans,
            );
        }
        {
            let mut geom_store = entity.geometry_store.write().unwrap();
            geom_store.vertices.iter_mut().for_each(|v| {
                (v[0], v[1], v[2]) = (v[1], v[0], v[2]);
            });
        }
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
        let flat_entities = if flatten {
            FlattenTreeTransform::transform(entity)
        } else {
            vec![entity]
        };
        for mut ent in flat_entities {
            let nusamai_citygml::Value::Object(obj) = &ent.root else {
                continue;
            };
            let mut child_lod = LodMask::default();
            let mut geom_feature_id: Option<String> = None;
            if let nusamai_citygml::object::ObjectStereotype::Feature { geometries, .. } =
                &obj.stereotype
            {
                for geom in geometries {
                    child_lod.add_lod(geom.lod);
                    let has_id = ent.id.as_ref().is_some_and(|id| !id.is_empty());
                    if !has_id && geom_feature_id.is_none() {
                        geom_feature_id = geom.feature_id.clone();
                    }
                }
            } else {
                continue;
            }
            transformer.transform(&mut ent);

            let child_id = ent.id.clone();
            let child_typename = ent.typename.clone();
            let mut attributes = attributes.clone();
            if flatten {
                if let Some(typename) = &child_typename {
                    if typename != "uro:DmGeometricAttribute" {
                        attributes.insert(
                            Attribute::new("featureType"),
                            AttributeValue::String(typename.to_string()),
                        );
                        attributes.insert(
                            Attribute::new("gmlName"),
                            AttributeValue::String(typename.to_string()),
                        );
                    }
                }
                let effective_lod = child_lod.highest_lod().or_else(|| lod.highest_lod());
                if let Some(max_lod) = effective_lod {
                    attributes.insert(
                        Attribute::new("lod"),
                        AttributeValue::String(max_lod.to_string()),
                    );
                }
            }

            let citygml_attributes = AttributeValue::from_nusamai_citygml_value(&ent.root);
            let citygml_attributes = AttributeValue::Map(citygml_attributes);
            let geometry: Geometry = ent.try_into().map_err(|e| {
                crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            let mut feature: Feature = geometry.into();
            feature.extend(attributes);
            feature.insert("cityGmlAttributes", citygml_attributes);
            let mut child_metadata = metadata.clone();
            if flatten {
                if child_lod.highest_lod().is_some() {
                    child_metadata.lod = Some(child_lod);
                }
                child_metadata.feature_id = child_id;
                child_metadata.feature_type = child_typename;
            }
            feature.metadata = child_metadata;

            serde_json::to_writer(&mut *writer, &feature).map_err(|e| {
                crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            writer.write_all(b"\n").map_err(|e| {
                crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            count += 1;
        }
    }
    Ok(count)
}
