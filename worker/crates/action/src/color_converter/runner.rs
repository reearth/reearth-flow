use core::result::Result;
use std::sync::Arc;

use anyhow::{anyhow, Ok};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_workflow::graph::NodeProperty;

use super::hsl_to_rgba;
use crate::action::{ActionContext, ActionDataframe};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub(crate) enum PropertySchema {
    #[serde(rename = "hslToRgba")]
    HslToRgba {
        #[serde(flatten)]
        property: hsl_to_rgba::HslPropertySchema,
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
    inputs: Option<ActionDataframe>,
) -> anyhow::Result<ActionDataframe> {
    let props = PropertySchema::try_from(ctx.node_property)?;
    debug!(?props, "read");
    let data = match props {
        PropertySchema::HslToRgba { property } => {
            hsl_to_rgba::convert_hsl_to_rgba(Arc::clone(&ctx.expr_engine), property, inputs).await?
        }
    };
    Ok(data)
}
