use std::io::Cursor;
use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::{AttributeValue, Result};
use reearth_flow_common::{csv::Delimiter, str::remove_bom, uri::Uri};
use reearth_flow_storage::resolve::StorageResolver;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CsvPropertySchema {
    pub(super) header: bool,
    pub(super) offset: Option<usize>,
}

pub(super) async fn read_csv(
    delimiter: Delimiter,
    input_path: Uri,
    props: &CsvPropertySchema,
    storage_resolver: Arc<StorageResolver>,
) -> Result<Vec<AttributeValue>> {
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(Error::input)?;
    let result = storage
        .get(input_path.path().as_path())
        .await
        .map_err(Error::internal_runtime)?;
    let byte = result.bytes().await.map_err(Error::internal_runtime)?;
    if props.header {
        let cursor = Cursor::new(byte);
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .delimiter(delimiter.into())
            .from_reader(cursor);
        let offset = props.offset.unwrap_or(0);
        let mut result: Vec<AttributeValue> = Vec::new();
        let header = rdr
            .deserialize()
            .nth(offset)
            .unwrap_or(Ok(Vec::<String>::new()))
            .map_err(Error::internal_runtime)?;
        for rd in rdr.deserialize() {
            let record: Vec<String> = rd.map_err(Error::internal_runtime)?;
            let row = record
                .iter()
                .enumerate()
                .map(|(i, value)| (header[i].clone(), AttributeValue::String(value.clone())))
                .collect::<HashMap<String, AttributeValue>>();
            result.push(AttributeValue::Map(row));
        }
        Ok(result)
    } else {
        let raw_str = String::from_utf8(byte.to_vec())
            .map_err(|e| Error::internal_runtime(format!("Failed to parse csv: {}", e)))?;
        let raw_str = remove_bom(&raw_str);
        let rows = raw_str
            .lines()
            .map(|line| {
                AttributeValue::Array(
                    line.split(',')
                        .map(|s| AttributeValue::String(s.to_string()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        Ok(rows)
    }
}
