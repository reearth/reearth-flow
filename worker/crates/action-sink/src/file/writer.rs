use std::collections::HashMap;
use std::{str::FromStr, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_eval_expr::utils::dynamic_to_value;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Expr, Feature};
use rhai::Dynamic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use reearth_flow_common::uri::Uri;
use serde_json::Value;

use crate::errors::SinkError;

use super::excel::write_excel;
use super::gltf::write_gltf;

#[derive(Debug, Clone, Default)]
pub struct FileWriterSinkFactory;

impl SinkFactory for FileWriterSinkFactory {
    fn name(&self) -> &str {
        "FileWriter"
    }

    fn description(&self) -> &str {
        "Writes features to a file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FileWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to serialize `with` parameter: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to deserialize `with` parameter: {}", e))
            })?
        } else {
            return Err(
                SinkError::BuildFactory("Missing required parameter `with`".to_string()).into(),
            );
        };

        let sink = FileWriter {
            params,
            buffer: Vec::new(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub struct FileWriter {
    pub(super) params: FileWriterParam,
    pub(super) buffer: Vec<Feature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileWriterCommonParam {
    pub(super) output: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "format")]
pub enum FileWriterParam {
    Csv {
        #[serde(flatten)]
        common_property: FileWriterCommonParam,
    },
    Tsv {
        #[serde(flatten)]
        common_property: FileWriterCommonParam,
    },
    Json {
        #[serde(flatten)]
        common_property: FileWriterCommonParam,
        #[serde(flatten)]
        json_property: JsonWriterParam,
    },
    Excel {
        #[serde(flatten)]
        common_property: FileWriterCommonParam,
    },
    Gltf {
        #[serde(flatten)]
        common_property: FileWriterCommonParam,
    },
}

impl FileWriterParam {
    pub fn to_common_param(&self) -> &FileWriterCommonParam {
        match self {
            FileWriterParam::Csv { common_property } => common_property,
            FileWriterParam::Tsv { common_property } => common_property,
            FileWriterParam::Json {
                common_property, ..
            } => common_property,
            FileWriterParam::Excel { common_property } => common_property,
            FileWriterParam::Gltf { common_property } => common_property,
        }
    }
}

impl Sink for FileWriter {
    fn initialize(&self, _ctx: NodeContext) {}
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        self.buffer.push(ctx.feature);
        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let common_param = self.params.to_common_param();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(common_param.output.as_ref())
            .unwrap_or_else(|_| common_param.output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;
        let result = match &self.params {
            FileWriterParam::Json { json_property, .. } => write_json(
                &output,
                json_property,
                &self.buffer,
                expr_engine,
                storage_resolver,
            ),
            FileWriterParam::Csv { .. } => {
                write_csv(&output, &self.buffer, Delimiter::Comma, storage_resolver)
            }
            FileWriterParam::Tsv { .. } => {
                write_csv(&output, &self.buffer, Delimiter::Tab, storage_resolver)
            }
            FileWriterParam::Excel { .. } => write_excel(&output, &self.buffer, storage_resolver),
            FileWriterParam::Gltf { .. } => write_gltf(&output, &self.buffer, storage_resolver),
        };
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonWriterParam {
    pub(super) converter: Option<Expr>,
}

fn write_json(
    output: &Uri,
    params: &JsonWriterParam,
    features: &[Feature],
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
) -> Result<(), crate::errors::SinkError> {
    let json_value: serde_json::Value = if let Some(converter) = &params.converter {
        let scope = expr_engine.new_scope();
        let value: serde_json::Value = serde_json::Value::Array(
            features
                .iter()
                .map(|feature| {
                    serde_json::Value::Object(
                        feature
                            .attributes
                            .clone()
                            .into_iter()
                            .map(|(k, v)| (k.into_inner().to_string(), v.into()))
                            .collect::<serde_json::Map<_, _>>(),
                    )
                })
                .collect::<Vec<_>>(),
        );
        scope.set("__features", value);
        let convert = scope.eval::<Dynamic>(converter.as_ref()).map_err(|e| {
            crate::errors::SinkError::FileWriter(format!("Failed to evaluate converter: {:?}", e))
        })?;
        dynamic_to_value(&convert)
    } else {
        features.into()
    };
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(json_value.to_string()))
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    Ok(())
}

fn write_csv(
    output: &Uri,
    features: &[Feature],
    delimiter: Delimiter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<(), crate::errors::SinkError> {
    if features.is_empty() {
        return Ok(());
    }
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter.into())
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_writer(vec![]);
    let rows: Vec<AttributeValue> = features.iter().map(|f| f.clone().into()).collect();
    let mut fields = get_fields(rows.first().unwrap());

    if let Some(ref mut fields) = fields {
        // Remove _id field
        fields.retain(|field| field != "_id");
        // Write header
        if !fields.is_empty() {
            wtr.write_record(fields.clone())
                .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
        }
    }

    for row in rows {
        match fields {
            Some(ref fields) if !fields.is_empty() => {
                let values = get_row_values(&row, &fields.clone())?;
                wtr.write_record(values)
                    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
            }
            _ => match row {
                AttributeValue::String(s) => wtr
                    .write_record(vec![s])
                    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?,
                AttributeValue::Array(s) => {
                    let values = s
                        .into_iter()
                        .map(|v| match v {
                            AttributeValue::String(s) => s,
                            _ => "".to_string(),
                        })
                        .collect::<Vec<_>>();
                    wtr.write_record(values)
                        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?
                }
                _ => {
                    return Err(crate::errors::SinkError::FileWriter(
                        "Unsupported input".to_string(),
                    ))
                }
            },
        }
    }
    wtr.flush()
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    let data = String::from_utf8(
        wtr.into_inner()
            .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?,
    )
    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(data))
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{:?}", e)))?;
    Ok(())
}

fn get_fields(row: &AttributeValue) -> Option<Vec<String>> {
    match row {
        AttributeValue::Map(row) => Some(row.keys().cloned().collect::<Vec<_>>()),
        _ => None,
    }
}

fn get_row_values(
    row: &AttributeValue,
    fields: &[String],
) -> Result<Vec<String>, crate::errors::SinkError> {
    fields
        .iter()
        .map(|field| match row {
            AttributeValue::Map(row) => row.get(field).map(|v| v.to_string()).ok_or_else(|| {
                crate::errors::SinkError::FileWriter(format!("Field not found: {}", field))
            }),
            _ => Err(crate::errors::SinkError::FileWriter(
                "Unsupported input".to_string(),
            )),
        })
        .collect()
}
