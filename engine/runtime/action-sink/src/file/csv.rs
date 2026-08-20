use std::collections::HashMap;

use bytes::Bytes;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
use reearth_flow_types::{AttributeValue, Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use crate::SinkOutput;

#[derive(Debug, Clone, Default)]
pub(crate) struct CsvWriterFactory;

impl SinkFactory for CsvWriterFactory {
    fn name(&self) -> &str {
        "CSV Writer"
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
        vec![FEATURES_PORT.clone()]
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
        let params: CsvWriterParam = if let Some(with) = with {
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
        let output = params.output.compile().map_err(|e| {
            SinkError::CsvWriterFactory(format!("Failed to compile `output`: {e:?}"))
        })?;
        let sink = CsvWriter {
            format: params.format,
            geometry: params.geometry,
            output,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct CsvWriter {
    format: CsvFormat,
    geometry: Option<super::writer_geometry::GeometryExportConfig>,
    output: CompiledCode,
    pub(super) buffer: HashMap<String, (SinkOutput, Vec<Feature>)>,
}

/// # CsvWriter Parameters
///
/// Configuration for writing features to CSV/TSV files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CsvWriterParam {
    /// # Output File
    /// Output path or expression for the CSV/TSV file to create.
    pub(super) output: Code,
    /// # File Format
    /// File format to write: CSV (comma-separated) or TSV (tab-separated).
    format: CsvFormat,
    /// # Geometry Configuration
    /// Optional configuration for exporting geometry to CSV columns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) geometry: Option<super::writer_geometry::GeometryExportConfig>,
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
        "CSV Writer"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let path = self
            .output
            .eval_string(&ctx.feature, ctx.variables.clone())
            .map_err(|e| SinkError::CsvWriter(format!("{e:?}")))?;
        let feature = ctx.feature.clone();
        let node_ctx: NodeContext = ctx.into();
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
        let delimiter = self.format.delimiter();
        for (out, features) in self.buffer.values() {
            write_csv(out, features, delimiter.clone(), self.geometry.as_ref())?;
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
    let failed = write_records(&mut wtr, features, geometry_config, out.uri())?;
    wtr.flush()
        .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
    let data = String::from_utf8(
        wtr.into_inner()
            .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?,
    )
    .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
    out.write(Bytes::from(data))
        .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
    // Logged here, after the write has succeeded, so the count stays
    // attributable to this file's path even though `write_records` (shared by
    // every buffered output file in a run) has no access to `out`.
    if failed > 0 {
        tracing::warn!(
            failed,
            "{failed} feature(s) could not export geometry to {}",
            out.uri()
        );
    }
    Ok(())
}

/// Writes the header record (if any) and then one record per feature directly
/// into `wtr`, streaming rows rather than buffering them all in memory first.
/// Returns the number of features whose geometry could not be exported.
fn write_records<W: std::io::Write>(
    wtr: &mut csv::Writer<W>,
    features: &[Feature],
    geometry_config: Option<&super::writer_geometry::GeometryExportConfig>,
    output: &Uri,
) -> Result<usize, crate::errors::SinkError> {
    if features.is_empty() {
        return Ok(0);
    }

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
            wtr.write_record(fields)
                .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
        }
    }

    // Accumulated over the rows so one line reports the file's failures rather
    // than one per row; the per-feature warning below names which feature failed.
    let mut failed = 0usize;

    // An empty attribute set is still a row when geometry columns were asked for:
    // a feature can carry geometry and nothing else. Without this, the header is
    // written and then the first row fails the whole file.
    let has_columns = attribute_fields
        .as_ref()
        .is_some_and(|fields| !fields.is_empty())
        || !geometry_columns.is_empty();

    // Write rows with geometry
    for (feature, row) in features.iter().zip(rows.iter()) {
        match attribute_fields {
            Some(ref attr_fields) if has_columns => {
                // Get attribute values only (not geometry)
                let mut values = get_row_values(row, attr_fields)?;

                // Add geometry values if configured
                if let Some(config) = geometry_config {
                    // Carries feature_id and output context onto every warning
                    // emitted while exporting this feature's geometry,
                    // including ones raised deep inside the geometry module
                    // (e.g. `warn_omitted`, `warn_mixed_frames`) that have no
                    // access to the feature or the destination themselves.
                    // WARN-level so the span stays enabled (and thus current,
                    // and thus attached to those warnings) under a
                    // WARN-filtered subscriber, which is how `cargo make
                    // test-qc` and production workers commonly run; an
                    // `info_span!` would be compiled out under that filter
                    // and the warnings would carry no context at all. Only
                    // entered inside this `if let Some(config) = ...` block,
                    // so a CSV written with no geometry parameter creates no
                    // span at all.
                    let _span = tracing::warn_span!(
                        "csv_geometry_export",
                        feature_id = %feature.id,
                        output = %output,
                    )
                    .entered();
                    match super::writer_geometry::export_geometry(&feature.geometry, config) {
                        Ok(geom_cols) => {
                            // Append geometry column values in the order specified in header
                            for col_name in &geometry_columns {
                                values.push(geom_cols.get(col_name).cloned().unwrap_or_default());
                            }
                        }
                        Err(e) => {
                            failed += 1;
                            tracing::warn!(
                                feature_id = %feature.id,
                                error = %e,
                                "failed to export geometry; writing empty geometry columns"
                            );
                            for _ in &geometry_columns {
                                values.push(String::new());
                            }
                        }
                    }
                }

                wtr.write_record(&values)
                    .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
            }
            _ => match row {
                AttributeValue::String(s) => {
                    wtr.write_record([s])
                        .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
                }
                AttributeValue::Array(s) => {
                    let values = s
                        .iter()
                        .map(|v| match v {
                            AttributeValue::String(s) => s.clone(),
                            _ => String::new(),
                        })
                        .collect::<Vec<_>>();
                    wtr.write_record(&values)
                        .map_err(|e| crate::errors::SinkError::CsvWriter(format!("{e:?}")))?;
                }
                _ => {
                    return Err(crate::errors::SinkError::CsvWriter(
                        "Unsupported input".to_string(),
                    ))
                }
            },
        }
    }

    Ok(failed)
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use indexmap::IndexMap;
    use reearth_flow_types::Attribute;

    use super::super::writer_geometry::{GeometryExportConfig, GeometryExportMode};
    use super::*;

    fn wkt_config() -> GeometryExportConfig {
        GeometryExportConfig {
            mode: GeometryExportMode::Wkt {
                column: "wkt".to_string(),
            },
            epsg_column: None,
        }
    }

    /// Streams `features` through `write_records` into an in-memory CSV
    /// writer and returns the resulting bytes decoded as text, so tests
    /// assert on the real serialized output rather than an intermediate
    /// `Vec<Vec<String>>`.
    fn written(features: &[Feature], geometry: Option<&GeometryExportConfig>) -> String {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(Delimiter::Comma.into())
            .quote_style(csv::QuoteStyle::NonNumeric)
            .from_writer(vec![]);
        let output = Uri::from_str("file:///tmp/test.csv").unwrap();
        write_records(&mut wtr, features, geometry, &output).expect("features should write");
        wtr.flush().unwrap();
        String::from_utf8(wtr.into_inner().unwrap()).unwrap()
    }

    // A feature can carry geometry and nothing else. Writing the header and then
    // failing the row would leave a CSV whose single column has no values.
    #[test]
    fn a_feature_with_no_attributes_writes_its_geometry_columns() {
        let feature = Feature::new_with_attributes(IndexMap::new());
        let csv_text = written(&[feature], Some(&wkt_config()));
        assert_eq!(csv_text, "\"wkt\"\n\"\"\n");
    }

    // The pre-existing path stays as it was: attributes with no geometry config.
    #[test]
    fn a_feature_with_attributes_writes_them_unchanged() {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("category".to_string()),
            AttributeValue::String("A".to_string()),
        );
        let feature = Feature::new_with_attributes(attributes);
        let csv_text = written(&[feature], None);
        // AttributeValue::String("A").to_string() yields the bare string "A"
        // (no embedded quote characters); the csv crate's
        // quote_style(NonNumeric) is what adds the surrounding quotes seen in
        // the actual output file, so there is no double-quoting here.
        assert_eq!(csv_text, "\"category\"\n\"A\"\n");
    }

    // `write_csv`'s early bail on `features.is_empty()` depends on this: an
    // empty feature set must produce no output at all, not an error or a
    // header-only record, so that no file gets written.
    #[test]
    fn no_features_produces_no_records() {
        let csv_text = written(&[], None);
        assert_eq!(csv_text, "");
    }

    // The guard fix only widens the matched arm for the geometry-columns
    // case; a feature with no attributes and no geometry config must still
    // fall through to the unsupported-input error, unchanged.
    #[test]
    fn a_feature_with_no_attributes_and_no_geometry_config_still_errors() {
        let feature = Feature::new_with_attributes(IndexMap::new());
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(Delimiter::Comma.into())
            .quote_style(csv::QuoteStyle::NonNumeric)
            .from_writer(vec![]);
        let output = Uri::from_str("file:///tmp/test.csv").unwrap();
        assert!(write_records(&mut wtr, &[feature], None, &output).is_err());
    }
}
