use core::result::Result;
use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_workflow::graph::NodeProperty;

use super::{csv, text};
use crate::action::{ActionContext, ActionDataframe, ActionValue, DEFAULT_PORT};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommonPropertySchema {
    pub(crate) dataset: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "format")]
pub(crate) enum PropertySchema {
    #[serde(rename = "csv")]
    Csv {
        #[serde(flatten)]
        common_property: CommonPropertySchema,
        #[serde(flatten)]
        property: csv::CsvPropertySchema,
    },
    #[serde(rename = "text")]
    Text {
        #[serde(flatten)]
        common_property: CommonPropertySchema,
    },
    #[serde(rename = "json")]
    Json {
        #[serde(flatten)]
        common_property: CommonPropertySchema,
    },
}

impl TryFrom<NodeProperty> for PropertySchema {
    type Error = anyhow::Error;

    fn try_from(node_property: NodeProperty) -> Result<Self, anyhow::Error> {
        serde_json::from_value(Value::Object(node_property)).map_err(|e| {
            anyhow!(
                "Failed to convert NodeProperty to PropertySchema with {}",
                e
            )
        })
    }
}

pub(crate) async fn run(
    ctx: ActionContext,
    _inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    debug!(?props, "read");
    let data = match props {
        PropertySchema::Csv {
            common_property,
            property,
        } => {
            let result = csv::read_csv(&common_property, &property).await?;
            ActionValue::Array(result)
        }
        PropertySchema::Text { common_property } => text::read_text(&common_property).await?,
        _ => return Err(anyhow!("Unsupported format")),
    };
    let mut output = HashMap::new();
    output.insert(DEFAULT_PORT.to_string(), Some(data));
    Ok(output)
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_parse_csv() {
        let json = r#"
        {
            "format": "csv",
            "dataset": "file:///hoge/fuga.csv",
            "header": true
        }
  "#;

        let props = serde_json::from_str::<NodeProperty>(json).unwrap();
        let schema = PropertySchema::try_from(props).unwrap();
        match schema {
            PropertySchema::Csv {
                common_property,
                property,
            } => {
                assert_eq!(common_property.dataset, "file:///hoge/fuga.csv");
                assert!(property.header);
            }
            _ => panic!("Unsupported format"),
        }
    }
}
