use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::csv::{
    auto_generate_header, build_csv_reader, read_merged_header, Delimiter,
};
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

pub(crate) async fn read_csv(
    delimiter: Delimiter,
    content: &Bytes,
    props: &CsvReaderParam,
    encoding: Option<&str>,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let offset = props.offset.unwrap_or(0);
    let mut rdr = build_csv_reader(content.as_ref(), encoding, delimiter, offset)
        .map_err(crate::errors::SourceError::CsvFileReader)?;

    let header_rows = props.header_rows.unwrap_or(1);
    let mut header = read_merged_header(&mut rdr, header_rows)
        .map_err(crate::errors::SourceError::CsvFileReader)?;

    for rd in rdr.deserialize() {
        let record: Vec<String> =
            rd.map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;

        if header_rows == 0 && header.is_empty() {
            header = auto_generate_header(record.len());
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
