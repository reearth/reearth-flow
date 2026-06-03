use std::collections::HashMap;

use bytes::Bytes;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use crate::SinkOutput;

#[derive(Debug, Clone, Default)]
pub(crate) struct CsvWriterFactory;

impl SinkFactory for CsvWriterFactory {
    fn name(&self) -> &str {
        "CsvWriter"
    }

    fn description(&self) -> &str {
        "Writes features to CSV or TSV files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CsvWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["csv"]
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
                SinkError::CsvWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::CsvWriterFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(SinkError::CsvWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let sink = CsvWriter {
            params,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct CsvWriter {
    pub(super) params: CsvWriterParam,
    pub(super) buffer: HashMap<String, (SinkOutput, Vec<Feature>)>,
}

/// # CsvWriter Parameters
///
/// Configuration for writing features to CSV/TSV files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CsvWriterParam {
    /// Output path or expression for the CSV/TSV file to create
    pub(super) output: Expr,
    /// File format: csv (comma) or tsv (tab)
    format: CsvFormat,
    /// # Geometry Configuration
    /// Optional configuration for exporting geometry to CSV columns
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<super::writer_geometry::GeometryExportConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CsvFormat {
    /// # CSV (Comma-Separated Values)
    /// File with comma-separated values
    Csv,
    /// # TSV (Tab-Separated Values)
    /// File with tab-separated values
    Tsv,
}

impl CsvFormat {
    fn delimiter(&self) -> Delimiter {
        match self {
            CsvFormat::Csv => Delimiter::Comma,
            CsvFormat::Tsv => Delimiter::Tab,
        }
    }
}

impl Sink for CsvWriter {
    fn name(&self) -> &str {
        "CsvWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let node_ctx: NodeContext = ctx.clone().into();
        let scope = node_ctx.expr_engine.new_scope();
        let path = scope
            .eval::<String>(self.params.output.as_ref())
            .unwrap_or_else(|_| self.params.output.as_ref().to_string());
        let feature = ctx.feature.clone();
        use std::collections::hash_map::Entry;
        match self.buffer.entry(path.clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().1.push(feature);
            }
            Entry::Vacant(e) => {
                let out = crate::SinkOutput::new(
                    &node_ctx.sandbox_root,
                    &path,
                    &node_ctx.storage_resolver,
                )
                .map_err(|e| SinkError::CsvWriter(e.to_string()))?;
                e.insert((out, vec![feature]));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        let delimiter = self.params.format.delimiter();
        for (out, features) in self.buffer.values() {
            write_csv(
                out,
                features,
                delimiter.clone(),
                self.params.geometry.as_ref(),
            )?;
        }
        Ok(())
    }
}

fn write_csv(
    out: &SinkOutput,
    features: &[Feature],
    delimiter: Delimiter,
    geometry_config: Option<&super::writer_geometry::GeometryExportConfig>,
) -> Result<(), crate::errors::SinkError> {
    if features.is_empty() {
        return Ok(());
    }
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter.into())
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_writer(vec![]);

    // Get geometry column names if geometry export is configured
    let geometry_columns = geometry_config
        .map(super::writer_geometry::get_geometry_column_names)
        .unwrap_or_default();

    let rows: Vec<AttributeValue> = features
        .iter()
        .map(|f| {
            AttributeValue::Map(
                f.attributes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect(),
            )
        })
        .collect();
    let mut attribute_fields = get_fields(rows.first().unwrap());

    // Prepare attribute fields (without geometry columns)
    if let Some(ref mut fields) = attribute_fields {
        // Remove _id field
        fields.retain(|field| field != "_id");
    }

    // Prepare full header fields (including geometry columns)
    let header_fields = if let Some(ref attr_fields) = attribute_fields {
        let mut header = attr_fields.clone();
        header.extend(geometry_columns.iter().cloned());
        Some(header)
    } else {
        None
    };

    // Write header
    if let Some(ref fields) = header_fields {
        if !fields.is_empty() {
            wtr.write_record(fields.clone())
                .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
        }
    }

    // Write rows with geometry
    for (feature, row) in features.iter().zip(rows.iter()) {
        match attribute_fields {
            Some(ref attr_fields) if !attr_fields.is_empty() => {
                // Get attribute values only (not geometry)
                let mut values = get_row_values(row, attr_fields)?;

                // Add geometry values if configured
                if let Some(config) = geometry_config {
                    match super::writer_geometry::export_geometry(&feature.geometry, config) {
                        Ok(geom_cols) => {
                            // Append geometry column values in the order specified in header
                            for col_name in &geometry_columns {
                                values.push(geom_cols.get(col_name).cloned().unwrap_or_default());
                            }
                        }
                        Err(e) => {
                            // Skip non-point geometries for coordinate mode, or log error for WKT mode
                            tracing::warn!("Failed to export geometry: {}", e);
                            // Write empty strings for geometry columns
                            for _ in &geometry_columns {
                                values.push(String::new());
                            }
                        }
                    }
                }

                wtr.write_record(values)
                    .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
            }
            _ => match row {
                AttributeValue::String(s) => wtr
                    .write_record(vec![s])
                    .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?,
                AttributeValue::Array(s) => {
                    let values = s
                        .iter()
                        .map(|v| match v {
                            AttributeValue::String(s) => s.clone(),
                            _ => String::new(),
                        })
                        .collect::<Vec<_>>();
                    wtr.write_record(values)
                        .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?
                }
                _ => {
                    return Err(crate::errors::SinkError::CsvWriter(
                        "Unsupported input".to_string(),
                    ))
                }
            },
        }
    }
    wtr.flush()
        .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
    let data = String::from_utf8(
        wtr.into_inner()
            .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?,
    )
    .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
    out.write(Bytes::from(data))
        .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
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
                crate::errors::SinkError::CsvWriter(format!("Field not found: {field}"))
            }),
            _ => Err(crate::errors::SinkError::CsvWriter(
                "Unsupported input".to_string(),
            )),
        })
        .collect()
}
