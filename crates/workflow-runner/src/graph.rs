use std::collections::HashMap;

use anyhow::Result;
use petgraph::graph::Graph;

use reearth_flow_workflow::error::Error;
use reearth_flow_workflow::graph::Node;
use reearth_flow_workflow::id::Id;
use reearth_flow_workflow::workflow::{Property, Workflow};

type Graphs = HashMap<Id, Graph<Node, EdgeProperty>>;

#[derive(Debug, Clone)]
pub struct EdgeProperty {
    pub from_output: String,
    pub to_input: String,
}

#[derive(Debug)]
pub struct ExecuteGraph {
    pub id: Id,
    pub name: String,
    pub with: Property,
    pub entry_graph: Graph<Node, EdgeProperty>,
    pub sub_graphs: Graphs,
}

impl ExecuteGraph {
    pub fn new(workflow: &Workflow) -> Result<Self> {
        let entry_graph = workflow
            .graphs
            .iter()
            .filter(|&graph| graph.id == workflow.entry_graph_id)
            .map(create_graph)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .next();
        let entry_graph = entry_graph.ok_or(Error::WorkflowConfigError(format!(
            "Failed to init entry graph with {}",
            workflow.entry_graph_id
        )))?;
        let sub_graphs = workflow
            .graphs
            .iter()
            .filter(|&graph| graph.id != workflow.entry_graph_id)
            .map(|graph| {
                let g = create_graph(graph)?;
                Ok((graph.id, g))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        Ok(Self {
            id: workflow.id,
            name: workflow.name.clone(),
            with: workflow.with.clone(),
            entry_graph,
            sub_graphs,
        })
    }
}

fn create_graph(graph: &reearth_flow_workflow::graph::Graph) -> Result<Graph<Node, EdgeProperty>> {
    let mut g = Graph::<Node, EdgeProperty>::new();
    let nodes = graph
        .nodes
        .iter()
        .map(|node| {
            let index = g.add_node(node.clone());
            (node.id(), index)
        })
        .collect::<HashMap<_, _>>();
    for edge in graph.edges.iter() {
        let from = *nodes
            .get(&edge.from)
            .ok_or(Error::WorkflowConfigError(format!(
                "Failed to get from node with edge = {:?}",
                edge
            )))?;
        let to = *nodes
            .get(&edge.to)
            .ok_or(Error::WorkflowConfigError(format!(
                "Failed to get to nodes with edge = {:?}",
                edge
            )))?;
        g.add_edge(
            from,
            to,
            EdgeProperty {
                from_output: edge.from_output.clone(),
                to_input: edge.to_input.clone(),
            },
        );
    }
    Ok(g)
}

mod tests {
    #[allow(unused_imports)]
    use petgraph::visit::IntoNodeIdentifiers;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_new() {
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
                        "name":"hoge-action-node",
                        "type":"action",
                        "action":"featureReader",
                        "with": {"format":"csv","dataset":"file:///hoge/fuga.csv"}
                     },
                     {
                        "id":"1efa785f-6550-4a54-9983-537a3d4bf341",
                        "name":"hoge-graph-node",
                        "type":"subGraph",
                        "subGraphId":"c6863b71-953b-4d15-af56-396fc93fc617",
                        "with": {}
                     }
                  ],
                  "edges":[
                     {
                        "id":"1379a497-9e4e-40fb-8361-d2eeeb491762",
                        "from":"a1a91180-ab88-4c1a-aab5-48c242a218ca",
                        "to":"1efa785f-6550-4a54-9983-537a3d4bf341",
                        "fromOutput":"default",
                        "toInput":"default"
                     }
                  ]
               },
               {
                  "id":"c6863b71-953b-4d15-af56-396fc93fc620",
                  "name":"sub-graph",
                  "nodes":[
                     {
                        "id":"05a17b1c-40d0-433d-8d17-f47ca49e5e9b",
                        "name":"hoge-action-node-01",
                        "type":"action",
                        "action":"featureReader",
                        "with": {"format":"csv","dataset":"file:///hoge/fuga.csv"}
                     },
                     {
                        "id":"06cee130-5828-412f-b467-17d58942e74d",
                        "name":"hoge-action-node-02",
                        "type":"action",
                        "action":"featureReader",
                        "with": {"format":"csv","dataset":"file:///hoge/fuga.csv"}
                     }
                  ],
                  "edges":[
                     {
                        "id":"1fc55186-2156-4283-bee5-fc86a90923ae",
                        "from":"05a17b1c-40d0-433d-8d17-f47ca49e5e9b",
                        "to":"06cee130-5828-412f-b467-17d58942e74d",
                        "fromOutput":"output_01",
                        "toInput":"input_01"
                    }
                  ]
               }
            ]
          }
  "#;
        let workflow: Workflow = serde_json::from_str(json).unwrap();
        let graph = ExecuteGraph::new(&workflow).unwrap();
        assert_eq!(graph.id, workflow.id);
        assert_eq!(graph.name, workflow.name);
        assert_eq!(graph.entry_graph.node_count(), 2);
        assert_eq!(graph.sub_graphs.len(), 1);
    }
}
