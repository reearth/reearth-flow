use std::collections::HashMap;
use std::str::FromStr;

use nusamai_citygml::GML31_NS;
use reearth_flow_common::{uri::Uri, xml};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use serde_json::Value;

use super::errors::PlateauProcessorError;

#[derive(Debug, Clone, Default)]
pub struct BuildingInstallationGeometryTypeExtractorFactory;

impl ProcessorFactory for BuildingInstallationGeometryTypeExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU3.BuildingInstallationGeometryTypeExtractor"
    }

    fn description(&self) -> &str {
        "Extracts BuildingInstallationGeometryType"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
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
        let process = BuildingInstallationGeometryTypeExtractor {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct BuildingInstallationGeometryTypeExtractor;

impl Processor for BuildingInstallationGeometryTypeExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let city_gml_path = feature
            .attributes
            .get(&Attribute::new("cityGmlPath"))
            .ok_or(
                PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(
                    "cityGmlPath key empty".to_string(),
                ),
            )?;
        let path_uri = Uri::from_str(city_gml_path.to_string().as_str()).map_err(|err| {
            PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                "cityGmlPath is not a valid uri: {}",
                err
            ))
        })?;
        let storage = ctx.storage_resolver.resolve(&path_uri).map_err(|err| {
            PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                "cityGmlPath is not a valid uri: {}",
                err
            ))
        })?;
        let xml_content = storage.get_sync(path_uri.path().as_path()).map_err(|err| {
            PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                "cityGmlPath is not a valid uri: {}",
                err
            ))
        })?;
        let xml_content = String::from_utf8(xml_content.to_vec()).map_err(|err| {
            PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                "cityGmlPath is not a valid uri: {}",
                err
            ))
        })?;
        let document = xml::parse(xml_content.as_str()).map_err(|err| {
            PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                "invalid xml: {}",
                err
            ))
        })?;
        let root_node = xml::get_root_readonly_node(&document).map_err(|err| {
            PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                "invalid xml: {}",
                err
            ))
        })?;
        let xml_ctx = xml::create_context(&document).map_err(|err| {
            PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                "failed to create context: {}",
                err
            ))
        })?;
        let buildings = xml::find_readonly_nodes_by_xpath(&xml_ctx, "*//bldg:Building", &root_node)
            .map_err(|err| {
                PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                    "failed to find buildings: {}",
                    err
                ))
            })?;
        for building in &buildings {
            let building_installations = xml::find_readonly_nodes_by_xpath(
                &xml_ctx,
                ".//bldg:BuildingInstallation",
                building,
            )
            .map_err(|err| {
                PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                    "failed to find building installations: {}",
                    err
                ))
            })?;
            for building_installation in &building_installations {
                let mut tags = Vec::new();
                for n in [2, 3, 4] {
                    let geom = xml::find_readonly_nodes_by_xpath(
                        &xml_ctx,
                        format!("./bldg:lod{}Geometry/*", n).as_str(),
                        building_installation,
                    )
                    .map_err(|err| {
                        PlateauProcessorError::BuildingInstallationGeometryTypeExtractor(format!(
                            "failed to find geometry: {}",
                            err
                        ))
                    })?;
                    geom.iter().for_each(|g| {
                        let geom_type = xml::get_readonly_node_tag(g);
                        tags.push(geom_type);
                    });
                }
                for tag in &tags {
                    let mut feature = feature.clone();
                    feature.refresh_id();
                    let attributes = HashMap::from([
                        (
                            Attribute::new("bldgGmlId"),
                            AttributeValue::String(
                                building
                                    .get_attribute_ns(
                                        "id",
                                        std::str::from_utf8(GML31_NS.into_inner()).unwrap(),
                                    )
                                    .unwrap_or_default(),
                            ),
                        ),
                        (
                            Attribute::new("instGmlId"),
                            AttributeValue::String(
                                building_installation
                                    .get_attribute_ns(
                                        "id",
                                        std::str::from_utf8(GML31_NS.into_inner()).unwrap(),
                                    )
                                    .unwrap_or_default(),
                            ),
                        ),
                        (
                            Attribute::new("geomTag"),
                            AttributeValue::String(tag.clone()),
                        ),
                    ]);
                    feature.attributes.extend(attributes);
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "BuildingInstallationGeometryTypeExtractor"
    }
}
