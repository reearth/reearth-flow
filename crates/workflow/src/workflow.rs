use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::graph::Graph;
use crate::id::Id;

pub type Parameter = Map<String, Value>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub id: Id,
    pub name: String,
    pub entry_graph_id: Id,
    pub with: Parameter,
    pub graphs: Vec<Graph>,
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_parse() {
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
                        "fromPort":"default",
                        "toPort":"default"
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
                        "fromPort":"output_01",
                        "toPort":"input_01"
                    }
                  ]
               }
            ]
          }
  "#;

        let workflow: Workflow = serde_json::from_str(json).unwrap();
        assert_eq!(
            workflow.id.to_string(),
            "7b66c0a4-e1fa-41dd-a0c9-df3f6e01cc22"
        );
        assert_eq!(workflow.name, "hoge-workflow");
        assert_eq!(
            workflow.entry_graph_id.to_string(),
            "c6863b71-953b-4d15-af56-396fc93fc617"
        );
        assert_eq!(workflow.with.len(), 2);
        assert_eq!(workflow.graphs.len(), 2);
        assert_eq!(
            workflow.graphs[0].id.to_string(),
            "c6863b71-953b-4d15-af56-396fc93fc617"
        );
        assert_eq!(workflow.graphs[0].name, "hoge-graph");
        assert_eq!(workflow.graphs[0].nodes.len(), 2);
    }
}
