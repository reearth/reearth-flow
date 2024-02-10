use std::collections::HashMap;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use async_recursion::async_recursion;
use petgraph::graph::NodeIndex;
use reearth_flow_action::action::ActionContext;
use tokio::task::JoinSet;
use tracing::info;

use reearth_flow_action::action::{Action, ActionDataframe};
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_workflow::graph::Node;
use reearth_flow_workflow::id::Id;
use reearth_flow_workflow::workflow::Workflow;

use super::dag_impl::Dag;
use super::error::Error;

type Graphs = HashMap<Id, Dag>;

pub struct DagExecutor {
    name: String,
    entry_dag: Dag,
    sub_dags: Graphs,
    expr_engine: Arc<Engine>,
}

impl DagExecutor {
    pub fn new(workflow: &Workflow) -> Result<Self> {
        let entry_graph = workflow
            .graphs
            .iter()
            .filter(|&graph| graph.id == workflow.entry_graph_id)
            .map(Dag::from_graph)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .next();
        let entry_dag = entry_graph.ok_or(Error::Init(format!(
            "Failed to init entry graph with {}",
            workflow.entry_graph_id
        )))?;
        let sub_dags = workflow
            .graphs
            .iter()
            .filter(|&graph| graph.id != workflow.entry_graph_id)
            .map(|graph| {
                let g = Dag::from_graph(graph)?;
                Ok((graph.id, g))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        let engine = Engine::new();
        workflow.with.iter().for_each(|(k, v)| {
            engine.set_scope_var(k, v);
        });
        Ok(Self {
            name: workflow.name.clone(),
            entry_dag,
            sub_dags,
            expr_engine: Arc::new(engine),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Start workflow = {:?}", self.name);
        let start = Instant::now();
        let _ = self.run_dag(&self.entry_dag).await?;
        let duration = start.elapsed();
        info!(
            "Finish workflow = {:?}, duration = {:?}",
            self.name, duration
        );
        Ok(())
    }

    #[async_recursion]
    pub async fn run_dag(&self, dag: &Dag) -> Result<ActionDataframe> {
        let mut dfs: HashMap<NodeIndex, ActionDataframe> = HashMap::new();
        let entry_node_ids = dag
            .entry_nodes()
            .iter()
            .map(|n| dag.node_index(n).unwrap())
            .collect::<Vec<_>>();
        let mut ready = VecDeque::from_iter(entry_node_ids);
        while !ready.is_empty() {
            let mut results = vec![];
            let mut async_tools = JoinSet::new();
            while let Some(ix) = ready.pop_front() {
                let input = dfs.remove(&ix);
                let node = dag.node_from_index(ix).ok_or(Error::Execution(format!(
                    "Failed to get node from index = {:?}",
                    ix
                )))?;
                let ctx = ActionContext::new(
                    node.id(),
                    node.name().to_owned(),
                    node.with().clone(),
                    Arc::clone(&self.expr_engine),
                );
                match node {
                    Node::Action { action, .. } => {
                        let action = Action::from_str(action)?;
                        async_tools.spawn(async move { run_async(ix, ctx, action, input).await });
                    }
                    Node::SubGraph { sub_graph_id, .. } => {
                        let sub_dag =
                            self.sub_dags
                                .get(sub_graph_id)
                                .ok_or(Error::Execution(format!(
                                    "Failed to get sub graph with id = {:?}",
                                    sub_graph_id
                                )))?;
                        let res = self.run_dag(sub_dag).await?;
                        results.push((ix, res));
                    }
                }
            }
            while let Some(res) = async_tools.join_next().await {
                results.push(res??);
            }
            for (ix, data_frame) in results {
                if dag.is_last_node_index(ix) {
                    dfs.insert(ix, data_frame);
                    continue;
                }
                let edges = dag.edges_from_node_index(ix);
                edges.for_each(|edge| {
                    let to_ix = edge.to_node;
                    let to_port = edge.to_port;
                    let from_port = edge.from_port;
                    let data = dfs.entry(to_ix).or_default();
                    let value = match data_frame.get(&from_port).cloned() {
                        Some(df) => df.clone(),
                        None => None,
                    };
                    data.insert(to_port, value);
                    let finish_all_ports = data.keys().map(|v| v.to_string()).collect::<Vec<_>>();
                    if dag.is_ready_node(to_ix, finish_all_ports) {
                        ready.push_back(to_ix);
                    }
                })
            }
        }
        let mut result = ActionDataframe::new();
        dfs.values().for_each(|value| {
            result.extend(value.clone());
        });
        Ok(result)
    }
}

async fn run_async(
    ix: NodeIndex,
    ctx: ActionContext,
    action: Action,
    input: Option<ActionDataframe>,
) -> Result<(NodeIndex, ActionDataframe)> {
    let node_name = ctx.node_name.clone();
    info!("Start action = {:?}, name = {:?}", action, node_name);
    let func = action.run(ctx, input);
    let res = func.await?;
    info!(
        "Finish action = {:?}, name = {:?}, ports = {:?}",
        action,
        node_name,
        res.keys()
    );
    Ok((ix, res))
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_run() {
        let json = r#"
        {
            "id":"7b66c0a4-e1fa-41dd-a0c9-df3f6e01cc22",
            "name":"hoge-workflow",
            "entryGraphId":"c6863b71-953b-4d15-af56-396fc93fc617",
            "with": {
                "param01": "sample",
                "param02": ["sample1", "sample2"]
            },
            "graphs":[
               {
                  "id":"c6863b71-953b-4d15-af56-396fc93fc617",
                  "name":"hoge-graph",
                  "nodes":[
                     {
                        "id":"a1a91180-ab88-4c1a-aab5-48c242a218ca",
                        "name":"hoge-action-node-01",
                        "type":"action",
                        "action":"fileReader",
                        "with": {"format":"csv","dataset":"ram:///root/summary.csv", "header": true}
                     },
                     {
                        "id":"a1a91180-ab88-4c1a-aab5-48c242a218cb",
                        "name":"hoge-action-node-02",
                        "type":"action",
                        "action":"attributeKeeper",
                        "with": {"keepAttributes": ["format", "name"]}
                     },
                     {
                        "id":"a1a91180-ab88-4c1a-aab5-48c242a218cc",
                        "name":"hoge-action-node-03",
                        "type":"action",
                        "action":"fileWriter",
                        "with": {"format":"csv","output":"ram:///root/output.csv"}
                     }
                  ],
                  "edges":[
                     {
                        "id":"1379a497-9e4e-40fb-8361-d2eeeb491762",
                        "from":"a1a91180-ab88-4c1a-aab5-48c242a218ca",
                        "to":"a1a91180-ab88-4c1a-aab5-48c242a218cb",
                        "fromPort":"default",
                        "toPort":"default"
                     },
                     {
                        "id":"1379a497-9e4e-40fb-8361-d2eeeb491763",
                        "from":"a1a91180-ab88-4c1a-aab5-48c242a218cb",
                        "to":"a1a91180-ab88-4c1a-aab5-48c242a218cc",
                        "fromPort":"default",
                        "toPort":"default"
                     }
                  ]
               }
            ]
          }
  "#;
        let workflow = Workflow::try_from_str(json).unwrap();
        let executor = DagExecutor::new(&workflow).unwrap();
        let res = executor.start().await;
        assert!(res.is_err());
    }
}
