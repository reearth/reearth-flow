use std::time::Instant;
use std::{collections::HashMap, sync::Arc};

use petgraph::graph::NodeIndex;
use reearth_flow_state::State;
use reearth_flow_workflow::graph::NodeAction;
use tracing::info_span;

use reearth_flow_action::{Action, ActionContext, ActionDataframe};
use reearth_flow_action_log::action_log;
#[allow(unused_imports)]
use reearth_flow_action_universal::prelude::*;

pub(crate) struct ActionRunner;

impl ActionRunner {
    pub(crate) async fn run_action(
        ctx: ActionContext,
        action: NodeAction,
        ix: NodeIndex,
        dataframe_state: Arc<State>,
        input: Option<ActionDataframe>,
    ) -> anyhow::Result<(NodeIndex, ActionDataframe)> {
        let node_id = ctx.node_id;
        let node_name = ctx.node_name.clone();
        let start_logger = Arc::clone(&ctx.logger);
        let end_logger = Arc::clone(&ctx.logger);
        let span = info_span!(
            parent: ctx.root_span.clone(), "run_async",
            "otel.name" = action.to_string().as_str(),
            "otel.kind" = "action",
            "workflow.action" = format!("{:?}", action),
            "workflow.node_id" = node_id.to_string().as_str(),
            "workflow.node_name" = node_name.as_str()
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
        ))?;
        let res = action_run.run(ctx, input).await?;
        dataframe_state
            .save(&convert_dataframe(&res), node_id.to_string().as_str())
            .await?;
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
