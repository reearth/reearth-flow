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

pub(super) fn read_citygml(
    ctx: Context,
    fw: ProcessorChannelForwarder,
    feature: Feature,
    dataset: rhai::AST,
    original_dataset: reearth_flow_types::Expr,
    flatten: Option<bool>,
    global_params: Option<HashMap<String, serde_json::Value>>,
) -> Result<(), crate::feature::errors::FeatureProcessorError> {
    let code_resolver = nusamai_plateau::codelist::Resolver::new();
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
        let city_gml_attributes = match attributes.len() {
            0 => AttributeValue::Null,
            1 => attributes.values().next().unwrap().clone(),
            _ => AttributeValue::Map(attributes),
        };
        let city_gml_attributes = city_gml_attributes.flatten();
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
        }
        attributes.extend(base_attributes.clone());
        let entities = if flatten {
            FlattenTreeTransform::transform(entity)
        } else {
            vec![entity]
        };
        for mut ent in entities {
            transformer.transform(&mut ent);
            let nusamai_citygml::Value::Object(obj) = &ent.root else {
                continue;
            };
            let nusamai_citygml::object::ObjectStereotype::Feature { .. } = &obj.stereotype else {
                continue;
            };
            let geometry: Geometry = ent.try_into().map_err(|e| {
                crate::feature::errors::FeatureProcessorError::FileCityGmlReader(format!("{e:?}"))
            })?;
            let mut feature: Feature = geometry.into();
            feature.extend(attributes.clone());
            feature.metadata = metadata.clone();
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
    }
    Ok(())
}
