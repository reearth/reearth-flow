use std::time::Instant;
use std::{collections::HashMap, sync::Arc};

use petgraph::graph::NodeIndex;
use reearth_flow_state::State;
use reearth_flow_workflow::graph::NodeAction;

use reearth_flow_action::{Action, ActionContext, ActionDataframe};
use reearth_flow_action_log::action_log;
use reearth_flow_action_log::span;
#[allow(unused_imports)]
use reearth_flow_action_plateau::prelude::*;
#[allow(unused_imports)]
use reearth_flow_action_universal::prelude::*;

pub(crate) struct ActionRunner;

impl ActionRunner {
    pub(crate) async fn run(
        ctx: ActionContext,
        action: NodeAction,
        ix: NodeIndex,
        dataframe_state: Arc<State>,
        input: Option<ActionDataframe>,
    ) -> crate::Result<(NodeIndex, ActionDataframe)> {
        let node_id = ctx.node_id;
        let node_name = ctx.node_name.clone();
        let start_logger = Arc::clone(&ctx.logger);
        let end_logger = Arc::clone(&ctx.logger);
        let span = span(
            ctx.root_span.clone(),
            action.to_string(),
            node_id.to_string(),
            node_name.clone(),
        );
        action_log!(
            parent: span,
            start_logger,
            "Start action = {:?}, name = {:?}",
            action,
            node_name,
        );
        let start = Instant::now();
        let action_run: Box<dyn Action> = serde_json::from_value(serde_json::Value::Object(
            vec![
                (
                    "action".to_owned(),
                    serde_json::Value::String(action.to_string()),
                ),
                (
                    "with".to_owned(),
                    serde_json::Value::from(ctx.node_property.clone()),
                ),
            ]
            .into_iter()
            .collect::<serde_json::Map<_, _>>(),
        ))
        .map_err(crate::Error::execution)?;
        let res = action_run
            .run(ctx, input)
            .await
            .map_err(|e| crate::Error::action(e, action.to_string()))?;
        dataframe_state
            .save(&convert_dataframe(&res), node_id.to_string().as_str())
            .await
            .map_err(crate::Error::execution)?;
        let duration = start.elapsed();
        action_log!(
            parent: span,
            end_logger,
            "Finish action = {:?}, name = {:?}, ports = {:?}, duration = {:?}",
            action,
            node_name,
            res.keys(),
            duration,
        );
        Ok((ix, res))
    }
}

fn convert_dataframe(dataframe: &ActionDataframe) -> HashMap<String, serde_json::Value> {
    dataframe
        .iter()
        .filter_map(|(k, v)| match v {
            Some(v) => {
                let value: serde_json::Value = v.clone().into();
                Some((k.clone(), value))
            }
            None => None,
        })
        .collect::<HashMap<String, serde_json::Value>>()
}
