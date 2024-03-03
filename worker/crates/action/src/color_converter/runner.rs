use core::result::Result;
use std::sync::Arc;

use anyhow::{anyhow, Ok};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use reearth_flow_macros::PropertySchema;

use super::hsl_to_rgba;
use crate::action::{ActionContext, ActionDataframe, ActionResult, ActionRunner};

#[derive(Serialize, Deserialize, Debug, Clone, PropertySchema)]
#[serde(tag = "type")]
pub(crate) enum PropertySchema {
    #[serde(rename = "hslToRgba")]
    HslToRgba {
        #[serde(flatten)]
        property: hsl_to_rgba::HslPropertySchema,
    },
}

pub(crate) struct ColorConverter;

#[async_trait::async_trait]
impl ActionRunner for ColorConverter {
    async fn run(&self, ctx: ActionContext, input: Option<ActionDataframe>) -> ActionResult {
        let props = PropertySchema::try_from(ctx.node_property)?;
        debug!(?props, "read");
        let data = match props {
            PropertySchema::HslToRgba { property } => {
                hsl_to_rgba::convert_hsl_to_rgba(Arc::clone(&ctx.expr_engine), property, input)
                    .await?
            }
        };
        Ok(data)
    }
}
