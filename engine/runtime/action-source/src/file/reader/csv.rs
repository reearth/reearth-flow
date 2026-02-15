use std::io::{BufRead, Cursor};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_types::{AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use super::csv_geometry::GeometryConfig;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CsvReaderParam {
    /// # Header Row Offset
    /// Skip this many rows from the beginning to find the header row (0 = first row is header)
    pub(crate) offset: Option<usize>,
    /// # Header Row Count
    /// Number of consecutive rows that make up the header (default: 1).
    /// When 0, no header rows are read and column names are auto-generated
    /// as "column1", "column2", etc.
    /// When greater than 1, column names are formed by joining non-empty values
    /// from each header row with "_".
    pub(crate) header_rows: Option<usize>,
    /// # Geometry Configuration
    /// Optional configuration for parsing geometry from CSV columns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) geometry: Option<GeometryConfig>,
}

/// Decode bytes from the specified encoding to UTF-8.
///
/// If encoding is None or "UTF-8", returns the original bytes unchanged.
/// Otherwise, uses `encoding_rs` to decode (matching the ShapefileReader pattern).
fn decode_content<'a>(
    content: &'a Bytes,
    encoding: Option<&str>,
) -> Result<std::borrow::Cow<'a, [u8]>, crate::errors::SourceError> {
    let encoding_name = match encoding {
        Some(name) if !name.is_empty() => name,
        _ => return Ok(std::borrow::Cow::Borrowed(content.as_ref())),
    };

    let name_upper = encoding_name.to_uppercase();
    if matches!(name_upper.as_str(), "UTF-8" | "UTF8" | "UNICODE" | "UTF_8") {
        return Ok(std::borrow::Cow::Borrowed(content.as_ref()));
    }

    let enc = encoding_rs::Encoding::for_label(encoding_name.as_bytes()).ok_or_else(|| {
        crate::errors::SourceError::CsvFileReader(format!("Unsupported encoding: {encoding_name}"))
    })?;

    let (decoded, _, had_errors) = enc.decode(content.as_ref());
    if had_errors {
        tracing::warn!(
            "Encoding conversion from {} had unmappable characters (replaced with U+FFFD)",
            enc.name()
        );
    }
    Ok(std::borrow::Cow::Owned(decoded.into_owned().into_bytes()))
}

pub(crate) async fn read_csv(
    delimiter: Delimiter,
    content: &Bytes,
    props: &CsvReaderParam,
    encoding: Option<&str>,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let decoded = decode_content(content, encoding)?;
    // Use BufRead to skip offset lines, because the csv crate silently
    // skips blank lines which causes the offset count to be wrong when the
    // file contains empty rows (e.g. JMA weather data CSVs).
    let mut cursor = Cursor::new(decoded.as_ref());
    let offset = props.offset.unwrap_or(0);
    for _ in 0..offset {
        let mut line = String::new();
        cursor.read_line(&mut line).map_err(|e| {
            crate::errors::SourceError::CsvFileReader(format!("Failed to skip offset line: {e:?}"))
        })?;
    }
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .trim(csv::Trim::All)
        .delimiter(delimiter.into())
        .from_reader(cursor);
    let header_rows = props.header_rows.unwrap_or(1);
    let mut header = if header_rows == 0 {
        Vec::new()
    } else {
        let mut iter = rdr.deserialize();
        // Read `header_rows` consecutive rows and merge them
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(header_rows);
        for _ in 0..header_rows {
            let row: Vec<String> = iter
                .next()
                .unwrap_or(Ok(Vec::new()))
                .map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;
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
            rd.map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;

        // Auto-generate column names when headerRows is 0
        if header_rows == 0 && header.is_empty() {
            header = (0..record.len())
                .map(|i| format!("column{}", i + 1))
                .collect();
        }

        // Build a map of column name -> value for geometry parsing
        let row_map: IndexMap<String, String> = record
            .iter()
            .enumerate()
            .filter_map(|(i, value)| header.get(i).map(|h| (h.clone(), value.clone())))
            .collect();

        // Parse geometry if config is provided and get column names to exclude
        let (geometry, excluded_columns) = if let Some(geom_config) = &props.geometry {
            let geom = super::csv_geometry::parse_geometry(&row_map, geom_config)?;
            let excluded = super::csv_geometry::get_geometry_column_names(geom_config);
            (geom, excluded)
        } else {
            (reearth_flow_types::Geometry::default(), vec![])
        };

        // Convert to attributes, excluding geometry columns
        let attributes = row_map
            .into_iter()
            .filter(|(k, _)| !excluded_columns.contains(k))
            .map(|(k, v)| (k, AttributeValue::String(v)))
            .collect::<IndexMap<String, AttributeValue>>();

        // Create feature with geometry
        let mut feature = Feature::from(attributes);
        feature.geometry = std::sync::Arc::new(geometry);

        sender
            .send((
                DEFAULT_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;
    }
    Ok(())
}
