use std::collections::HashMap;

use itertools::Itertools;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use serde_json::Value;

use crate::plateau4::errors::PlateauProcessorError;

#[derive(Debug, Clone, Default)]
pub struct AttributeFlattenerFactory;

impl ProcessorFactory for AttributeFlattenerFactory {
    fn name(&self) -> &str {
        "PLATEAU4.AttributeFlattener"
    }

    fn description(&self) -> &str {
        "Flatten attributes for building feature"
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
        let flattener = super::flattener::Flattener;
        let process = AttributeFlattener { flattener };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct AttributeFlattener {
    flattener: super::flattener::Flattener,
}

impl Processor for AttributeFlattener {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let Some(AttributeValue::Map(city_gml_attribute)) = feature.get(&"cityGmlAttributes")
        else {
            return Err(PlateauProcessorError::AttributeFlattener(format!(
                "No cityGmlAttributes found with feature id = {:?}",
                feature.id
            ))
            .into());
        };
        let mut new_city_gml_attribute = HashMap::new();
        if let Some(flatten_attributes) =
            super::constants::FLATTEN_ATTRIBUTES.get("bldg/bldg:Building")
        {
            for attribute in flatten_attributes {
                let mut json_path: Vec<&str> = vec![];
                json_path.extend(attribute.json_path.split(" "));
                let Some(new_attribute) =
                    super::flattener::get_value_from_json_path(&json_path, city_gml_attribute)
                else {
                    continue;
                };
                new_city_gml_attribute
                    .insert(Attribute::new(attribute.attribute.clone()), new_attribute);
            }
        }
        let edit_city_gml_attribute = city_gml_attribute
            .clone()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<HashMap<String, AttributeValue>>();
        new_city_gml_attribute.extend(
            self.flattener
                .extract_fld_risk_attribute(&edit_city_gml_attribute),
        );

        new_city_gml_attribute.extend(
            self.flattener
                .extract_tnm_htd_ifld_risk_attribute(&edit_city_gml_attribute),
        );
        new_city_gml_attribute.extend(
            self.flattener
                .extract_lsld_risk_attribute(&edit_city_gml_attribute),
        );

        feature.attributes.extend(
            new_city_gml_attribute
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<Attribute, AttributeValue>>(),
        );
        feature.remove(&"cityGmlAttributes");
        let keys = feature.attributes.keys().cloned().collect_vec();
        let attributes = &mut feature.attributes;
        for key in keys.iter() {
            if (key.to_string().starts_with("uro:") || key.to_string().starts_with("bldg:"))
                && key.to_string().ends_with("_type")
            {
                attributes.remove(key);
            }
            if ["gen:genericAttribute", "uro:buildingDisasterRiskAttribute"]
                .contains(&key.to_string().as_str())
            {
                attributes.remove(key);
            }
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeFlattener"
    }
}
