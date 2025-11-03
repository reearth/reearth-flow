use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_eval_expr::utils::dynamic_to_value;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Expr, Feature};
use rhai::Dynamic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use reearth_flow_common::uri::Uri;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonWriterParam {
    pub(super) converter: Option<Expr>,
}

pub(super) fn write_json(
    output: &Uri,
    params: &JsonWriterParam,
    features: &[Feature],
    expr_engine: &Arc<Engine>,
    storage_resolver: &Arc<StorageResolver>,
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
            crate::errors::SinkError::FileWriter(format!("Failed to evaluate converter: {e:?}"))
        })?;
        dynamic_to_value(&convert)
    } else {
        let attributes = features
            .iter()
            .map(|f| {
                serde_json::Value::Object(
                    f.attributes
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k.into_inner().to_string(), v.into()))
                        .collect::<serde_json::Map<_, _>>(),
                )
            })
            .collect::<Vec<serde_json::Value>>();
        serde_json::Value::Array(attributes)
    };
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(json_value.to_string()))
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?;
    Ok(())
}

pub(super) fn write_csv(
    output: &Uri,
    features: &[Feature],
    delimiter: Delimiter,
    storage_resolver: &Arc<StorageResolver>,
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
                .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?;
        }
    }

    for row in rows {
        match fields {
            Some(ref fields) if !fields.is_empty() => {
                let values = get_row_values(&row, &fields.clone())?;
                wtr.write_record(values)
                    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?;
            }
            _ => match row {
                AttributeValue::String(s) => wtr
                    .write_record(vec![s])
                    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?,
                AttributeValue::Array(s) => {
                    let values = s
                        .into_iter()
                        .map(|v| match v {
                            AttributeValue::String(s) => s,
                            _ => "".to_string(),
                        })
                        .collect::<Vec<_>>();
                    wtr.write_record(values)
                        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?
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
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?;
    let data = String::from_utf8(
        wtr.into_inner()
            .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?,
    )
    .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?;
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(data))
        .map_err(|e| crate::errors::SinkError::FileWriter(format!("{e:?}")))?;
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
                crate::errors::SinkError::FileWriter(format!("Field not found: {field}"))
            }),
            _ => Err(crate::errors::SinkError::FileWriter(
                "Unsupported input".to_string(),
            )),
        })
        .collect()
}
