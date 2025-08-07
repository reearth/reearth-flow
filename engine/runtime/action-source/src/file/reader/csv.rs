use std::io::Cursor;

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_types::{AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CsvReaderParam {
    /// # Header Row Offset
    /// Skip this many rows from the beginning to find the header row (0 = first row is header)
    pub(crate) offset: Option<usize>,
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
        let row = record
            .iter()
            .enumerate()
            .map(|(i, value)| (header[i].clone(), AttributeValue::String(value.clone())))
            .collect::<IndexMap<String, AttributeValue>>();
        let feature = Feature::from(row);
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
