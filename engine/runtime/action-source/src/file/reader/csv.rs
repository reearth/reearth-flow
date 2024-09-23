use std::{collections::HashMap, io::Cursor, sync::Arc};

use reearth_flow_common::{csv::Delimiter, uri::Uri};
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CsvReaderParam {
    pub(super) offset: Option<usize>,
}

pub(crate) async fn read_csv(
    delimiter: Delimiter,
    input_path: Uri,
    props: &CsvReaderParam,
    storage_resolver: Arc<StorageResolver>,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let result = storage
        .get(input_path.path().as_path())
        .await
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let byte = result
        .bytes()
        .await
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let cursor = Cursor::new(byte);
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
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    for rd in rdr.deserialize() {
        let record: Vec<String> =
            rd.map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
        let row = record
            .iter()
            .enumerate()
            .map(|(i, value)| (header[i].clone(), AttributeValue::String(value.clone())))
            .collect::<HashMap<String, AttributeValue>>();
        let feature = Feature::from(row);
        sender
            .send((
                DEFAULT_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    }
    Ok(())
}
