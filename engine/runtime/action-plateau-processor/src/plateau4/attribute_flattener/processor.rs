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
        ("attributes".to_string(), AttributeValue::default_string()),
        (
            "core:creationDate".to_string(),
            AttributeValue::default_string(),
        ),
    ]
});

// outer attribute key used to construct inner attributes to be serialized as JSON
static COMMON_ATTRIBUTES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    vec![
        ("meshcode".to_string(), "meshcode".to_string()),
        ("gml_id".to_string(), "gml:id".to_string()),
        ("featureType".to_string(), "feature_type".to_string()),
    ]
    .into_iter()
    .collect::<HashMap<String, String>>()
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
    encountered_feature_types: HashSet<String>,
    flattener: super::flattener::Flattener,
    common_attribute_processor: super::flattener::CommonAttributeProcessor,
    // store citygml attributes to build the `ancestors` attribute
    gmlid_to_citygml_attributes: HashMap<String, AttributeValue>,
    // buffer to store children features that arrive before their parents
    children_buffer: HashMap<String, Vec<Feature>>,
}

// remove parentId and parentType created by FeatureCitygmlReader's FlattenTreeTransform
fn strip_parent_info(attr: &mut AttributeValue) {
    if let AttributeValue::Map(ref mut map) = attr {
        map.remove("parentId");
        map.remove("parentType");
    }
}

impl AttributeFlattener {
    fn get_parent_id(attr: &AttributeValue) -> Option<String> {
        if let AttributeValue::Map(map) = attr {
            if let Some(AttributeValue::String(parent_id)) = map.get("parentId") {
                return Some(parent_id.clone());
            }
        }
        None
    }

    fn build_ancestors_attribute(&self, attr: &AttributeValue) -> Vec<AttributeValue> {
        let mut ancestors = Vec::new();
        let mut parent_id: Option<String> = Self::get_parent_id(attr);
        while let Some(id) = parent_id {
            let Some(attr) = self.gmlid_to_citygml_attributes.get(&id) else {
                tracing::warn!("Parent ID {id} not found. Children sent before parents?");
                break;
            };
            parent_id = Self::get_parent_id(attr);
            let mut attr = attr.clone();
            strip_parent_info(&mut attr);
            ancestors.push(attr);
        }
        ancestors
    }

    fn process_buffered_children(
        &mut self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        parent_id: &str,
    ) -> Result<(), BoxedError> {
        if let Some(children) = self.children_buffer.remove(parent_id) {
            for child in children {
                let flattened_child = self.flatten_feature(child)?;
                fw.send(
                    ctx.new_with_feature_and_port(flattened_child.clone(), DEFAULT_PORT.clone()),
                );

                // Recursively process this child's buffered children
                if let Some(child_id) = flattened_child.feature_id() {
                    self.process_buffered_children(ctx, fw, &child_id)?;
                }
            }
        }
        Ok(())
    }

    fn generate_schema_feature(&self, feature_type_key: &str) -> Feature {
        let mut feature = Feature::new();
        for (key, value) in BASE_SCHEMA_KEYS.clone().into_iter() {
            feature.attributes.insert(Attribute::new(key), value);
        }

        // Add attributes specific to this feature type that were actually used
        if let Some(flatten_attributes) = super::constants::FLATTEN_ATTRIBUTES.get(feature_type_key)
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
            feature_type: Some(feature_type_key.to_string()),
            lod: None,
        };
        feature
    }

    fn flatten_feature(&mut self, feature: Feature) -> Result<Feature, BoxedError> {
        let mut feature = feature;

        let Some(AttributeValue::Map(city_gml_attribute)) = feature.get(&"cityGmlAttributes")
        else {
            return Err(PlateauProcessorError::AttributeFlattener(format!(
                "No cityGmlAttributes found with feature id = {:?}",
                feature.id
            ))
            .into());
        };

        let mut new_city_gml_attribute = HashMap::new();

        // Build lookup key from package and feature type
        let lookup_key = if let (Some(AttributeValue::String(package)), Some(feature_type)) =
            (feature.get(&"package"), &feature.metadata.feature_type)
        {
            format!("{package}/{feature_type}")
        } else {
            return Err(PlateauProcessorError::AttributeFlattener(
                "Cannot build lookup key for flatten attributes".to_string(),
            )
            .into());
        };

        // Track encountered feature type
        self.encountered_feature_types.insert(lookup_key.clone());

        let mut inner_attributes = city_gml_attribute.clone();
        // add common attributes by copying from feature attributes
        for (key, value) in COMMON_ATTRIBUTES.iter() {
            if let Some(attr_value) = feature.get(&Attribute::new(key.clone())) {
                inner_attributes.insert(value.clone(), attr_value.clone());
            }
        }
        let mut inner_attributes_value = AttributeValue::Map(inner_attributes);
        // attribute must be cached BEFORE inserting ancestors
        if let Some(feature_id) = feature.feature_id() {
            self.gmlid_to_citygml_attributes
                .insert(feature_id, inner_attributes_value.clone());
        }
        let ancestors = self.build_ancestors_attribute(&inner_attributes_value);
        strip_parent_info(&mut inner_attributes_value);
        if let AttributeValue::Map(ref mut map) = inner_attributes_value {
            if !ancestors.is_empty() {
                map.insert("ancestors".to_string(), AttributeValue::Array(ancestors));
            }
            // json path must be extracted AFTER building ancestors attribute
            if let Some(flatten_attributes) = super::constants::FLATTEN_ATTRIBUTES.get(&lookup_key)
            {
                for attribute in flatten_attributes {
                    let mut json_path: Vec<&str> = vec![];
                    json_path.extend(attribute.json_path.split(" "));
                    let Some(new_attribute) =
                        super::flattener::get_value_from_json_path(&json_path, map)
                    else {
                        continue;
                    };
                    self.existing_flatten_attributes
                        .insert(attribute.attribute.clone());
                    new_city_gml_attribute
                        .insert(Attribute::new(attribute.attribute.clone()), new_attribute);
                }
            }
        }
        // save the whole `city_gml_attribute` values as `attributes`
        let inner_attributes_json =
            serde_json::to_string(&serde_json::Value::from(inner_attributes_value)).unwrap();

        let edit_city_gml_attribute = city_gml_attribute
            .clone()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<HashMap<String, AttributeValue>>();

        new_city_gml_attribute.insert(
            Attribute::new("attributes".to_string()),
            AttributeValue::String(inner_attributes_json),
        );

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

        // Set feature_type from metadata
        if let Some(feature_type) = &feature.metadata.feature_type {
            new_city_gml_attribute.insert(
                Attribute::new("feature_type".to_string()),
                AttributeValue::String(feature_type.clone()),
            );
        }

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
        Ok(feature)
    }
}

impl Processor for AttributeFlattener {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = ctx.feature.clone();

        // Get cityGmlAttributes to check for parent
        let Some(AttributeValue::Map(city_gml_attribute)) = feature.get(&"cityGmlAttributes")
        else {
            return Err(PlateauProcessorError::AttributeFlattener(format!(
                "No cityGmlAttributes found with feature id = {:?}",
                feature.id
            ))
            .into());
        };

        // Check if this feature has a parent and if the parent exists in cache
        let parent_id = Self::get_parent_id(&AttributeValue::Map(city_gml_attribute.clone()));
        let parent_exists = parent_id
            .as_ref()
            .map(|id| self.gmlid_to_citygml_attributes.contains_key(id))
            .unwrap_or(true); // No parent means it's a root feature

        if !parent_exists {
            // Buffer this child feature until its parent arrives
            if let Some(parent_id) = parent_id {
                self.children_buffer
                    .entry(parent_id)
                    .or_default()
                    .push(feature);
            }
            return Ok(());
        }

        // Process this feature immediately
        let flattened_feature = self.flatten_feature(feature)?;
        fw.send(ctx.new_with_feature_and_port(flattened_feature.clone(), DEFAULT_PORT.clone()));

        // Check if this feature has any buffered children and process them recursively
        if let Some(feature_id) = flattened_feature.feature_id() {
            self.process_buffered_children(&ctx, fw, &feature_id)?;
        }

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // Warn about any remaining buffered children (orphans without parents)
        if !self.children_buffer.is_empty() {
            tracing::warn!(
                "Found {} orphaned features without parents in buffer",
                self.children_buffer
                    .values()
                    .map(|v| v.len())
                    .sum::<usize>()
            );
            for (parent_id, children) in &self.children_buffer {
                tracing::warn!(
                    "Parent ID {} has {} orphaned children",
                    parent_id,
                    children.len()
                );
            }
        }

        // Generate a schema feature for each encountered feature type
        for feature_type_key in &self.encountered_feature_types {
            let feature = self.generate_schema_feature(feature_type_key);
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                SCHEMA_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeFlattener"
    }
}
