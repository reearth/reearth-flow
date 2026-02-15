use std::{
    collections::HashMap,
    io::{BufRead, Cursor},
    str::FromStr,
    sync::Arc,
};

use reearth_flow_common::{csv::Delimiter, uri::Uri};
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

/// Decode bytes from the specified encoding to UTF-8.
///
/// If encoding is None or "UTF-8", returns the original bytes unchanged.
/// Otherwise, uses `encoding_rs` to decode (matching the ShapefileReader pattern).
fn decode_content<'a>(
    content: &'a [u8],
    encoding: Option<&str>,
) -> Result<std::borrow::Cow<'a, [u8]>, super::errors::FeatureProcessorError> {
    let encoding_name = match encoding {
        Some(name) if !name.is_empty() => name,
        _ => return Ok(std::borrow::Cow::Borrowed(content)),
    };

    let name_upper = encoding_name.to_uppercase();
    if matches!(name_upper.as_str(), "UTF-8" | "UTF8" | "UNICODE" | "UTF_8") {
        return Ok(std::borrow::Cow::Borrowed(content));
    }

    let enc = encoding_rs::Encoding::for_label(encoding_name.as_bytes()).ok_or_else(|| {
        super::errors::FeatureProcessorError::FileCsvReader(format!(
            "Unsupported encoding: {encoding_name}"
        ))
    })?;

    let (decoded, _, had_errors) = enc.decode(content);
    if had_errors {
        tracing::warn!(
            "Encoding conversion from {} had unmappable characters (replaced with U+FFFD)",
            enc.name()
        );
    }
    Ok(std::borrow::Cow::Owned(decoded.into_owned().into_bytes()))
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
    let decoded = decode_content(&byte, encoding)?;
    // Use BufRead to skip offset lines, because the csv crate silently
    // skips blank lines which causes the offset count to be wrong when the
    // file contains empty rows (e.g. JMA weather data CSVs).
    let mut cursor = Cursor::new(decoded.as_ref());
    let offset = csv_params.offset.unwrap_or(0);
    for _ in 0..offset {
        let mut line = String::new();
        cursor.read_line(&mut line).map_err(|e| {
            super::errors::FeatureProcessorError::FileCsvReader(format!(
                "Failed to skip offset line: {e:?}"
            ))
        })?;
    }
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .trim(csv::Trim::All)
        .delimiter(delimiter.into())
        .from_reader(cursor);
    let header_rows = csv_params.header_rows.unwrap_or(1);
    let mut header = if header_rows == 0 {
        Vec::new()
    } else {
        let mut iter = rdr.deserialize();
        // Read `header_rows` consecutive rows and merge them
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(header_rows);
        for _ in 0..header_rows {
            let row: Vec<String> = iter.next().unwrap_or(Ok(Vec::new())).map_err(|e| {
                super::errors::FeatureProcessorError::FileCsvReader(format!("{e:?}"))
            })?;
            rows.push(row);
        }
        // Determine maximum column count across all header rows
        let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        // Merge: for each column, join non-empty values with "_"
        (0..max_cols)
            .map(|col_idx| {
                rows.iter()
                    .filter_map(|row| row.get(col_idx).map(|s| s.trim()).filter(|s| !s.is_empty()))
                    .collect::<Vec<_>>()
                    .join("_")
            })
            .collect::<Vec<String>>()
    };
    for rd in rdr.deserialize() {
        let record: Vec<String> =
            rd.map_err(|e| super::errors::FeatureProcessorError::FileCsvReader(format!("{e:?}")))?;

        // Auto-generate column names when headerRows is 0
        if header_rows == 0 && header.is_empty() {
            header = (0..record.len())
                .map(|i| format!("column{}", i + 1))
                .collect();
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
