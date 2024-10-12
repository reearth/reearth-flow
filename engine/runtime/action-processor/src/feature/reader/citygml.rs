use std::{
    io::{BufRead, BufReader, Cursor},
    sync::{Arc, RwLock},
};

use nusamai_citygml::{CityGmlElement, CityGmlReader, Envelope, ParseError, SubTreeReader};
use nusamai_plateau::{appearance::AppearanceStore, models, Entity};
use quick_xml::NsReader;
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder, executor_operation::ExecutorContext, node::DEFAULT_PORT,
};
use reearth_flow_types::{geometry::Geometry, Attribute, AttributeValue, Feature};
use url::Url;

use super::CompiledCommonPropertySchema;

pub(crate) fn read_citygml(
    params: &CompiledCommonPropertySchema,
    ctx: ExecutorContext,
    fw: &mut dyn ProcessorChannelForwarder,
) -> Result<(), super::errors::FeatureProcessorError> {
    let code_resolver = nusamai_plateau::codelist::Resolver::new();
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let feature = &ctx.feature;
    let scope = feature.new_scope(expr_engine.clone());
    let city_gml_path = scope.eval_ast::<String>(&params.expr).map_err(|e| {
        super::errors::FeatureProcessorError::FileCityGmlReader(format!(
            "Failed to evaluate expr: {}",
            e
        ))
    })?;
    let input_path = Uri::for_test(city_gml_path.as_str());
    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(|e| super::errors::FeatureProcessorError::FileCityGmlReader(format!("{:?}", e)))?;
    let byte = storage
        .get_sync(input_path.path().as_path())
        .map_err(|e| super::errors::FeatureProcessorError::FileCityGmlReader(format!("{:?}", e)))?;
    let cursor = Cursor::new(byte);
    let buf_reader = BufReader::new(cursor);

    let base_url: Url = input_path.into();
    let mut xml_reader = NsReader::from_reader(buf_reader);
    let context = nusamai_citygml::ParseContext::new(base_url.clone(), &code_resolver);
    let mut citygml_reader = CityGmlReader::new(context);
    let mut st = citygml_reader
        .start_root(&mut xml_reader)
        .map_err(|e| super::errors::FeatureProcessorError::FileCityGmlReader(format!("{:?}", e)))?;
    parse_tree_reader(&mut st, base_url, ctx, fw)
        .map_err(|e| super::errors::FeatureProcessorError::FileCityGmlReader(format!("{:?}", e)))?;
    Ok(())
}

fn parse_tree_reader<R: BufRead>(
    st: &mut SubTreeReader<'_, '_, R>,
    base_url: Url,
    ctx: ExecutorContext,
    fw: &mut dyn ProcessorChannelForwarder,
) -> Result<(), super::errors::FeatureProcessorError> {
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
                let description = cityobj.description();
                let bounded_by = cityobj.bounded_by();
                if let Some(root) = cityobj.into_object() {
                    if let nusamai_citygml::object::Value::Object(obj) = &root {
                        let entity = Entity {
                            id,
                            description,
                            name: obj.typename.to_string(),
                            root,
                            base_url: base_url.clone(),
                            geometry_store: RwLock::new(geometry_store).into(),
                            appearance_store: Default::default(),
                            bounded_by,
                            geometry_refs: st.geometry_refs().clone(),
                        };
                        entities.push(entity);
                    }
                }
                st.refresh_geomrefs();
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
    .map_err(|e| super::errors::FeatureProcessorError::FileCityGmlReader(format!("{:?}", e)))?;
    for entity in entities {
        {
            let geom_store = entity.geometry_store.read().unwrap();
            entity.appearance_store.write().unwrap().merge_global(
                &mut global_appearances,
                &geom_store.ring_ids,
                &geom_store.surface_spans,
            );
        }
        let attributes = entity.root.to_attribute_json();

        let name = entity.name.clone();
        let gml_id = entity.id.clone();

        let geometry: Geometry = entity.try_into().map_err(|e| {
            super::errors::FeatureProcessorError::FileCityGmlReader(format!("{:?}", e))
        })?;
        let mut feature = Feature::new_with_attributes(ctx.feature.attributes.clone());
        feature
            .attributes
            .insert(Attribute::new("cityGmlAttributes"), attributes.into());
        feature
            .attributes
            .insert(Attribute::new("gmlName"), AttributeValue::String(name));
        feature
            .attributes
            .insert(Attribute::new("gmlId"), AttributeValue::String(gml_id));

        feature.attributes.insert(
            Attribute::new("gmlRootId"),
            AttributeValue::String(format!("root_{}", to_hash(base_url.as_str()))),
        );
        feature.geometry = Some(geometry);
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
    }
    Ok(())
}
