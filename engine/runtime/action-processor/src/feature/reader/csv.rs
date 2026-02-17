use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_common::csv::{
    auto_generate_header, build_csv_reader, read_merged_header, Delimiter,
};
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    executor_operation::ExecutorContext, forwarder::ProcessorChannelForwarder, node::DEFAULT_PORT,
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CompiledCommonReaderParam;

/// # CsvReader Parameters
///
/// Configuration for reading CSV data within feature processing workflows.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CsvReaderParam {
    /// The offset of the first row to read
    offset: Option<usize>,
    /// # Header Row Count
    /// Number of consecutive rows that make up the header (default: 1).
    /// When 0, no header rows are read and column names are auto-generated
    /// as "column1", "column2", etc.
    /// When greater than 1, column names are formed by joining non-empty values
    /// from each header row with "_".
    header_rows: Option<usize>,
}

pub(crate) fn read_csv(
    delimiter: Delimiter,
    global_params: &Option<HashMap<String, serde_json::Value>>,
    params: &CompiledCommonReaderParam,
    csv_params: &CsvReaderParam,
    encoding: Option<&str>,
    ctx: ExecutorContext,
    fw: &ProcessorChannelForwarder,
) -> Result<(), super::errors::FeatureProcessorError> {
    let feature = &ctx.feature;
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let storage_resolver = &ctx.storage_resolver;
    let scope = feature.new_scope(expr_engine.clone(), global_params);
    let csv_path = scope
        .eval_ast::<String>(&params.expr)
        .unwrap_or_else(|_| params.original_expr.to_string());
    let input_path = Uri::from_str(csv_path.as_str())
        .map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{e:?}")))?;
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{e:?}")))?;
    let byte = storage
        .get_sync(input_path.path().as_path())
        .map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{e:?}")))?;

    let offset = csv_params.offset.unwrap_or(0);
    let mut rdr = build_csv_reader(&byte, encoding, delimiter, offset)
        .map_err(super::errors::FeatureProcessorError::FileCsvReader)?;

    let header_rows = csv_params.header_rows.unwrap_or(1);
    let mut header = read_merged_header(&mut rdr, header_rows)
        .map_err(super::errors::FeatureProcessorError::FileCsvReader)?;

    for rd in rdr.deserialize() {
        let record: Vec<String> =
            rd.map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{e:?}")))?;

        if header_rows == 0 && header.is_empty() {
            header = auto_generate_header(record.len());
        }
        let row = record
            .iter()
            .enumerate()
            .map(|(i, value)| {
                (
                    Attribute::new(header[i].clone()),
                    AttributeValue::String(value.clone()),
                )
            })
            .collect::<HashMap<Attribute, AttributeValue>>();
        let mut feature = feature.clone();
        feature.extend(row);
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
    }
    Ok(())
}
