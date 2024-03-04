use std::io::Cursor;
use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use reearth_flow_common::{csv::Delimiter, str::remove_bom, uri::Uri};
use reearth_flow_storage::resolve::StorageResolver;

use crate::action::ActionValue;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CsvPropertySchema {
    pub(crate) header: bool,
    pub(crate) offset: Option<usize>,
}

pub(crate) async fn read_csv(
    delimiter: Delimiter,
    input_path: Uri,
    props: &CsvPropertySchema,
    storage_resolver: Arc<StorageResolver>,
) -> anyhow::Result<Vec<ActionValue>> {
    let storage = storage_resolver.resolve(&input_path)?;
    let result = storage.get(input_path.path().as_path()).await?;
    let byte = result.bytes().await?;
    if props.header {
        let cursor = Cursor::new(byte);
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)
            .delimiter(delimiter.into())
            .from_reader(cursor);
        let offset = props.offset.unwrap_or(0);
        let mut result: Vec<ActionValue> = Vec::new();
        let header = rdr
            .deserialize()
            .nth(offset)
            .unwrap_or(Ok(Vec::<String>::new()))?;
        for rd in rdr.deserialize() {
            let record: Vec<String> = rd?;
            let row = record
                .iter()
                .enumerate()
                .map(|(i, value)| (header[i].clone(), ActionValue::String(value.clone())))
                .collect::<HashMap<String, ActionValue>>();
            result.push(ActionValue::Map(row));
        }
        Ok(result)
    } else {
        let raw_str = String::from_utf8(byte.to_vec())
            .map_err(|e| anyhow::anyhow!("Failed to parse csv: {}", e))?;
        let raw_str = remove_bom(&raw_str);
        let rows = raw_str
            .lines()
            .map(|line| {
                ActionValue::Array(
                    line.split(',')
                        .map(|s| ActionValue::String(s.to_string()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        Ok(rows)
    }
}
