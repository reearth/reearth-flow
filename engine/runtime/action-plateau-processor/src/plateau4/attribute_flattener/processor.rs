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
use reearth_flow_types::{metadata::Metadata, Attribute, AttributeValue, Attributes, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::plateau4::errors::PlateauProcessorError;

/// Attribute keys to skip during attribute filtering.
/// Based on FME reference implementation's SKIPPABLE_TAGS (usually the inner tag of the FME skippable tag)
static SKIPPABLE_KEYS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "brid:BridgeInstallation",
        // TODO: add other tags
    ])
});

/// DM Geometric Attribute keys to extract to toplevel.
/// Based on FME reference implementation's DmGeometricAttributeExtractor ATTRS list.
static DM_GEOMETRIC_ATTRS: &[&str] = &[
    "uro:dmCode",
    "uro:dmCode_code",
    "uro:geometryType",
    "uro:geometryType_code",
    "uro:mapLevel",
    "uro:mapLevel_code",
    "uro:shapeType",
    "uro:shapeType_code",
];

static SCHEMA_PORT: Lazy<Port> = Lazy::new(|| Port::new("schema"));
static BASE_SCHEMA_KEYS: Lazy<Vec<(String, AttributeValue)>> = Lazy::new(|| {
    vec![
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
static BLDG_SCHEMA_KEYS: Lazy<Vec<(String, AttributeValue)>> = Lazy::new(|| {
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
    ]
});

/// # AttributeFlattener Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttributeFlattenerParam {
    /// When true, only include attributes that were actually used during processing in the schema output.
    /// When false (default), include all defined attributes in the schema regardless of usage.
    #[serde(default)]
    existing_flatten_attributes: bool,
}

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
        Some(schemars::schema_for!(AttributeFlattenerParam))
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributeFlattenerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::AttributeFlattenerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::AttributeFlattenerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            AttributeFlattenerParam::default()
        };
        let process = AttributeFlattener {
            filter_existing_flatten_attributes: params.existing_flatten_attributes,
            ..Default::default()
        };
        Ok(Box::new(process))
    }
}

type AttributeMap = HashMap<String, AttributeValue>;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeFlattener {
    filter_existing_flatten_attributes: bool,
    existing_flatten_attributes: HashMap<String, HashSet<String>>, // feature_type_key -> attribute names
    encountered_feature_types: HashSet<String>,
    flattener: super::flattener::Flattener,
    common_attribute_processor: super::flattener::CommonAttributeProcessor,
    // storing processed features' citygml attributes for ancestor lookup
    // does not include pending features in children_buffer
    gmlid_to_citygml_attributes: HashMap<String, AttributeMap>,
    // storing processed features' citygml attributes for subfeature auto-inheriting toplevel attributes
    // corresponding to two-round processing in FME:
    // 1. PythonCaller produces toplevel attributes
    // 2. The toplevel attributes are copied to all its subfeatures
    // 3. These subfeatures are then processed by PythonCaller2 without toplevel as ancestor
    gmlid_to_subfeature_inherited: HashMap<String, AttributeMap>,
    // blocking ancestor gml_id -> children features
    children_buffer: HashMap<String, Vec<Feature>>,
    // LOD4 feature_type_key -> ancestor feature_type_key mapping for schema generation
    lod4_to_ancestor_type: HashMap<String, String>,
    // risk attribute keys per feature, for excluding from LOD4 inheritance
    gmlid_to_risk_attr_keys: HashMap<String, HashSet<String>>,
}

// remove parentId and parentType created by FeatureCitygmlReader's FlattenTreeTransform
fn strip_parent_info(map: &mut HashMap<String, AttributeValue>) {
    map.remove("parentId");
    map.remove("parentType");
}

// GYear fields that should be converted from string to number
static GYEAR_FIELDS: &[&str] = &[
    "uro:surveyYear",
    "uro:year",
    "bldg:yearOfConstruction",
    "bldg:yearOfDemolition",
    "uro:yearOpened",
    "uro:yearClosed",
    "uro:enactmentFiscalYear",
    "uro:expirationFiscalYear",
    "uro:fiscalYearOfPublication",
    "uro:assessmentFiscalYear",
    "uro:installationYear",
    "urf:enactmentFiscalYear",
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

/// Recursively filters skippable keys from citygml attributes.
/// Skippable keys are PLATEAU-specific sub-features and geometry boundaries
/// that shouldn't be included as simple attributes.
fn filter_skippable_keys(
    mut map: HashMap<String, AttributeValue>,
) -> HashMap<String, AttributeValue> {
    map.retain(|key, value| {
        // Skip known skippable keys
        if SKIPPABLE_KEYS.contains(key.as_str()) {
            return false;
        }

        // Recursively filter nested structures
        match value {
            AttributeValue::Map(nested_map) => {
                let filtered = filter_skippable_keys(std::mem::take(nested_map));
                *value = AttributeValue::Map(filtered);
                true
            }
            AttributeValue::Array(arr) => {
                let filtered_arr: Vec<AttributeValue> = std::mem::take(arr)
                    .into_iter()
                    .map(|item| match item {
                        AttributeValue::Map(nested_map) => {
                            AttributeValue::Map(filter_skippable_keys(nested_map))
                        }
                        other => other,
                    })
                    .collect();
                *value = AttributeValue::Array(filtered_arr);
                true
            }
            _ => true,
        }
    });
    map
}

impl AttributeFlattener {
    fn process_and_add_risk_attributes(
        &mut self,
        feature: &mut Feature,
        citygml_attributes: &HashMap<String, AttributeValue>,
    ) -> HashSet<String> {
        let edit_citygml_attributes = citygml_attributes
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<HashMap<String, AttributeValue>>();

        if self.should_flatten_generic_attributes(feature) {
            feature.extend(
                self.common_attribute_processor
                    .flatten_generic_attributes(&edit_citygml_attributes),
            );
        }

        let mut risk_keys = HashSet::new();

        let fld = self
            .flattener
            .extract_fld_risk_attribute(&edit_citygml_attributes);
        risk_keys.extend(fld.keys().map(|k| k.to_string()));
        feature.extend(fld);

        let tnm = self
            .flattener
            .extract_tnm_htd_ifld_risk_attribute(&edit_citygml_attributes);
        risk_keys.extend(tnm.keys().map(|k| k.to_string()));
        feature.extend(tnm);

        let lsld = self
            .flattener
            .extract_lsld_risk_attribute(&edit_citygml_attributes);
        risk_keys.extend(lsld.keys().map(|k| k.to_string()));
        feature.extend(lsld);

        risk_keys
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

    // LOD4 subfeatures inherit top-level ancestor's extracted attributes
    // They also do not have toplevel ancestor in their ancestors list (popped here)
    fn inherit_lod4_attributes(
        &mut self,
        feature: &mut Feature,
        ancestors: &mut Vec<AttributeValue>,
        lookup_key: &str,
    ) {
        let is_lod4 = matches!(feature.get("lod"), Some(AttributeValue::String(lod)) if lod == "4");
        if !is_lod4 {
            return;
        }

        let Some(AttributeValue::Map(toplevel_ancestor)) = ancestors.pop() else {
            return;
        };
        let Some(AttributeValue::String(toplevel_gml_id)) = toplevel_ancestor.get("gml:id") else {
            return;
        };
        let Some(toplevel_attrs) = self.gmlid_to_subfeature_inherited.get(toplevel_gml_id) else {
            return;
        };

        // Track LOD4 -> ancestor type mapping for schema generation
        if let Some(AttributeValue::String(ancestor_feature_type)) =
            toplevel_ancestor.get("feature_type")
        {
            if let Some(package) = feature.get("package").and_then(|v| v.as_string()) {
                let ancestor_lookup_key = format!("{}/{}", package, ancestor_feature_type);
                self.lod4_to_ancestor_type
                    .insert(lookup_key.to_string(), ancestor_lookup_key);
            }
        }

        // Inherit attributes that the subfeature doesn't have
        for (key, value) in toplevel_attrs.iter() {
            let attr_key = Attribute::new(key.clone());
            if !feature.attributes.contains_key(&attr_key) {
                feature.insert(key.clone(), value.clone());
            }
        }
    }

    /// Walks the parent chain from a cached feature to find the root (toplevel) ancestor's gml:id.
    fn find_toplevel_ancestor_id(&self, feature_id: &str) -> Option<String> {
        let attr = self.gmlid_to_citygml_attributes.get(feature_id)?;
        let mut toplevel_id = None;
        let mut current_id = Self::get_parent_id(attr);
        let mut seen_ids = HashSet::new();
        while let Some(id) = current_id {
            if seen_ids.contains(&id) {
                break;
            }
            seen_ids.insert(id.clone());
            let Some(attr) = self.gmlid_to_citygml_attributes.get(&id) else {
                break;
            };
            toplevel_id = Some(id);
            current_id = Self::get_parent_id(attr);
        }
        toplevel_id
    }

    /// LOD4 v2: overrides the feature's inner attributes with the toplevel ancestor's
    /// citygml_attributes, and inherits toplevel's extracted feature-level attributes.
    /// Called after normal processing to override results for LOD4 features.
    fn inherit_lod4_attributes_v2(&mut self, feature: &mut Feature, lookup_key: &str) {
        let is_lod4 = matches!(feature.get("lod"), Some(AttributeValue::String(lod)) if lod == "4");
        if !is_lod4 {
            return;
        }
        let is_tun = feature
            .get("package")
            .and_then(|v| v.as_string())
            .is_some_and(|p| p == "tun");
        if !is_tun {
            return;
        }

        let Some(feature_id) = feature.feature_id() else {
            return;
        };
        let Some(toplevel_id) = self.find_toplevel_ancestor_id(&feature_id) else {
            return;
        };

        // Override inner attributes with toplevel's citygml_attributes,
        // but keep the surface's own feature_type and gml:id
        let own_citygml = self.gmlid_to_citygml_attributes.get(&feature_id);
        let own_feature_type = own_citygml.and_then(|a| a.get("feature_type").cloned());
        let own_gml_id = own_citygml.and_then(|a| a.get("gml:id").cloned());

        if let Some(toplevel_citygml) = self.gmlid_to_citygml_attributes.get(&toplevel_id) {
            let mut attrs = toplevel_citygml.clone();
            strip_parent_info(&mut attrs);
            if let Some(v) = own_feature_type {
                attrs.insert("feature_type".to_string(), v);
            }
            if let Some(v) = own_gml_id {
                attrs.insert("gml:id".to_string(), v);
            }
            let attrs = convert_gyear_fields(attrs);
            let json = serde_json::to_string(&serde_json::Value::from(AttributeValue::Map(attrs)))
                .unwrap();
            feature.insert("attributes".to_string(), AttributeValue::String(json));
        }

        // Track LOD4 -> ancestor type mapping for schema generation
        if let Some(toplevel_citygml) = self.gmlid_to_citygml_attributes.get(&toplevel_id) {
            if let Some(AttributeValue::String(ancestor_feature_type)) =
                toplevel_citygml.get("feature_type")
            {
                if let Some(package) = feature.get("package").and_then(|v| v.as_string()) {
                    let ancestor_lookup_key = format!("{}/{}", package, ancestor_feature_type);
                    self.lod4_to_ancestor_type
                        .insert(lookup_key.to_string(), ancestor_lookup_key);
                }
            }
        }

        // Inherit toplevel's extracted feature-level attributes, excluding risk attributes
        let toplevel_attrs = self
            .gmlid_to_subfeature_inherited
            .get(&toplevel_id)
            .cloned();
        let toplevel_risk_keys = self.gmlid_to_risk_attr_keys.get(&toplevel_id);
        if let Some(toplevel_attrs) = toplevel_attrs {
            for (key, value) in toplevel_attrs.iter() {
                if toplevel_risk_keys.is_some_and(|keys| keys.contains(key)) {
                    continue;
                }
                let attr_key = Attribute::new(key.clone());
                if !feature.attributes.contains_key(&attr_key) {
                    feature.insert(key.clone(), value.clone());
                }
            }
        }
    }

    fn insert_common_attributes(
        feature: &Feature,
        citygml_attributes: &mut HashMap<String, AttributeValue>,
    ) {
        // Check if this is a risk feature type that should exclude gml_id and meshcode
        let is_risk_package = feature
            .get("package")
            .and_then(|p| p.as_string())
            .map(|pkg| ["fld", "tnm", "htd", "ifld", "rfld"].contains(&pkg.as_str()))
            .unwrap_or(false);

        if !is_risk_package {
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
        let feature_type = feature.feature_type().unwrap_or_default();
        let mut citygml_attributes = if feature_type == "uro:DmGeometricAttribute" {
            // get parent attributes before stripping parent info
            let mut parent_attr = self.get_parent_attr(&citygml_attributes);
            strip_parent_info(&mut citygml_attributes);
            // extract attributes to toplevel
            for (key, value) in citygml_attributes.iter() {
                if !DM_GEOMETRIC_ATTRS.contains(&key.as_str()) {
                    continue;
                }
                let key = key.replace("uro:", "dm_");
                feature.insert(key.clone(), value.clone());
            }
            let dm_attributes_value = AttributeValue::Map(citygml_attributes);
            let json_string =
                serde_json::to_string(&serde_json::Value::from(dm_attributes_value)).unwrap();
            feature.insert(
                "dm_attributes".to_string(),
                AttributeValue::String(json_string),
            );
            if let Some(feature_type) = feature.metadata.feature_type.as_ref() {
                feature.insert(
                    "dm_feature_type".to_string(),
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
            if feature_type.starts_with("urf:") {
                // skip ancestor building for urf features
            } else {
                ancestors = self.build_ancestors_attribute(&citygml_attributes);
            }
            citygml_attributes
        };
        strip_parent_info(&mut citygml_attributes);

        // For LOD4 subfeatures, inherit top-level ancestor's extracted attributes
        // PLATEAU v5 uses a different lod4 inheritance implementation (implemented in inherit_lod4_attributes_v2)
        // The proper fix is to implement PLATEAU5.attribute_flattener. For now, we specially match tun which has only v5 data
        let package = feature
            .get("package")
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        if package != "tun" {
            self.inherit_lod4_attributes(feature, &mut ancestors, lookup_key);
        }

        if !ancestors.is_empty() {
            citygml_attributes.insert(
                "ancestors".to_string(),
                AttributeValue::Array(ancestors.iter().rev().cloned().collect()),
            );
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

                // Attribute value 9999/-9999 will be filtered to match FME implementation
                let is_unknown_value = match &new_attribute {
                    AttributeValue::Number(n) => {
                        n.as_i64() == Some(9999)
                            || n.as_i64() == Some(-9999)
                            || n.as_f64()
                                .map(|f| f == 9999.0 || f == -9999.0)
                                .unwrap_or(false)
                    }
                    _ => false,
                };

                if is_unknown_value {
                    continue;
                }

                feature.insert(attribute.attribute.clone(), new_attribute);
            }
        }

        // Convert GYear string fields to numbers in citygml_attributes before serialization
        let citygml_attributes = convert_gyear_fields(citygml_attributes);

        // Cache attributes only for top-level + LOD4 features (for LOD4 subfeature inheritance)
        let is_toplevel = !citygml_attributes.contains_key("parentId");
        let is_lod4 = matches!(feature.get("lod"), Some(AttributeValue::String(lod)) if lod == "4");
        if let Some(feature_id) = feature.feature_id() {
            if is_toplevel || is_lod4 {
                let flattened_attrs: AttributeMap = feature
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect();
                self.gmlid_to_subfeature_inherited
                    .insert(feature_id, flattened_attrs);
            }
        }

        // save the whole `citygml_attributes` values as `attributes`
        let citygml_attributes_json = serde_json::to_string(&serde_json::Value::from(
            AttributeValue::Map(citygml_attributes),
        ))
        .unwrap();

        feature.insert(
            "attributes".to_string(),
            AttributeValue::String(citygml_attributes_json),
        );

        // For LOD4 subfeatures, override with toplevel ancestor's attributes
        self.inherit_lod4_attributes_v2(feature, lookup_key);
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
        let mut attributes = Attributes::new();
        for (key, value) in BASE_SCHEMA_KEYS.clone().into_iter() {
            attributes.insert(Attribute::new(key), value);
        }
        if feature_type_key.starts_with("bldg/") {
            for (key, value) in BLDG_SCHEMA_KEYS.clone().into_iter() {
                attributes.insert(Attribute::new(key), value);
            }
        }
        let mut feature = Feature::new_with_attributes(attributes);

        // For LOD4 features, use ancestor's feature_type_key for schema lookup
        let schema_lookup_key = self
            .lod4_to_ancestor_type
            .get(feature_type_key)
            .map(|s| s.as_str())
            .unwrap_or(feature_type_key);

        // Add attributes specific to this feature type that were actually used
        if let Some(flatten_attributes) =
            super::constants::FLATTEN_ATTRIBUTES.get(schema_lookup_key)
        {
            let used_attributes = self.existing_flatten_attributes.get(feature_type_key);
            for attribute in flatten_attributes {
                if self.filter_existing_flatten_attributes
                    && !used_attributes
                        .map(|attrs| attrs.contains(&attribute.attribute))
                        .unwrap_or(false)
                {
                    continue;
                }
                let data_type = match attribute.data_type.as_str() {
                    "string" | "date" | "buffer" => AttributeValue::default_string(),
                    "int" | "int16" => AttributeValue::default_number(),
                    "double" | "real64" | "measure" => AttributeValue::default_float(),
                    _ => continue,
                };
                feature.insert(attribute.attribute.clone(), data_type);
            }
        }
        let generic_schema = self.common_attribute_processor.get_generic_schema();
        feature.extend(generic_schema);

        // fld attributes use sorted order (by desc_code, admin_code, scale_code, order)
        if let Some(fld_definitions) = self.flattener.risk_to_attribute_definitions.get("fld") {
            let mut entries = self.flattener.fld_sort_entries.clone();
            super::flattener::sort_fld_entries(&mut entries);
            for entry in &entries {
                if let Some(value_type) = fld_definitions.get(&entry.attr_name) {
                    feature.insert(entry.attr_name.clone(), value_type.clone());
                }
            }
        }
        for typ in ["tnm", "htd", "ifld", "rfld", "lsld"] {
            if let Some(definition) = self.flattener.risk_to_attribute_definitions.get(typ) {
                feature.extend(
                    definition
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (Attribute::new(k), v)),
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

        // Filter out PLATEAU-specific skippable keys (sub-features, geometry boundaries)
        let mut citygml_attributes = filter_skippable_keys(citygml_attributes);

        // Extract bldg:address from core:Address nested structure and remove core:Address
        // This mirrors FME's behavior where bldg:address is extracted but core:Address is not included
        if !citygml_attributes.contains_key("bldg:address") {
            if let Some(address) = super::flattener::Flattener::extract_address(&citygml_attributes)
            {
                citygml_attributes.insert("bldg:address".to_string(), address);
            }
        }
        citygml_attributes.remove("core:Address");

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
        let risk_keys = self.process_and_add_risk_attributes(&mut feature, &citygml_attributes);
        if let Some(feature_id) = feature.feature_id() {
            self.gmlid_to_risk_attr_keys.insert(feature_id, risk_keys);
        }

        // Process inner attributes
        self.process_inner_attributes(&mut feature, citygml_attributes, &lookup_key);

        let keys = feature.attributes.keys().cloned().collect_vec();
        let attributes = feature.attributes_mut();
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

        // Collect all attribute keys that ended up on the feature for schema generation
        // This is done in one place after all processing (flattening, inheritance, etc.) is complete
        let attribute_keys: HashSet<String> =
            feature.attributes.keys().map(|k| k.to_string()).collect();
        self.existing_flatten_attributes
            .entry(lookup_key)
            .or_default()
            .extend(attribute_keys);

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
    /// For now, the bldg and gen packages always flattens generic attributes.
    /// Support for other packages needs to be added later.
    fn should_flatten_generic_attributes(&self, feature: &Feature) -> bool {
        let package = feature
            .get("package")
            .and_then(|v| v.as_string())
            .unwrap_or("".to_string());
        if package == "bldg" || package == "gen" {
            return true;
        }
        false
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

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
        let mut feature = Feature::new_with_attributes(Attributes::new());
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

        // Verify date types - core:creationDate is parsed as String
        let creation_date = result.get("core:creationDate");
        match creation_date {
            Some(AttributeValue::String(dt)) => {
                assert_eq!(
                    dt, "2025-03-14",
                    "core:creationDate should be 2025-03-14, got {dt}",
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

    /// Test that sentinel values 9999 and -9999 are filtered out
    /// These values represent unknown/undefined data in PLATEAU and should not be set as attributes
    #[test]
    fn test_sentinel_values_filtered() {
        let citygml_attrs = citygml_attrs_from_json(
            r#"{
            "uro:BuildingDetailAttribute": [{
                "uro:specifiedBuildingCoverageRate": -9999,
                "uro:specifiedFloorAreaRate": -9999,
                "uro:buildingStructureType": "鉄筋コンクリート造",
                "uro:buildingStructureType_code": "610",
                "uro:surveyYear": "2022"
            }]
        }"#,
        );

        let feature = create_test_feature(
            "bldg_test-010",
            "bldg:Building",
            "bldg",
            citygml_attrs,
            "53404336_bldg_6697.gml",
        );

        let mut flattener = AttributeFlattener::default();
        let result = flattener.flatten_feature(feature).unwrap();

        // Verify that -9999 sentinel values are NOT present in the output
        assert_eq!(
            result.get("uro:BuildingDetailAttribute_uro:specifiedBuildingCoverageRate"),
            None,
            "Sentinel value -9999 for specifiedBuildingCoverageRate should be filtered out"
        );
        assert_eq!(
            result.get("uro:BuildingDetailAttribute_uro:specifiedFloorAreaRate"),
            None,
            "Sentinel value -9999 for specifiedFloorAreaRate should be filtered out"
        );

        // Verify that valid values are still present
        assert_eq!(
            result.get("uro:BuildingDetailAttribute_uro:buildingStructureType"),
            Some(&AttributeValue::String("鉄筋コンクリート造".to_string())),
            "Valid string attributes should still be present"
        );

        let survey_year = result.get("uro:BuildingDetailAttribute_uro:surveyYear");
        match survey_year {
            Some(AttributeValue::Number(n)) => {
                assert_eq!(n.as_i64().unwrap(), 2022);
            }
            _ => panic!(
                "Valid numeric attributes should still be present, got {:?}",
                survey_year
            ),
        }
    }

    /// Test attribute tracking is per feature type
    #[test]
    fn test_attribute_tracking_per_feature_type() {
        let building_attrs = citygml_attrs_from_json(r#"{"bldg:measuredHeight": 10.5}"#);
        let building = create_test_feature("b1", "bldg:Building", "bldg", building_attrs, "m.gml");
        let road_attrs = citygml_attrs_from_json(r#"{"tran:function": "道路"}"#);
        let road = create_test_feature("t1", "tran:Road", "tran", road_attrs, "m.gml");

        let mut flattener = AttributeFlattener {
            filter_existing_flatten_attributes: true,
            ..Default::default()
        };
        let _ = flattener.flatten_feature(building).unwrap();
        let _ = flattener.flatten_feature(road).unwrap();

        let bldg_schema = flattener.generate_schema_feature("bldg/bldg:Building");
        let road_schema = flattener.generate_schema_feature("tran/tran:Road");

        assert!(bldg_schema
            .attributes
            .contains_key(&Attribute::new("bldg:measuredHeight".to_string())));
        assert!(!road_schema
            .attributes
            .contains_key(&Attribute::new("bldg:measuredHeight".to_string())));
        assert!(road_schema
            .attributes
            .contains_key(&Attribute::new("tran:function".to_string())));
        assert!(!bldg_schema
            .attributes
            .contains_key(&Attribute::new("tran:function".to_string())));
    }

    /// Test all attributes on processed feature are tracked
    #[test]
    fn test_all_final_attributes_tracked() {
        let attrs = citygml_attrs_from_json(r#"{"bldg:class": "普通建物", "bldg:usage": "住宅"}"#);
        let feature = create_test_feature("b1", "bldg:Building", "bldg", attrs, "m.gml");

        let mut flattener = AttributeFlattener {
            filter_existing_flatten_attributes: true,
            ..Default::default()
        };
        let _ = flattener.flatten_feature(feature).unwrap();
        let schema = flattener.generate_schema_feature("bldg/bldg:Building");

        assert!(schema
            .attributes
            .contains_key(&Attribute::new("bldg:class".to_string())));
        assert!(schema
            .attributes
            .contains_key(&Attribute::new("bldg:usage".to_string())));
    }
}
