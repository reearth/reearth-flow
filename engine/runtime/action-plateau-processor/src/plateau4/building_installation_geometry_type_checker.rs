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
use thiserror::Error;

use super::errors::PlateauProcessorError;

#[derive(Error, Debug)]
enum Error {
    #[error("cityGmlPath key empty")]
    MissingCityGmlPath,
    #[error("cityGmlPath is not a valid uri: {0}")]
    InvalidUri(#[from] reearth_flow_common::Error),
    #[error("failed to resolve storage: {0}")]
    StorageResolver(#[from] reearth_flow_runtime::errors::ExecutionError),
    #[error("failed to read file: {0}")]
    FileRead(String),
    #[error("failed to parse UTF-8: {0}")]
    Utf8Parse(#[from] std::string::FromUtf8Error),
    #[error("storage error: {0}")]
    Storage(#[from] reearth_flow_storage::Error),
}

impl From<Error> for PlateauProcessorError {
    fn from(err: Error) -> Self {
        PlateauProcessorError::BuildingInstallationGeometryTypeChecker(err.to_string())
    }
}

#[derive(Debug, Clone, Default)]
pub struct BuildingInstallationGeometryTypeCheckerFactory;

impl ProcessorFactory for BuildingInstallationGeometryTypeCheckerFactory {
    fn name(&self) -> &str {
        "PLATEAU4.BuildingInstallationGeometryTypeChecker"
    }

    fn description(&self) -> &str {
        "Checks BuildingInstallation's geometry type"
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
        let process = BuildingInstallationGeometryTypeChecker {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct BuildingInstallationGeometryTypeChecker;

impl Processor for BuildingInstallationGeometryTypeChecker {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.process_impl(ctx, fw).map_err(Into::into)
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "BuildingInstallationGeometryTypeChecker"
    }
}

impl BuildingInstallationGeometryTypeChecker {
    fn process_impl(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), Error> {
        let feature = &ctx.feature;

        let city_gml_path = feature
            .attributes
            .get(&Attribute::new("gmlPath"))
            .ok_or(Error::MissingCityGmlPath)?;
        let path_uri = Uri::from_str(city_gml_path.to_string().as_str())?;
        let storage = ctx.storage_resolver.resolve(&path_uri)?;
        let xml_content = storage
            .get_sync(path_uri.path().as_path())
            .map_err(|e| Error::FileRead(format!("{e:?}")))?;
        let xml_content = String::from_utf8(xml_content.to_vec())?;
        let document = xml::parse(xml_content.as_str())?;
        let root_node = xml::get_root_readonly_node(&document)?;
        let xml_ctx = xml::create_context(&document)?;
        let buildings =
            xml::find_readonly_nodes_by_xpath(&xml_ctx, "*//bldg:Building", &root_node)?;
        for building in &buildings {
            let building_installations = xml::find_readonly_nodes_by_xpath(
                &xml_ctx,
                ".//bldg:BuildingInstallation",
                building,
            )?;
            for building_installation in &building_installations {
                let mut tags = Vec::new();
                for n in [2, 3, 4] {
                    let geom = xml::find_readonly_nodes_by_xpath(
                        &xml_ctx,
                        format!("./bldg:lod{n}Geometry/*").as_str(),
                        building_installation,
                    )?;
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
}
