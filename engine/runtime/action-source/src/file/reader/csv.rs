use std::io::Cursor;

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
    /// # Geometry Configuration
    /// Optional configuration for parsing geometry from CSV columns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) geometry: Option<GeometryConfig>,
}

pub(crate) async fn read_csv(
    delimiter: Delimiter,
    content: &Bytes,
    props: &CsvReaderParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let cursor = Cursor::new(content);
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .delimiter(delimiter.into())
        .from_reader(cursor);
    let offset = props.offset.unwrap_or(0);
    let header = rdr
        .deserialize()
        .nth(offset)
        .unwrap_or(Ok(Vec::<String>::new()))
        .map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;
    for rd in rdr.deserialize() {
        let record: Vec<String> =
            rd.map_err(|e| crate::errors::SourceError::CsvFileReader(format!("{e:?}")))?;

        // Build a map of column name -> value for geometry parsing
        let row_map: IndexMap<String, String> = record
            .iter()
            .enumerate()
            .map(|(i, value)| (header[i].clone(), value.clone()))
            .collect();

        // Parse geometry if config is provided and get column names to exclude
        let (geometry, excluded_columns) = if let Some(geom_config) = &props.geometry {
            let geom = super::csv_geometry::parse_geometry(&row_map, geom_config).map_err(|e| {
                crate::errors::SourceError::CsvFileReader(format!("Geometry parse error: {}", e))
            })?;
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
        feature.geometry = geometry;

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
