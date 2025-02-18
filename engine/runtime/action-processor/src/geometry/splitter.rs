use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct GeometrySplitterFactory;

impl ProcessorFactory for GeometrySplitterFactory {
    fn name(&self) -> &str {
        "GeometrySplitter"
    }

    fn description(&self) -> &str {
        "Split geometry by type"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process = GeometrySplitter {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct GeometrySplitter;

impl Processor for GeometrySplitter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            return Ok(());
        }
        match &geometry.value {
            GeometryValue::CityGmlGeometry(city_gml_geometry) => {
                if city_gml_geometry.gml_geometries.len() < 2 {
                    let Some(feature_geometry) = city_gml_geometry.gml_geometries.first() else {
                        fw.send(
                            ctx.new_with_feature_and_port(
                                ctx.feature.clone(),
                                DEFAULT_PORT.clone(),
                            ),
                        );
                        return Ok(());
                    };
                    let mut feature = ctx.feature.clone();
                    feature.insert(
                        Attribute::new("geometryName"),
                        AttributeValue::String(feature_geometry.name().to_string()),
                    );
                    feature.insert(
                        Attribute::new("lod"),
                        feature_geometry
                            .lod
                            .map(|lod| AttributeValue::String(lod.to_string()))
                            .unwrap_or(AttributeValue::Null),
                    );
                    let feature_id = feature_geometry.feature_id.clone();
                    let parent_id = if let Some(feature_id) = feature_id {
                        if let Some(AttributeValue::String(gml_id)) = feature.get(&"gmlId") {
                            let gml_id = gml_id.to_string();
                            if gml_id == feature_id {
                                if let Some(AttributeValue::String(gml_root_id)) =
                                    feature.get(&"gmlRootId")
                                {
                                    Some(gml_root_id.to_string())
                                } else {
                                    None
                                }
                            } else {
                                Some(gml_id)
                            }
                        } else if let Some(AttributeValue::String(gml_root_id)) =
                            feature.get(&"gmlRootId")
                        {
                            Some(gml_root_id.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    feature.insert(
                        Attribute::new("featureParentId"),
                        parent_id
                            .map(AttributeValue::String)
                            .unwrap_or(AttributeValue::Null),
                    );
                    feature.insert(
                        Attribute::new("featureId"),
                        feature_geometry
                            .feature_id
                            .as_ref()
                            .map(|feature_id| AttributeValue::String(feature_id.to_string()))
                            .unwrap_or(AttributeValue::Null),
                    );
                    feature.insert(
                        Attribute::new("featureType"),
                        feature_geometry
                            .feature_type
                            .as_ref()
                            .map(|feature_type| AttributeValue::String(feature_type.to_string()))
                            .unwrap_or(AttributeValue::Null),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    return Ok(());
                }
                for split_feature in city_gml_geometry.split_feature() {
                    let mut geometry = geometry.clone();
                    let mut attributes = feature.attributes.clone();
                    let Some(geometry_feature) = split_feature.gml_geometries.first() else {
                        continue;
                    };
                    attributes.insert(
                        Attribute::new("geometryName"),
                        AttributeValue::String(geometry_feature.name().to_string()),
                    );
                    attributes.insert(
                        Attribute::new("lod"),
                        geometry_feature
                            .lod
                            .map(|lod| AttributeValue::String(lod.to_string()))
                            .unwrap_or(AttributeValue::Null),
                    );
                    attributes.insert(
                        Attribute::new("featureId"),
                        geometry_feature
                            .feature_id
                            .as_ref()
                            .map(|feature_id| AttributeValue::String(feature_id.to_string()))
                            .unwrap_or(AttributeValue::Null),
                    );
                    attributes.insert(
                        Attribute::new("featureType"),
                        geometry_feature
                            .feature_type
                            .as_ref()
                            .map(|feature_type| AttributeValue::String(feature_type.to_string()))
                            .unwrap_or(AttributeValue::Null),
                    );

                    let parent_id = if let Some(feature_id) = &geometry_feature.feature_id {
                        if let Some(AttributeValue::String(gml_id)) = feature.get(&"gmlId") {
                            let gml_id = gml_id.to_string();
                            if gml_id == feature_id.clone() {
                                if let Some(AttributeValue::String(gml_root_id)) =
                                    feature.get(&"gmlRootId")
                                {
                                    Some(gml_root_id.to_string())
                                } else {
                                    None
                                }
                            } else {
                                Some(gml_id)
                            }
                        } else if let Some(AttributeValue::String(gml_root_id)) =
                            feature.get(&"gmlRootId")
                        {
                            Some(gml_root_id.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    attributes.insert(
                        Attribute::new("featureParentId"),
                        parent_id
                            .map(AttributeValue::String)
                            .unwrap_or(AttributeValue::Null),
                    );
                    geometry.value = GeometryValue::CityGmlGeometry(split_feature);
                    fw.send(ctx.new_with_feature_and_port(
                        Feature::new_with_attributes_and_geometry(
                            attributes,
                            geometry,
                            feature.metadata.clone(),
                        ),
                        DEFAULT_PORT.clone(),
                    ));
                }
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GeometrySplitter"
    }
}
