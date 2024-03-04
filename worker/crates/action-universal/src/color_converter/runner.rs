use std::sync::Arc;

use serde::{Deserialize, Serialize};

use reearth_flow_action::{Action, ActionContext, ActionDataframe, ActionResult};

use super::hsl_to_rgba;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ColorConverter {
    #[serde(rename = "hslToRgba")]
    HslToRgba {
        #[serde(flatten)]
        property: hsl_to_rgba::HslPropertySchema,
    },
}

#[async_trait::async_trait]
#[typetag::serde(name = "colorConverter")]
impl Action for ColorConverter {
    async fn run(&self, ctx: ActionContext, input: Option<ActionDataframe>) -> ActionResult {
        let data = match self {
            Self::HslToRgba { property } => {
                hsl_to_rgba::convert_hsl_to_rgba(Arc::clone(&ctx.expr_engine), property, input)
                    .await?
            }
        };
        Ok(data)
    }
}
