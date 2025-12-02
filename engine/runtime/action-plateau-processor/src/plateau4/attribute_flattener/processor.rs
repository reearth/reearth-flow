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
        ("_lod".to_string(), AttributeValue::default_number()),
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

type AttributeMap = HashMap<String, AttributeValue>;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeFlattener {
    existing_flatten_attributes: HashSet<String>,
    encountered_feature_types: HashSet<String>,
    flattener: super::flattener::Flattener,
    common_attribute_processor: super::flattener::CommonAttributeProcessor,
    // storing processed features' citygml attributes for ancestor lookup
    // does not include pending features in children_buffer
    gmlid_to_citygml_attributes: HashMap<String, AttributeMap>,
    // blocking ancestor gml_id -> children features
    children_buffer: HashMap<String, Vec<Feature>>,
}

// remove parentId and parentType created by FeatureCitygmlReader's FlattenTreeTransform
fn strip_parent_info(map: &mut HashMap<String, AttributeValue>) {
    map.remove("parentId");
    map.remove("parentType");
}

// GYear fields that should be converted from string to number
static GYEAR_FIELDS: &[&str] = &[
    "uro:surveyYear",
    "bldg:yearOfConstruction",
    "bldg:yearOfDemolition",
    "uro:yearOpened",
    "uro:yearClosed",
    "uro:enactmentFiscalYear",
    "uro:expirationFiscalYear",
    "uro:fiscalYearOfPublication",
];

/// Convert GYear string fields to numbers recursively in the attribute map
fn convert_gyear_fields(
    mut map: HashMap<String, AttributeValue>,
) -> HashMap<String, AttributeValue> {
    for (key, value) in map.iter_mut() {
        *value = convert_gyear_value(key, std::mem::take(value));
    }
    map
}

fn convert_gyear_value(key: &str, value: AttributeValue) -> AttributeValue {
    match value {
        AttributeValue::String(s) if GYEAR_FIELDS.contains(&key) => {
            if let Ok(n) = s.parse::<i64>() {
                AttributeValue::Number(serde_json::Number::from(n))
            } else {
                AttributeValue::String(s)
            }
        }
        AttributeValue::Array(arr) => AttributeValue::Array(
            arr.into_iter()
                .map(|v| convert_gyear_value(key, v))
                .collect(),
        ),
        AttributeValue::Map(inner_map) => AttributeValue::Map(convert_gyear_fields(inner_map)),
        other => other,
    }
}

impl AttributeFlattener {
    fn process_and_add_risk_attributes(
        &mut self,
        feature: &mut Feature,
        citygml_attributes: &HashMap<String, AttributeValue>,
    ) {
        let mut edit_citygml_attributes = citygml_attributes
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<HashMap<String, AttributeValue>>();

        // Extract bldg:address from core:Address nested structure
        if let Some(address) =
            super::flattener::Flattener::extract_address(&edit_citygml_attributes)
        {
            edit_citygml_attributes.insert("bldg:address".to_string(), address);
        }

        if self.should_flatten_generic_attributes(feature) {
            feature.attributes.extend(
                self.common_attribute_processor
                    .flatten_generic_attributes(&edit_citygml_attributes),
            );
        }

        feature.attributes.extend(
            self.flattener
                .extract_fld_risk_attribute(&edit_citygml_attributes),
        );

        feature.attributes.extend(
            self.flattener
                .extract_tnm_htd_ifld_risk_attribute(&edit_citygml_attributes),
        );

        feature.attributes.extend(
            self.flattener
                .extract_lsld_risk_attribute(&edit_citygml_attributes),
        );
    }

    fn get_parent_attr(&self, citygml_attributes: &AttributeMap) -> AttributeMap {
        if let Some(AttributeValue::String(parent_id)) = citygml_attributes.get("parentId") {
            if let Some(parent_attr) = self.gmlid_to_citygml_attributes.get(parent_id) {
                // use parent attributes as inner attributes for DmGeometricAttribute
                return parent_attr.clone();
            }
        }
        // should be unreachable since parentId lookup and error handling is handled in process()
        tracing::error!("Unreachable code: parent ID not found for DmGeometricAttribute");
        AttributeMap::new()
    }

    fn insert_common_attributes(
        feature: &Feature,
        citygml_attributes: &mut HashMap<String, AttributeValue>,
    ) {
        if let Some(gml_id) = feature.get("gml_id").or_else(|| feature.get("gmlId")) {
            citygml_attributes.insert("gml:id".to_string(), gml_id.clone());
        }

        // meshcode: extract from path attribute (e.g., "55371111_bldg_6697_op.gml" -> 55371111)
        if let Some(AttributeValue::String(path)) = feature.get("path") {
            if let Some(filename) = path.rsplit('/').next() {
                if let Some(meshcode_str) = filename.split('_').next() {
                    citygml_attributes.insert(
                        "meshcode".to_string(),
                        AttributeValue::String(meshcode_str.to_string()),
                    );
                }
            }
        }

        // feature_type
        if let Some(feature_type) = feature.get("featureType") {
            citygml_attributes.insert("feature_type".to_string(), feature_type.clone());
        }
    }

    fn process_inner_attributes(
        &mut self,
        feature: &mut Feature,
        mut citygml_attributes: HashMap<String, AttributeValue>,
        lookup_key: &str,
    ) {
        let mut ancestors = vec![];
        let mut citygml_attributes =
            if feature.feature_type().as_deref() == Some("uro:DmGeometricAttribute") {
                // get parent attributes before stripping parent info
                let mut parent_attr = self.get_parent_attr(&citygml_attributes);
                strip_parent_info(&mut citygml_attributes);
                // extract attributes to toplevel
                for (key, value) in citygml_attributes.iter() {
                    let key = key.replace("uro:", "dm_");
                    feature
                        .attributes
                        .insert(Attribute::new(key.clone()), value.clone());
                }
                let dm_attributes_value = AttributeValue::Map(citygml_attributes);
                let json_string =
                    serde_json::to_string(&serde_json::Value::from(dm_attributes_value)).unwrap();
                feature.attributes.insert(
                    Attribute::new("dm_attributes".to_string()),
                    AttributeValue::String(json_string),
                );
                if let Some(feature_type) = feature.metadata.feature_type.as_ref() {
                    feature.attributes.insert(
                        Attribute::new("dm_feature_type".to_string()),
                        AttributeValue::String(
                            feature_type
                                .strip_prefix("uro:")
                                .unwrap_or(feature_type)
                                .to_string(),
                        ),
                    );
                }
                // DmGeometricAttribute uses parent attributes (the real feature) as inner attributes
                // add common attributes AFTER swapping with parent attributes
                Self::insert_common_attributes(feature, &mut parent_attr);
                parent_attr
            } else {
                // add common attributes BEFORE caching and building ancestors
                Self::insert_common_attributes(feature, &mut citygml_attributes);
                // attribute must be cached BEFORE inserting ancestors, AFTER inserting common attributes
                if let Some(feature_id) = feature.feature_id() {
                    self.gmlid_to_citygml_attributes
                        .insert(feature_id, citygml_attributes.clone());
                }
                ancestors = self.build_ancestors_attribute(&citygml_attributes);
                citygml_attributes
            };
        strip_parent_info(&mut citygml_attributes);

        if !ancestors.is_empty() {
            citygml_attributes.insert("ancestors".to_string(), AttributeValue::Array(ancestors));
        }

        // Extract bldg:address from core:Address nested structure if not present
        if !citygml_attributes.contains_key("bldg:address") {
            if let Some(address) = super::flattener::Flattener::extract_address(&citygml_attributes)
            {
                citygml_attributes.insert("bldg:address".to_string(), address);
            }
        }

        // json path must be extracted AFTER building ancestors attribute
        if let Some(flatten_attributes) = super::constants::FLATTEN_ATTRIBUTES.get(lookup_key) {
            for attribute in flatten_attributes {
                let mut json_path: Vec<&str> = vec![];
                json_path.extend(attribute.json_path.split(" "));
                let Some(new_attribute) =
                    super::flattener::get_value_from_json_path(&json_path, &citygml_attributes)
                else {
                    continue;
                };
                // Convert string to number if data_type is "int" or "int16"
                let new_attribute = match attribute.data_type.as_str() {
                    "int" | "int16" => {
                        if let AttributeValue::String(s) = &new_attribute {
                            if let Ok(n) = s.parse::<i64>() {
                                AttributeValue::Number(serde_json::Number::from(n))
                            } else {
                                new_attribute
                            }
                        } else {
                            new_attribute
                        }
                    }
                    _ => new_attribute,
                };
                self.existing_flatten_attributes
                    .insert(attribute.attribute.clone());
                feature
                    .attributes
                    .insert(Attribute::new(attribute.attribute.clone()), new_attribute);
            }
        }

        // Convert GYear string fields to numbers in citygml_attributes before serialization
        let citygml_attributes = convert_gyear_fields(citygml_attributes);

        // save the whole `citygml_attributes` values as `attributes`
        let citygml_attributes_json = serde_json::to_string(&serde_json::Value::from(
            AttributeValue::Map(citygml_attributes),
        ))
        .unwrap();

        feature.attributes.insert(
            Attribute::new("attributes".to_string()),
            AttributeValue::String(citygml_attributes_json),
        );
    }

    fn get_parent_id(map: &AttributeMap) -> Option<String> {
        if let Some(AttributeValue::String(parent_id)) = map.get("parentId") {
            return Some(parent_id.clone());
        }
        None
    }

    fn build_ancestors_attribute(&self, attr: &AttributeMap) -> Vec<AttributeValue> {
        let mut ancestors = Vec::new();
        let mut parent_id: Option<String> = Self::get_parent_id(attr);
        let mut seen_ids = HashSet::new();
        while let Some(id) = parent_id {
            if seen_ids.contains(&id) {
                tracing::warn!(
                    "Detected cyclic ancestor reference for ID {id}. Stopping ancestor building."
                );
                break;
            }
            seen_ids.insert(id.clone());
            let Some(attr) = self.gmlid_to_citygml_attributes.get(&id) else {
                tracing::warn!("Parent ID {id} not found. Children sent before parents?");
                break;
            };
            parent_id = Self::get_parent_id(attr);
            let mut attr = attr.clone();
            strip_parent_info(&mut attr);
            ancestors.push(AttributeValue::Map(attr));
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
                    "int" => AttributeValue::default_number(),
                    "double" | "measure" => AttributeValue::default_float(),
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
        // Extract the feature_type without package prefix to match actual feature metadata
        // feature_type_key is "{package}/{feature_type}" e.g., "bldg/bldg:Building"
        // but actual features have metadata.feature_type = "bldg:Building"
        let schema_feature_type = feature_type_key
            .split('/')
            .nth(1)
            .unwrap_or(feature_type_key)
            .to_string();
        feature.metadata = Metadata {
            feature_id: None,
            feature_type: Some(schema_feature_type),
            lod: None,
        };
        feature
    }

    fn flatten_feature(&mut self, mut feature: Feature) -> Result<Feature, BoxedError> {
        let Some(AttributeValue::Map(citygml_attributes)) = feature.remove("cityGmlAttributes")
        else {
            return Err(PlateauProcessorError::AttributeFlattener(format!(
                "No cityGmlAttributes found with feature id = {:?}",
                feature.id
            ))
            .into());
        };

        // Build lookup key from package and attribute feature type
        // for example dmGeometricAttribute should find attributes from their parent feature type
        let lookup_key = feature
            .get("featureType")
            .and_then(|v| v.as_string())
            .and_then(|feature_type| {
                if let Some(AttributeValue::String(package)) = feature.get("package") {
                    Some(format!("{package}/{feature_type}"))
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                PlateauProcessorError::AttributeFlattener(
                    "Cannot build lookup key for flatten attributes".to_string(),
                )
            })?;

        // Track encountered feature type
        self.encountered_feature_types.insert(lookup_key.clone());

        // Process risk attributes before consuming citygml_attributes
        self.process_and_add_risk_attributes(&mut feature, &citygml_attributes);

        // Process inner attributes
        self.process_inner_attributes(&mut feature, citygml_attributes, &lookup_key);

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

    /// Determines whether generic attributes should be flattened for the given package and feature type.
    ///
    /// The flattening rules are defined in the following spreadsheet:
    /// https://docs.google.com/spreadsheets/d/1c_sn2GkUR7f5zfXGxQdx9g22UlIGdyWP/edit?gid=1962278825
    ///
    /// However, looking at the FME workflow implementation, it appears that the bldg package
    /// always flattens gen:genericAttribute, so the actual implementation does not necessarily
    /// follow the definition strictly.
    ///
    /// For now, the bldg package always flattens generic attributes.
    /// Support for other packages needs to be added later.
    fn should_flatten_generic_attributes(&self, feature: &Feature) -> bool {
        feature
            .get("package")
            .map(|package| package.as_string().as_deref() == Some("bldg"))
            .unwrap_or(false)
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
        let Some(AttributeValue::Map(citygml_attributes)) = feature.get("cityGmlAttributes") else {
            return Err(PlateauProcessorError::AttributeFlattener(format!(
                "No cityGmlAttributes found with feature id = {:?}",
                feature.id
            ))
            .into());
        };

        // Check if this feature has a parent and if the parent exists in cache
        let parent_id = Self::get_parent_id(citygml_attributes);
        let parent_ready = parent_id
            .as_ref()
            .map(|id| self.gmlid_to_citygml_attributes.contains_key(id))
            .unwrap_or(true); // No parent means it's a root feature

        if !parent_ready {
            // Buffer this child feature until its parent is processed
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
            tracing::error!(
                "Found {} orphaned features without parents in buffer",
                self.children_buffer
                    .values()
                    .map(|v| v.len())
                    .sum::<usize>()
            );
            for (parent_id, children) in &self.children_buffer {
                tracing::error!(
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a citygml_attributes map from JSON
    fn citygml_attrs_from_json(json: &str) -> HashMap<String, AttributeValue> {
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        match AttributeValue::from(value) {
            AttributeValue::Map(map) => map,
            _ => panic!("Expected map"),
        }
    }

    /// Helper to create a Feature with required attributes for flatten_feature
    fn create_test_feature(
        gml_id: &str,
        feature_type: &str,
        package: &str,
        citygml_attributes: HashMap<String, AttributeValue>,
        path: &str,
    ) -> Feature {
        let mut feature = Feature::new();
        feature.id = uuid::Uuid::new_v4();
        feature.insert("gmlId", AttributeValue::String(gml_id.to_string()));
        feature.insert(
            "featureType",
            AttributeValue::String(feature_type.to_string()),
        );
        feature.insert("package", AttributeValue::String(package.to_string()));
        feature.insert("path", AttributeValue::String(path.to_string()));
        feature.insert("cityGmlAttributes", AttributeValue::Map(citygml_attributes));
        feature.metadata.feature_type = Some(feature_type.to_string());
        feature
    }

    /// Test that _lod is number type
    #[test]
    fn test_lod_is_number_in_schema() {
        let citygml_attrs = citygml_attrs_from_json(r#"{}"#);

        let feature = create_test_feature(
            "bldg_test-006",
            "bldg:Building",
            "bldg",
            citygml_attrs,
            "51393186_bldg_6697_op.gml",
        );

        let mut flattener = AttributeFlattener::default();
        let _ = flattener.flatten_feature(feature).unwrap();

        // Generate schema feature
        let schema_feature = flattener.generate_schema_feature("bldg/bldg:Building");
        let lod_schema = schema_feature.get("_lod").unwrap();

        // Schema should indicate number type (default_number returns Number(0))
        match lod_schema {
            AttributeValue::Number(n) => {
                assert!(n.is_i64(), "_lod schema should be integer type");
            }
            _ => panic!("_lod schema should be Number, got {:?}", lod_schema),
        }
    }

    /// Test that bldg:measuredHeight is converted to float (measure type)
    #[test]
    fn test_measured_height_is_float() {
        let citygml_attrs = citygml_attrs_from_json(
            r#"{
            "bldg:measuredHeight": 7.883,
            "bldg:measuredHeight_uom": "m"
        }"#,
        );

        let feature = create_test_feature(
            "bldg_test-001",
            "bldg:Building",
            "bldg",
            citygml_attrs,
            "51393186_bldg_6697_op.gml",
        );

        let mut flattener = AttributeFlattener::default();
        let result = flattener.flatten_feature(feature).unwrap();

        // Check that bldg:measuredHeight is a float
        let measured_height = result.get("bldg:measuredHeight").unwrap();
        match measured_height {
            AttributeValue::Number(n) => {
                assert!(n.is_f64(), "bldg:measuredHeight should be float");
                assert!((n.as_f64().unwrap() - 7.883).abs() < 0.0001);
            }
            _ => panic!(
                "bldg:measuredHeight should be Number, got {:?}",
                measured_height
            ),
        }
    }

    /// Test that uro:surveyYear (GYear field) is converted from string to integer
    #[test]
    fn test_survey_year_converted_to_integer() {
        let citygml_attrs = citygml_attrs_from_json(
            r#"{
                "uro:BuildingDetailAttribute": [{
                    "uro:surveyYear": "2022"
                }]
            }"#,
        );

        let feature = create_test_feature(
            "bldg_test-003",
            "bldg:Building",
            "bldg",
            citygml_attrs,
            "51393186_bldg_6697_op.gml",
        );

        let mut flattener = AttributeFlattener::default();
        let result = flattener.flatten_feature(feature).unwrap();

        // Check flattened attribute
        let survey_year = result.get("uro:BuildingDetailAttribute_uro:surveyYear");
        match survey_year {
            Some(AttributeValue::Number(n)) => {
                assert!(n.is_i64(), "uro:surveyYear should be integer");
                assert_eq!(n.as_i64().unwrap(), 2022);
            }
            _ => panic!(
                "uro:BuildingDetailAttribute_uro:surveyYear should be Number, got {:?}",
                survey_year
            ),
        }

        // Also check it's converted in the attributes JSON
        let attributes_json = result.get("attributes");
        match attributes_json {
            Some(AttributeValue::String(json_str)) => {
                let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
                let survey_year_in_json =
                    &parsed["uro:BuildingDetailAttribute"][0]["uro:surveyYear"];
                assert!(
                    survey_year_in_json.is_i64(),
                    "uro:surveyYear in attributes JSON should be integer, got {:?}",
                    survey_year_in_json
                );
                assert_eq!(survey_year_in_json.as_i64().unwrap(), 2022);
            }
            _ => panic!("attributes should be String, got {:?}", attributes_json),
        }
    }

    /// Test that generic attributes are properly flattened
    #[test]
    fn test_generic_attributes_flattened() {
        let citygml_attrs = citygml_attrs_from_json(
            r#"{
            "gen:genericAttribute": [
                {
                    "type": "string",
                    "name": "延べ面積換算係数",
                    "value": "0.65"
                },
                {
                    "type": "string",
                    "name": "大字・町コード",
                    "value": "2"
                }
            ]
        }"#,
        );

        let feature = create_test_feature(
            "bldg_test-007",
            "bldg:Building",
            "bldg",
            citygml_attrs,
            "51393186_bldg_6697_op.gml",
        );

        let mut flattener = AttributeFlattener::default();
        let result = flattener.flatten_feature(feature).unwrap();

        // Check generic attributes are flattened to top level
        let coefficient = result.get("延べ面積換算係数");
        match coefficient {
            Some(AttributeValue::String(s)) => {
                assert_eq!(s, "0.65");
            }
            _ => panic!("延べ面積換算係数 should be String, got {:?}", coefficient),
        }

        let town_code = result.get("大字・町コード");
        match town_code {
            Some(AttributeValue::String(s)) => {
                assert_eq!(s, "2");
            }
            _ => panic!("大字・町コード should be String, got {:?}", town_code),
        }
    }

    /// Test that bldg:address is extracted from nested core:Address structure
    #[test]
    fn test_address_extracted_from_nested_structure() {
        let citygml_attrs = citygml_attrs_from_json(
            r#"{
            "core:Address": [{
                "xAL:AddressDetails": [{
                    "xAL:Country": [{
                        "xAL:Locality": "東京都新島村式根島"
                    }]
                }]
            }]
        }"#,
        );

        let feature = create_test_feature(
            "bldg_test-008",
            "bldg:Building",
            "bldg",
            citygml_attrs,
            "51393186_bldg_6697_op.gml",
        );

        let mut flattener = AttributeFlattener::default();
        let result = flattener.flatten_feature(feature).unwrap();

        // Check top-level flattened attribute
        let address = result.get("bldg:address");
        match address {
            Some(AttributeValue::String(s)) => {
                assert_eq!(s, "東京都新島村式根島");
            }
            _ => panic!("bldg:address should be String, got {:?}", address),
        }

        // Check attributes JSON also contains bldg:address
        let attributes_json = result.get("attributes");
        match attributes_json {
            Some(AttributeValue::String(json_str)) => {
                let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
                let address_in_json = &parsed["bldg:address"];
                assert!(
                    address_in_json.is_string(),
                    "bldg:address in attributes JSON should be string, got {:?}",
                    address_in_json
                );
                assert_eq!(
                    address_in_json.as_str().unwrap(),
                    "東京都新島村式根島",
                    "bldg:address in attributes should match"
                );
            }
            _ => panic!("attributes should be String, got {:?}", attributes_json),
        }
    }

    /// Test that meshcode is extracted from path
    #[test]
    fn test_meshcode_extracted_from_path() {
        let citygml_attrs = citygml_attrs_from_json(r#"{}"#);

        let feature = create_test_feature(
            "bldg_test-009",
            "bldg:Building",
            "bldg",
            citygml_attrs,
            "/path/to/51393186_bldg_6697_op.gml",
        );

        let mut flattener = AttributeFlattener::default();
        let result = flattener.flatten_feature(feature).unwrap();

        // Check in attributes JSON
        let attributes_json = result.get("attributes");
        match attributes_json {
            Some(AttributeValue::String(json_str)) => {
                let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
                let meshcode = &parsed["meshcode"];
                assert_eq!(meshcode.as_str().unwrap(), "51393186");
            }
            _ => panic!("attributes should be String, got {:?}", attributes_json),
        }
    }

    /// Test comprehensive building attributes matching expected output format
    #[test]
    fn test_comprehensive_building_attributes() {
        let citygml_attrs = citygml_attrs_from_json(
            r#"{
            "core:creationDate": "2025-03-14",
            "bldg:class": "普通建物",
            "bldg:class_code": "3001",
            "bldg:usage": "供給処理施設",
            "bldg:usage_code": "452",
            "bldg:measuredHeight": 7.883,
            "bldg:measuredHeight_uom": "m",
            "bldg:storeysAboveGround": 2,
            "bldg:storeysBelowGround": 0,
            "uro:BuildingIDAttribute": [{
                "uro:buildingID": "13363-bldg-5013",
                "uro:prefecture": "東京都",
                "uro:prefecture_code": "13",
                "uro:city": "東京都新島村",
                "uro:city_code": "13363"
            }],
            "uro:BuildingDetailAttribute": [{
                "uro:buildingRoofEdgeArea": 336.3913,
                "uro:buildingRoofEdgeArea_uom": "m2",
                "uro:fireproofStructureType": "耐火",
                "uro:fireproofStructureType_code": "1001",
                "uro:urbanPlanType": "都市計画区域",
                "uro:urbanPlanType_code": "21",
                "uro:landUseType": "公益施設用地（官公庁施設、文教厚生施設、供給処理施設）",
                "uro:landUseType_code": "214",
                "uro:surveyYear": "2022"
            }],
            "uro:DataQualityAttribute": [{
                "uro:geometrySrcDescLod1": "公共測量成果又は基本測量成果",
                "uro:lod1HeightType": "点群から取得_中央値"
            }],
            "core:Address": [{
                "xAL:AddressDetails": [{
                    "xAL:Country": [{
                        "xAL:Locality": "東京都新島村式根島"
                    }]
                }]
            }],
            "gen:genericAttribute": [
                {"type": "string", "name": "延べ面積換算係数", "value": "0.65"},
                {"type": "string", "name": "大字・町コード", "value": "2"}
            ]
        }"#,
        );

        let feature = create_test_feature(
            "bldg_0a8a9e20-38b5-481c-8153-d08622007198",
            "bldg:Building",
            "bldg",
            citygml_attrs,
            "51393186_bldg_6697_op.gml",
        );

        let mut flattener = AttributeFlattener::default();
        let result = flattener.flatten_feature(feature).unwrap();

        // Verify date types - core:creationDate is parsed as DateTime
        let creation_date = result.get("core:creationDate");
        match creation_date {
            Some(AttributeValue::DateTime(dt)) => {
                let datetime_str = dt.to_string();
                assert!(
                    datetime_str.starts_with("2025-03-14"),
                    "core:creationDate should be 2025-03-14, got {}",
                    datetime_str
                );
            }
            _ => panic!(
                "core:creationDate should be DateTime, got {:?}",
                creation_date
            ),
        }
        assert_eq!(
            result.get("bldg:class"),
            Some(&AttributeValue::String("普通建物".to_string()))
        );
        assert_eq!(
            result.get("bldg:usage"),
            Some(&AttributeValue::String("供給処理施設".to_string()))
        );
        assert_eq!(
            result.get("bldg:address"),
            Some(&AttributeValue::String("東京都新島村式根島".to_string()))
        );

        // Verify integer types
        let storeys_above = result.get("bldg:storeysAboveGround");
        match storeys_above {
            Some(AttributeValue::Number(n)) => {
                assert!(n.is_i64());
                assert_eq!(n.as_i64().unwrap(), 2);
            }
            _ => panic!(
                "bldg:storeysAboveGround should be Number, got {:?}",
                storeys_above
            ),
        }

        let storeys_below = result.get("bldg:storeysBelowGround");
        match storeys_below {
            Some(AttributeValue::Number(n)) => {
                assert!(n.is_i64());
                assert_eq!(n.as_i64().unwrap(), 0);
            }
            _ => panic!(
                "bldg:storeysBelowGround should be Number, got {:?}",
                storeys_below
            ),
        }

        // Verify float types (measure)
        let measured_height = result.get("bldg:measuredHeight");
        match measured_height {
            Some(AttributeValue::Number(n)) => {
                assert!(n.is_f64(), "bldg:measuredHeight should be float");
                assert!((n.as_f64().unwrap() - 7.883).abs() < 0.0001);
            }
            _ => panic!(
                "bldg:measuredHeight should be Number, got {:?}",
                measured_height
            ),
        }

        // Verify nested attributes are flattened
        assert_eq!(
            result.get("uro:BuildingIDAttribute_uro:buildingID"),
            Some(&AttributeValue::String("13363-bldg-5013".to_string()))
        );
        assert_eq!(
            result.get("uro:BuildingIDAttribute_uro:prefecture"),
            Some(&AttributeValue::String("東京都".to_string()))
        );
        assert_eq!(
            result.get("uro:BuildingIDAttribute_uro:city"),
            Some(&AttributeValue::String("東京都新島村".to_string()))
        );

        // Verify surveyYear is integer
        let survey_year = result.get("uro:BuildingDetailAttribute_uro:surveyYear");
        match survey_year {
            Some(AttributeValue::Number(n)) => {
                assert!(n.is_i64(), "surveyYear should be integer");
                assert_eq!(n.as_i64().unwrap(), 2022);
            }
            _ => panic!(
                "uro:BuildingDetailAttribute_uro:surveyYear should be Number, got {:?}",
                survey_year
            ),
        }

        // Verify buildingRoofEdgeArea is float
        let roof_area = result.get("uro:BuildingDetailAttribute_uro:buildingRoofEdgeArea");
        match roof_area {
            Some(AttributeValue::Number(n)) => {
                assert!(n.is_f64(), "buildingRoofEdgeArea should be float");
                assert!((n.as_f64().unwrap() - 336.3913).abs() < 0.0001);
            }
            _ => panic!(
                "uro:BuildingDetailAttribute_uro:buildingRoofEdgeArea should be Number, got {:?}",
                roof_area
            ),
        }

        // Verify DataQualityAttribute fields
        assert_eq!(
            result.get("uro:geometrySrcDescLod1"),
            Some(&AttributeValue::String(
                "公共測量成果又は基本測量成果".to_string()
            ))
        );
        assert_eq!(
            result.get("uro:lod1HeightType"),
            Some(&AttributeValue::String("点群から取得_中央値".to_string()))
        );

        // Verify generic attributes
        assert_eq!(
            result.get("延べ面積換算係数"),
            Some(&AttributeValue::String("0.65".to_string()))
        );
        assert_eq!(
            result.get("大字・町コード"),
            Some(&AttributeValue::String("2".to_string()))
        );
    }
}
