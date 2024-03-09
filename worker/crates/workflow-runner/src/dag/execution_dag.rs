use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use async_recursion::async_recursion;
use petgraph::graph::NodeIndex;
use reearth_flow_workflow::graph::NodeAction;
use tokio::task::JoinSet;
use tracing::{info, info_span};

use reearth_flow_action::{ActionContext, ActionDataframe};
use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_workflow::{graph::Node, id::Id, workflow::Workflow};

use super::dag_impl::Dag;
use crate::action_runner::ActionRunner;
use crate::Error;

type Graphs = HashMap<Id, Dag>;

pub struct DagExecutor {
    job_id: Id,
    workflow_id: Id,
    workflow_name: String,
    entry_dag: Dag,
    sub_dags: Graphs,
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    dataframe_state: Arc<State>,
    logger_factory: Arc<LoggerFactory>,
    root_span: tracing::Span,
}

impl DagExecutor {
    pub fn new(
        job_id: Id,
        workflow: &Workflow,
        storage_resolver: Arc<StorageResolver>,
        dataframe_state: Arc<State>,
        logger_factory: Arc<LoggerFactory>,
    ) -> crate::Result<Self> {
        let entry_graph = workflow
            .graphs
            .iter()
            .filter(|&graph| graph.id == workflow.entry_graph_id)
            .map(Dag::from_graph)
            .collect::<crate::Result<Vec<_>>>()?
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
            .collect::<crate::Result<HashMap<_, _>>>()?;
        let engine = Engine::new();
        workflow.with.iter().for_each(|(k, v)| {
            engine.set_scope_var(k, v);
        });
        let root_span = info_span!(
            "root",
            "otel.name" = workflow.name.as_str(),
            "otel.kind" = "workflow",
            "workflow.id" = workflow.id.to_string().as_str(),
            "workflow.job_id" = job_id.to_string().as_str(),
        );
        Ok(Self {
            workflow_id: workflow.id,
            job_id,
            workflow_name: workflow.name.clone(),
            entry_dag,
            sub_dags,
            expr_engine: Arc::new(engine),
            storage_resolver: Arc::clone(&storage_resolver),
            dataframe_state: Arc::clone(&dataframe_state),
            logger_factory: Arc::clone(&logger_factory),
            root_span,
        })
    }

    pub async fn start(&self) -> crate::Result<()> {
        let workflow_name = self.workflow_name.clone();
        info!(parent: &self.root_span, "Start workflow = {:?}", workflow_name);
        let start = Instant::now();
        let _ = self.run_dag(&self.entry_dag).await?;
        let duration = start.elapsed();
        info!(
            parent: &self.root_span,
            "Finish workflow = {:?}, duration = {:?}",
            self.workflow_name, duration
        );
        Ok(())
    }

    #[async_recursion]
    pub async fn run_dag(&self, dag: &Dag) -> crate::Result<ActionDataframe> {
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
                match node {
                    Node::Action { action, .. } => {
                        let node_id = node.id();
                        let ctx = ActionContext::new(
                            self.job_id,
                            self.workflow_id,
                            node_id,
                            node.name().to_owned(),
                            node.with().clone(),
                            Arc::clone(&self.expr_engine),
                            Arc::clone(&self.storage_resolver),
                            self.logger_factory
                                .action_logger(node_id.to_string().as_str()),
                            self.root_span.clone(),
                        );
                        let dataframe_state = Arc::clone(&self.dataframe_state);
                        let action = action.clone();
                        async_tools.spawn(async move {
                            run_async(ix, ctx, action, dataframe_state, input).await
                        });
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
                results.push(res.map_err(crate::Error::execution)??);
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
    action: NodeAction,
    dataframe_state: Arc<State>,
    input: Option<ActionDataframe>,
) -> crate::Result<(NodeIndex, ActionDataframe)> {
    ActionRunner::run(ctx, action, ix, dataframe_state, input).await
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[allow(unused_imports)]
    use super::*;

    use bytes::Bytes;
    use std::str::FromStr;

    use reearth_flow_common::uri::Uri;

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
                        "action":"FileReader",
                        "with": {"format":"csv","dataset":"\"ram:///root/summary.csv\"", "header": true}
                     },
                     {
                        "id":"a1a91180-ab88-4c1a-aab5-48c242a218cb",
                        "name":"hoge-action-node-02",
                        "type":"action",
                        "action":"AttributeKeeper",
                        "with": {"keepAttributes": ["format", "name"]}
                     },
                     {
                        "id":"a1a91180-ab88-4c1a-aab5-48c242a218cc",
                        "name":"hoge-action-node-03",
                        "type":"action",
                        "action":"FileWriter",
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
        let storage_resolver = Arc::new(StorageResolver::new());
        let state =
            Arc::new(State::new(&Uri::for_test("ram:///state/"), &storage_resolver).unwrap());
        let storage = storage_resolver
            .resolve(&Uri::from_str("ram:///root/summary.csv").unwrap())
            .unwrap();
        let bytes = Bytes::from_static(b"format,name,age\njson,alice,20\njson,bob,30");
        storage
            .put(PathBuf::from("/root/summary.csv").as_path(), bytes)
            .await
            .unwrap();

        let workflow = Workflow::try_from_str(json).unwrap();
        let job_id = Id::new_v4();
        let log_factory = Arc::new(LoggerFactory::new(
            reearth_flow_action_log::ActionLogger::root(
                reearth_flow_action_log::Discard,
                reearth_flow_action_log::o!(),
            ),
            PathBuf::new(),
        ));
        let executor =
            DagExecutor::new(job_id, &workflow, storage_resolver, state, log_factory).unwrap();
        let res = executor.start().await;
        assert!(res.is_ok());
    }
}
