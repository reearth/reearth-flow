use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Cursor},
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

#[allow(clippy::uninlined_format_args)]
pub(super) fn read_citygml(
    ctx: Context,
    fw: ProcessorChannelForwarder,
    feature: Feature,
    dataset: rhai::AST,
    original_dataset: reearth_flow_types::Expr,
    flatten: Option<bool>,
    lossless_mode: Option<bool>,
    global_params: Option<HashMap<String, serde_json::Value>>,
) -> Result<(), crate::feature::errors::FeatureProcessorError> {
    let code_resolver = Box::new(nusamai_plateau::codelist::Resolver::new());
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
    let context = nusamai_citygml::ParseContext::new(base_url.clone(), code_resolver.as_ref(), lossless_mode.unwrap_or(false));
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
                // Parse top-level CityGML objects (Building, Road, etc.)
                // TopLevelCityObject is an enum of CityGML Feature Types defined in the specification
                // with #[citygml_feature] macro, ensuring only valid Feature Types are parsed
                let mut cityobj: models::TopLevelCityObject = Default::default();
                cityobj.parse(st)?;
                let geometry_store = st.collect_geometries(envelope.crs_uri.clone());
                let id = cityobj.id();
                let typename = cityobj.name();
                let bounded_by = cityobj.bounded_by();
                if let Some(root) = cityobj.into_object() {
                    let entity = Entity {
                        id: Some(id.to_string()),
                        typename: Some(typename.to_string()),
                        root,
                        base_url: base_url.clone(),
                        geometry_store: RwLock::new(geometry_store).into(),
                        appearance_store: Default::default(),
                        bounded_by,
                    };
                    entities.push(entity);
                }
                Ok(())
            }
            b"app:appearanceMember" => {
                let mut app: models::appearance::AppearanceProperty = Default::default();
                app.parse(st)?;
                let models::appearance::AppearanceProperty::Appearance(app) = app else {
                    unreachable!();
                };
                global_appearances.update(app);
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
                // Swap x and y (lat, lng -> lng, lat)
                (v[0], v[1], v[2]) = (v[1], v[0], v[2]);
            });
        }
        let attributes = AttributeValue::from_nusamai_cityml_value(&entity.root);
        let attributes = AttributeValue::convert_array_attributes(&attributes);
        let mut city_gml_attributes = match attributes.len() {
            0 => AttributeValue::Null,
            1 => attributes.values().next().unwrap().clone(),
            _ => AttributeValue::Map(attributes),
        };
        // Debug: Dump cityGmlAttributes as JSON
        if flatten {
            city_gml_attributes = city_gml_attributes.flatten();
        }
        let city_gml_attributes = if let AttributeValue::Map(map) = &city_gml_attributes {
            AttributeValue::Map(AttributeValue::convert_array_attributes(map))
        } else {
            city_gml_attributes
        };
        let gml_id = entity.root.id();
        let name = entity.root.typename();
        let lod = LodMask::find_lods_by_citygml_value(&entity.root);
        let metadata = Metadata {
            feature_id: gml_id.map(|id| id.to_string()),
            feature_type: name.map(|name| name.to_string()),
            lod: Some(lod),
        };
        let mut attributes = HashMap::<Attribute, AttributeValue>::from([
            (Attribute::new("cityGmlAttributes"), city_gml_attributes),
            (
                Attribute::new("gmlName"),
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
            // Also add as "lod" attribute for StatisticsCalculator to use
            attributes.insert(
                Attribute::new("lod"),
                AttributeValue::String(max_lod.to_string()),
            );
        }
        attributes.extend(base_attributes.clone());
        let entities = if flatten {
            FlattenTreeTransform::transform(entity)
        } else {
            vec![entity]
        };
        for mut ent in entities {
            // Calculate child LOD from GeometryRefs in geometry_store that match this child entity
            // Also extract parent feature_id from GeometryRef if child doesn't have gml:id
            let mut child_lod = LodMask::default();
            let mut parent_feature_id: Option<String> = None;
            if let nusamai_citygml::Value::Object(obj) = &ent.root {
                if let nusamai_citygml::object::ObjectStereotype::Feature { geometries, .. } =
                    &obj.stereotype
                {
                    for geom in geometries {
                        child_lod.add_lod(geom.lod);
                        // If child has no gml:id (None or empty string), use parent's feature_id from GeometryRef
                        let has_id = ent.id.as_ref().is_some_and(|id| !id.is_empty());
                        if !has_id && parent_feature_id.is_none() {
                            parent_feature_id = geom.feature_id.clone();
                        }
                    }
                }
            }
            transformer.transform(&mut ent);
            let nusamai_citygml::Value::Object(obj) = &ent.root else {
                continue;
            };
            let nusamai_citygml::object::ObjectStereotype::Feature { .. } = &obj.stereotype else {
                continue;
            };

            // Use entity's own gml:id if it has one (and not empty), otherwise use parent's feature_id from GeometryRef
            let child_id = match &ent.id {
                Some(id) if !id.is_empty() => Some(id.clone()),
                _ => parent_feature_id,
            };
            let child_typename = ent.typename.clone();
            let mut attributes = attributes.clone();
            if flatten {
                if let Some(typename) = &child_typename {
                    attributes.insert(
                        Attribute::new("featureType"),
                        AttributeValue::String(typename.to_string()),
                    );
                    // Override gmlName with child's typename
                    attributes.insert(
                        Attribute::new("gmlName"),
                        AttributeValue::String(typename.to_string()),
                    );
                }
                // Add lod attribute for StatisticsCalculator to use
                // Use child_lod if available, otherwise use parent lod
                let effective_lod = child_lod.highest_lod().or_else(|| lod.highest_lod());
                if let Some(max_lod) = effective_lod {
                    attributes.insert(
                        Attribute::new("lod"),
                        AttributeValue::String(max_lod.to_string()),
                    );
                }
            }

            let geometry: Geometry = ent.try_into().map_err(|e| {
                crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            let mut feature: Feature = geometry.into();
            feature.extend(attributes);
            // When flatten is true, each child entity should have its own LOD and feature_type/feature_id
            // calculated from its geometries instead of inheriting from the parent
            let mut child_metadata = metadata.clone();
            if flatten {
                if child_lod.highest_lod().is_some() {
                    child_metadata.lod = Some(child_lod);
                }
                child_metadata.feature_id = child_id;
                // Use the entity's own typename for feature_type, not from GeometryRef
                // GeometryRef.feature_type points to the parent feature (e.g., Building)
                // but for error reporting we need the child's typename (e.g., GroundSurface)
                child_metadata.feature_type = child_typename;
            }
            feature.metadata = child_metadata;

            fw.send(ExecutorContext::new_with_context_feature_and_port(
                ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
    }
    Ok(())
}
