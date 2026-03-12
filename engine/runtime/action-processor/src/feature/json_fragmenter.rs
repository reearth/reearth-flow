use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_common::{json::find_by_json_path, uri::Uri};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct JSONFragmenterFactory;

impl ProcessorFactory for JSONFragmenterFactory {
    fn name(&self) -> &str {
        "JSONFragmenter"
    }

    fn description(&self) -> &str {
        "Fragments JSON documents into individual features based on a JSONPath query"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(JSONFragmenterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: JSONFragmenterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::JSONFragmenterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::JSONFragmenterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::JSONFragmenterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(JSONFragmenter { params }))
    }
}

/// Common query and output options shared by both input source modes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JSONFragmenterOptions {
    /// JSONPath expression to select elements (e.g., "$[*]", "$.results[*]")
    json_query: String,
    /// If true, flatten JSON object keys into feature attributes
    #[serde(default)]
    flatten_query_result: bool,
    /// If true, recursively flatten nested objects using dot-separated keys
    #[serde(default)]
    recursively_flatten: bool,
    /// Optional prefix for flattened attribute names
    #[serde(default)]
    attribute_prefix: Option<String>,
    /// If true, reject features that produce no fragments
    #[serde(default)]
    reject_no_fragments: bool,
}

/// # JSONFragmenter Parameters
///
/// Configuration for fragmenting JSON documents into individual features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "inputSource")]
pub enum JSONFragmenterParam {
    /// Read JSON from a feature attribute
    #[serde(rename = "attribute")]
    Attribute {
        #[serde(rename = "jsonAttribute")]
        /// The attribute containing the JSON text to fragment
        json_attribute: Attribute,
        #[serde(flatten)]
        options: JSONFragmenterOptions,
    },
    /// Read JSON from a file path or URL
    #[serde(rename = "fileUrl")]
    FileUrl {
        /// Expression evaluating to the file path or URL containing JSON
        path: Expr,
        #[serde(flatten)]
        options: JSONFragmenterOptions,
    },
}

impl JSONFragmenterParam {
    fn options(&self) -> &JSONFragmenterOptions {
        match self {
            JSONFragmenterParam::Attribute { options, .. }
            | JSONFragmenterParam::FileUrl { options, .. } => options,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JSONFragmenter {
    params: JSONFragmenterParam,
}

impl Processor for JSONFragmenter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let json_text = match &self.params {
            JSONFragmenterParam::Attribute { json_attribute, .. } => match feature
                .attributes
                .get(json_attribute)
            {
                Some(AttributeValue::String(s)) => s.clone(),
                Some(other) => serde_json::to_string(&serde_json::Value::from(other.clone()))
                    .map_err(|e| {
                        FeatureProcessorError::JSONFragmenter(format!(
                            "Failed to serialize attribute value: {e}"
                        ))
                    })?,
                None => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }
            },
            JSONFragmenterParam::FileUrl { path, .. } => {
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let scope = feature.new_scope(expr_engine.clone(), &None);
                let path_ast = expr_engine
                    .compile(path.to_string().as_str())
                    .map_err(|e| {
                        FeatureProcessorError::JSONFragmenter(format!(
                            "Failed to compile path expression: {e:?}"
                        ))
                    })?;
                let url_str: String = scope.eval_ast(&path_ast).map_err(|e| {
                    FeatureProcessorError::JSONFragmenter(format!(
                        "Failed to evaluate path expression: {e:?}"
                    ))
                })?;
                let url = Uri::from_str(&url_str).map_err(|e| {
                    FeatureProcessorError::JSONFragmenter(format!("Invalid URL: {e:?}"))
                })?;
                let storage = ctx.storage_resolver.resolve(&url).map_err(|e| {
                    FeatureProcessorError::JSONFragmenter(format!(
                        "Failed to resolve storage: {e:?}"
                    ))
                })?;
                let bytes = storage.get_sync(&url.path()).map_err(|e| {
                    FeatureProcessorError::JSONFragmenter(format!("Failed to read file: {e:?}"))
                })?;
                String::from_utf8(bytes.to_vec()).map_err(|e| {
                    FeatureProcessorError::JSONFragmenter(format!(
                        "File content is not valid UTF-8: {e}"
                    ))
                })?
            }
        };

        let json_value: Value = match serde_json::from_str(&json_text) {
            Ok(v) => v,
            Err(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        let opts = self.params.options();
        let matches = match find_by_json_path(json_value.clone(), &opts.json_query) {
            Ok(m) => m,
            Err(e) => {
                return Err(FeatureProcessorError::JSONFragmenter(format!(
                    "JSONPath query error: {e}"
                ))
                .into());
            }
        };

        if matches.is_empty() {
            if opts.reject_no_fragments {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            } else {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            return Ok(());
        }

        for (index, matched_value) in matches.into_iter().enumerate() {
            let mut new_feature = feature.clone();
            new_feature.refresh_id();

            let json_type = json_type_name(&matched_value);
            new_feature.insert(
                Attribute::new("json_type"),
                AttributeValue::String(json_type.to_string()),
            );
            new_feature.insert(
                Attribute::new("json_index"),
                AttributeValue::Number(Number::from(index)),
            );

            if let JSONFragmenterParam::Attribute { json_attribute, .. } = &self.params {
                new_feature.insert(
                    json_attribute.clone(),
                    AttributeValue::String(
                        serde_json::to_string(&matched_value).unwrap_or_default(),
                    ),
                );
            }

            if opts.flatten_query_result {
                if let Value::Object(obj) = &matched_value {
                    flatten_object(
                        &mut new_feature,
                        obj,
                        opts.attribute_prefix.as_deref().unwrap_or(""),
                        opts.recursively_flatten,
                    );
                }
            }

            fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
        }

        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "JSONFragmenter"
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn flatten_object(
    feature: &mut Feature,
    obj: &serde_json::Map<String, Value>,
    prefix: &str,
    recursive: bool,
) {
    for (key, value) in obj {
        let attr_name = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match value {
            Value::Object(nested) if recursive => {
                flatten_object(feature, nested, &attr_name, recursive);
            }
            _ => {
                feature.insert(
                    Attribute::new(&attr_name),
                    AttributeValue::from(value.clone()),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Geometry;

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    /// Variant that returns the Result so we can assert on errors.
    fn run_processor_result(
        feature: &Feature,
        params: JSONFragmenterParam,
    ) -> Result<(Vec<Feature>, Vec<Port>), BoxedError> {
        let mut processor = JSONFragmenter { params };
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = create_default_execute_context(feature);
        processor.process(ctx, &fw)?;
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap().clone();
            let ports = noop.send_ports.lock().unwrap().clone();
            Ok((features, ports))
        } else {
            unreachable!()
        }
    }

    fn make_feature_with_json(json: &str) -> Feature {
        let mut feature = Feature::new_with_attributes(Default::default());
        feature.insert(
            Attribute::new("_response_body"),
            AttributeValue::String(json.to_string()),
        );
        feature
    }

    fn run_processor(feature: &Feature, params: JSONFragmenterParam) -> (Vec<Feature>, Vec<Port>) {
        let mut processor = JSONFragmenter { params };
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = create_default_execute_context(feature);
        processor.process(ctx, &fw).unwrap();
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap().clone();
            let ports = noop.send_ports.lock().unwrap().clone();
            (features, ports)
        } else {
            unreachable!()
        }
    }

    fn attribute_params(query: &str) -> JSONFragmenterParam {
        JSONFragmenterParam::Attribute {
            json_attribute: Attribute::new("_response_body"),
            options: JSONFragmenterOptions {
                json_query: query.to_string(),
                flatten_query_result: false,
                recursively_flatten: false,
                attribute_prefix: None,
                reject_no_fragments: false,
            },
        }
    }

    fn attribute_params_with_flatten(
        query: &str,
        recursive: bool,
        prefix: Option<&str>,
    ) -> JSONFragmenterParam {
        JSONFragmenterParam::Attribute {
            json_attribute: Attribute::new("_response_body"),
            options: JSONFragmenterOptions {
                json_query: query.to_string(),
                flatten_query_result: true,
                recursively_flatten: recursive,
                attribute_prefix: prefix.map(|s| s.to_string()),
                reject_no_fragments: false,
            },
        }
    }

    #[test]
    fn test_fragment_json_array() {
        let json = r#"[{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]"#;
        let feature = make_feature_with_json(json);
        let (features, ports) = run_processor(&feature, attribute_params("$[*]"));

        assert_eq!(features.len(), 2);
        assert!(ports.iter().all(|p| *p == DEFAULT_PORT.clone()));

        let body0 = features[0]
            .attributes
            .get(&Attribute::new("_response_body"))
            .unwrap();
        assert!(matches!(body0, AttributeValue::String(s) if s.contains("Alice")));

        let json_type = features[0]
            .attributes
            .get(&Attribute::new("json_type"))
            .unwrap();
        assert_eq!(json_type, &AttributeValue::String("object".to_string()));

        let json_index = features[0]
            .attributes
            .get(&Attribute::new("json_index"))
            .unwrap();
        assert_eq!(json_index, &AttributeValue::Number(Number::from(0)));

        let json_index1 = features[1]
            .attributes
            .get(&Attribute::new("json_index"))
            .unwrap();
        assert_eq!(json_index1, &AttributeValue::Number(Number::from(1)));
    }

    #[test]
    fn test_fragment_nested_query() {
        let json =
            r#"{"data": {"users": [{"name": "Alice"}, {"name": "Bob"}, {"name": "Charlie"}]}}"#;
        let feature = make_feature_with_json(json);
        let (features, ports) = run_processor(&feature, attribute_params("$.data.users[*]"));

        assert_eq!(features.len(), 3);
        assert!(ports.iter().all(|p| *p == DEFAULT_PORT.clone()));
    }

    #[test]
    fn test_flatten_object_keys() {
        let json = r#"[{"name": "Alice", "score": 95, "active": true}]"#;
        let feature = make_feature_with_json(json);
        let (features, _) =
            run_processor(&feature, attribute_params_with_flatten("$[*]", false, None));

        assert_eq!(features.len(), 1);
        let f = &features[0];
        assert_eq!(
            f.attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("Alice".to_string()))
        );
        assert!(matches!(
            f.attributes.get(&Attribute::new("score")),
            Some(AttributeValue::Number(_))
        ));
        assert_eq!(
            f.attributes.get(&Attribute::new("active")),
            Some(&AttributeValue::Bool(true))
        );
    }

    #[test]
    fn test_recursive_flatten() {
        let json = r#"[{"user": {"name": "Alice", "address": {"city": "Tokyo"}}}]"#;
        let feature = make_feature_with_json(json);
        let (features, _) =
            run_processor(&feature, attribute_params_with_flatten("$[*]", true, None));

        assert_eq!(features.len(), 1);
        let f = &features[0];
        assert_eq!(
            f.attributes.get(&Attribute::new("user.name")),
            Some(&AttributeValue::String("Alice".to_string()))
        );
        assert_eq!(
            f.attributes.get(&Attribute::new("user.address.city")),
            Some(&AttributeValue::String("Tokyo".to_string()))
        );
    }

    #[test]
    fn test_flatten_with_prefix() {
        let json = r#"[{"name": "Alice"}]"#;
        let feature = make_feature_with_json(json);
        let (features, _) = run_processor(
            &feature,
            attribute_params_with_flatten("$[*]", false, Some("result")),
        );

        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].attributes.get(&Attribute::new("result.name")),
            Some(&AttributeValue::String("Alice".to_string()))
        );
    }

    #[test]
    fn test_invalid_json_rejected() {
        let feature = make_feature_with_json("not valid json");
        let (_, ports) = run_processor(&feature, attribute_params("$[*]"));

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
    }

    #[test]
    fn test_missing_attribute_rejected() {
        let feature = Feature::new_with_attributes(Default::default());
        let (_, ports) = run_processor(&feature, attribute_params("$[*]"));

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
    }

    #[test]
    fn test_no_matches_reject_mode() {
        let json = r#"{"data": []}"#;
        let feature = make_feature_with_json(json);
        let params = JSONFragmenterParam::Attribute {
            json_attribute: Attribute::new("_response_body"),
            options: JSONFragmenterOptions {
                json_query: "$.data[*]".to_string(),
                flatten_query_result: false,
                recursively_flatten: false,
                attribute_prefix: None,
                reject_no_fragments: true,
            },
        };
        let (_, ports) = run_processor(&feature, params);

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
    }

    #[test]
    fn test_no_matches_pass_through() {
        let json = r#"{"data": []}"#;
        let feature = make_feature_with_json(json);
        let (_, ports) = run_processor(&feature, attribute_params("$.data[*]"));

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], DEFAULT_PORT.clone());
    }

    #[test]
    fn test_preserves_input_attributes() {
        let mut feature = make_feature_with_json(r#"[{"x": 1}]"#);
        feature.insert(
            Attribute::new("existing_attr"),
            AttributeValue::String("preserved".to_string()),
        );
        feature.geometry = Geometry {
            epsg: Some(4326),
            ..Default::default()
        }
        .into();

        let (features, _) = run_processor(&feature, attribute_params("$[*]"));

        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].attributes.get(&Attribute::new("existing_attr")),
            Some(&AttributeValue::String("preserved".to_string()))
        );
        assert_eq!(features[0].geometry.epsg, Some(4326));
    }

    #[test]
    fn test_fragment_single_object_key() {
        let json = r#"{"Depth": 1.5, "Area": "Zone A"}"#;
        let feature = make_feature_with_json(json);
        let (features, ports) = run_processor(&feature, attribute_params("$.Depth"));

        assert_eq!(features.len(), 1);
        assert!(ports.iter().all(|p| *p == DEFAULT_PORT.clone()));
        let json_type = features[0]
            .attributes
            .get(&Attribute::new("json_type"))
            .unwrap();
        assert_eq!(json_type, &AttributeValue::String("number".to_string()));
    }

    #[test]
    fn test_unique_feature_ids() {
        let json = r#"[1, 2, 3]"#;
        let feature = make_feature_with_json(json);
        let (features, _) = run_processor(&feature, attribute_params("$[*]"));

        assert_eq!(features.len(), 3);
        let ids: Vec<_> = features.iter().map(|f| f.id).collect();
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        assert_ne!(ids[0], ids[2]);
        // Each fragment id should differ from the original
        assert!(ids.iter().all(|id| *id != feature.id));
    }

    #[test]
    fn test_invalid_json_query_returns_error() {
        let json = r#"[1, 2, 3]"#;
        let feature = make_feature_with_json(json);
        // "$[" is syntactically invalid JSONPath
        let result = run_processor_result(&feature, attribute_params("$["));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("JSONPath query error"),
            "Expected JSONPath error, got: {err_msg}"
        );
    }

    #[test]
    fn test_file_url_success() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("data.json");
        let mut f = std::fs::File::create(&file_path).unwrap();
        write!(f, r#"[{{"name": "Alice"}}, {{"name": "Bob"}}]"#).unwrap();
        drop(f);

        let file_uri = format!("file://{}", file_path.display());
        // Build params via serde so the Expr nutype is constructed correctly
        let params: JSONFragmenterParam = serde_json::from_value(serde_json::json!({
            "inputSource": "fileUrl",
            "path": format!("\"{}\"", file_uri),
            "jsonQuery": "$[*]",
            "flattenQueryResult": true
        }))
        .unwrap();

        let feature = Feature::new_with_attributes(Default::default());
        let (features, ports) = run_processor(&feature, params);

        assert_eq!(features.len(), 2);
        assert!(ports.iter().all(|p| *p == DEFAULT_PORT.clone()));
        assert_eq!(
            features[0].attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("Alice".to_string()))
        );
        assert_eq!(
            features[1].attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("Bob".to_string()))
        );
    }

    #[test]
    fn test_file_url_missing_file_returns_error() {
        let params: JSONFragmenterParam = serde_json::from_value(serde_json::json!({
            "inputSource": "fileUrl",
            "path": "\"file:///nonexistent/path/data.json\"",
            "jsonQuery": "$[*]"
        }))
        .unwrap();

        let feature = Feature::new_with_attributes(Default::default());
        let result = run_processor_result(&feature, params);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to read file"),
            "Expected file read error, got: {err_msg}"
        );
    }
}
