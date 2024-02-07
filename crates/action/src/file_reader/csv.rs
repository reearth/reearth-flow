use std::io::Cursor;
use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use reearth_flow_common::{str::remove_bom, uri::Uri};
use reearth_flow_storage::resolve;

use super::base::CommonPropertySchema;
use crate::action::ActionValue;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CsvPropertySchema {
    pub(crate) header: bool,
}

pub(crate) async fn read_csv(
    common_props: &CommonPropertySchema,
    props: &CsvPropertySchema,
) -> anyhow::Result<Vec<ActionValue>> {
    let uri = Uri::from_str(&common_props.dataset)?;
    let storage = resolve(&uri)?;
    let result = storage.get(uri.path().as_path()).await?;
    let byte = result.bytes().await?;
    if props.header {
        let cursor = Cursor::new(byte);
        let mut rdr = csv::Reader::from_reader(cursor);
        let mut result: Vec<ActionValue> = Vec::new();
        for rd in rdr.deserialize() {
            let record: HashMap<String, String> = rd?;
            let mut row: HashMap<String, ActionValue> = HashMap::new();
            record.iter().for_each(|(k, v)| {
                row.insert(k.to_string(), ActionValue::String(v.to_string()));
            });
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
