use std::{
    io::{BufRead, BufReader, Cursor},
    sync::{Arc, RwLock},
};

use nusamai_citygml::{CityGmlElement, CityGmlReader, Envelope, ParseError, SubTreeReader};
use nusamai_plateau::{appearance::AppearanceStore, models, Entity};
use quick_xml::NsReader;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder, executor_operation::ExecutorContext, node::DEFAULT_PORT,
};
use reearth_flow_types::{geometry::Geometry, Attribute, Feature};
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
    let scope = expr_engine.new_scope();
    for (k, v) in &feature.attributes {
        scope.set(k.inner().as_str(), v.clone().into());
    }
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

    st.parse_children(|st| {
        match st.current_path() {
            b"gml:boundedBy" => {
                // skip
                Ok(())
            }
            b"gml:boundedBy/gml:Envelope" => {
                let mut envelope = Envelope::default();
                envelope.parse(st)?;
                Ok(())
            }
            b"core:cityObjectMember" => {
                let mut cityobj: models::TopLevelCityObject = Default::default();
                cityobj.parse(st)?;
                let geometry_store = st.collect_geometries();
                let id = cityobj.id();
                let name = cityobj.name();
                let description = cityobj.description();
                let bounded = cityobj.bounded_by();
                if let Some(root) = cityobj.into_object() {
                    let entity = Entity {
                        id,
                        name,
                        description,
                        root,
                        base_url: base_url.clone(),
                        geometry_store: RwLock::new(geometry_store).into(),
                        appearance_store: Default::default(),
                        bounded,
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
        let geometry: Geometry = entity.try_into().map_err(|e| {
            super::errors::FeatureProcessorError::FileCityGmlReader(format!("{:?}", e))
        })?;
        let mut feature = Feature::new_with_attributes(ctx.feature.attributes.clone());
        feature.geometry = Some(geometry);
        feature
            .attributes
            .insert(Attribute::new("cityGmlAttributes"), attributes.into());
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
    }
    Ok(())
}
