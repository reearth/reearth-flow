use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{metadata::Metadata, Attribute, AttributeValue, Feature};
use serde_json::Value;

use crate::plateau4::errors::PlateauProcessorError;

static SCHEMA_PORT: Lazy<Port> = Lazy::new(|| Port::new("schema"));
static BASE_SCHEMA_KEYS: Lazy<Vec<(String, AttributeValue)>> = Lazy::new(|| {
    vec![
        ("_lod".to_string(), AttributeValue::default_string()),
        ("_lod_type".to_string(), AttributeValue::default_string()),
        ("_x".to_string(), AttributeValue::default_float()),
        ("_y".to_string(), AttributeValue::default_float()),
        ("_xmin".to_string(), AttributeValue::default_float()),
        ("_xmax".to_string(), AttributeValue::default_float()),
        ("_ymin".to_string(), AttributeValue::default_float()),
        ("_ymax".to_string(), AttributeValue::default_float()),
        ("_zmin".to_string(), AttributeValue::default_float()),
        ("_zmax".to_string(), AttributeValue::default_float()),
        ("meshcode".to_string(), AttributeValue::default_string()),
        ("feature_type".to_string(), AttributeValue::default_string()),
        ("city_code".to_string(), AttributeValue::default_string()),
        ("city_name".to_string(), AttributeValue::default_string()),
        ("gml_id".to_string(), AttributeValue::default_string()),
    ]
});

#[derive(Debug, Clone, Default)]
pub(crate) struct AttributeFlattenerFactory;

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
        vec![DEFAULT_PORT.clone(), SCHEMA_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process = AttributeFlattener::default();
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeFlattener {
    existing_flatten_attributes: HashSet<String>,
    flattener: super::flattener::Flattener,
    common_attribute_processor: super::flattener::CommonAttributeProcessor,
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
                self.existing_flatten_attributes
                    .insert(attribute.attribute.clone());
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
            self.common_attribute_processor
                .flatten_generic_attributes(&edit_city_gml_attribute),
        );

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
                attributes.swap_remove(key);
            }
            if ["gen:genericAttribute", "uro:buildingDisasterRiskAttribute"]
                .contains(&key.to_string().as_str())
            {
                attributes.swap_remove(key);
            }
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let mut feature = Feature::new();
        for (key, value) in BASE_SCHEMA_KEYS.clone().into_iter() {
            feature.attributes.insert(Attribute::new(key), value);
        }
        if let Some(flatten_attributes) =
            super::constants::FLATTEN_ATTRIBUTES.get("bldg/bldg:Building")
        {
            for attribute in flatten_attributes {
                if !self
                    .existing_flatten_attributes
                    .contains(&attribute.attribute)
                {
                    continue;
                }
                let data_type = match attribute.data_type.as_str() {
                    "string" | "date" => AttributeValue::default_string(),
                    "int" | "double" | "measure" => AttributeValue::default_number(),
                    _ => continue,
                };
                feature
                    .attributes
                    .insert(Attribute::new(attribute.attribute.clone()), data_type);
            }
        }
        let generic_schema = self.common_attribute_processor.get_generic_schema();
        feature.extend(generic_schema);

        for typ in ["fld", "tnm", "htd", "ifld", "rfld", "lsld"] {
            if let Some(definition) = self.flattener.risk_to_attribute_definitions.get(typ) {
                feature.extend(
                    definition
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (Attribute::new(k), v))
                        .collect::<HashMap<Attribute, AttributeValue>>(),
                );
            }
        }
        feature.metadata = Metadata {
            feature_id: None,
            feature_type: Some("bldg:Building".to_string()),
            lod: None,
        };
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            &ctx,
            feature,
            SCHEMA_PORT.clone(),
        ));
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeFlattener"
    }
}
