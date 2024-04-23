use std::sync::Arc;

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe, Feature,
};
use reearth_flow_common::collection;
use reearth_flow_eval_expr::engine::Engine;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributeManager {
    operations: Vec<Operation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct Operation {
    pub(super) attribute: String,
    pub(super) method: Method,
    pub(super) value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) enum Method {
    #[serde(rename = "convert")]
    Convert,
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "rename")]
    Rename,
    #[serde(rename = "remove")]
    Remove,
}

#[derive(Debug, Clone)]
pub(super) enum Operate {
    Convert {
        expr: Option<rhai::AST>,
        attribute: String,
    },
    Create {
        expr: Option<rhai::AST>,
        attribute: String,
    },
    Rename {
        new_key: String,
        attribute: String,
    },
    Remove {
        attribute: String,
    },
}

#[async_trait::async_trait]
#[typetag::serde(name = "AttributeManager")]
impl AsyncAction for AttributeManager {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let operations = convert_single_operation(&self.operations, Arc::clone(&expr_engine));

        let mut output = ActionDataframe::new();
        for (port, data) in inputs {
            let processed_data = collection::map(&data.features, |row| {
                mapper(row, &operations, Arc::clone(&expr_engine))
            });
            output.insert(port, Dataframe::new(processed_data));
        }
        Ok(output)
    }
}

fn mapper(row: &Feature, operations: &[Operate], expr_engine: Arc<Engine>) -> Feature {
    let mut result = row.clone();
    for operation in operations {
        match operation {
            Operate::Convert { expr, attribute } => {
                let value = row.get(attribute);
                if value.is_none() {
                    continue;
                }
                let scope = expr_engine.new_scope();
                for (k, v) in row.iter() {
                    scope.set(k.inner().as_str(), v.clone().into());
                }
                if let Some(expr) = expr {
                    let new_value = scope.eval_ast::<Dynamic>(expr);
                    if let Ok(new_value) = new_value {
                        if let Ok(new_value) = new_value.try_into() {
                            result.insert(attribute.clone(), new_value);
                        }
                    }
                }
            }
            Operate::Create { expr, attribute } => {
                let scope = expr_engine.new_scope();
                for (k, v) in row.iter() {
                    scope.set(k.inner().as_str(), v.clone().into());
                }
                if let Some(expr) = expr {
                    let new_value = scope.eval_ast::<Dynamic>(expr);
                    if let Ok(new_value) = new_value {
                        if let Ok(new_value) = new_value.try_into() {
                            result.insert(attribute.clone(), new_value);
                        }
                    }
                }
            }
            Operate::Rename { new_key, attribute } => {
                let value = row.get(attribute);
                if value.is_none() {
                    continue;
                }
                result.remove(attribute);
                result.insert(new_key.clone(), value.unwrap().clone());
            }
            Operate::Remove { attribute } => {
                let value = row.get(attribute);
                if value.is_none() {
                    continue;
                }
                result.remove(attribute);
            }
        };
    }
    result
}

fn convert_single_operation(operations: &[Operation], expr_engine: Arc<Engine>) -> Vec<Operate> {
    operations
        .iter()
        .map(|operation| {
            let method = &operation.method;
            let attribute = &operation.attribute;
            let value = operation.value.clone().unwrap_or_default();
            match method {
                Method::Convert => Operate::Convert {
                    expr: expr_engine.compile(&value).ok(),
                    attribute: attribute.clone(),
                },
                Method::Create => Operate::Create {
                    expr: expr_engine.compile(&value).ok(),
                    attribute: attribute.clone(),
                },
                Method::Rename => Operate::Rename {
                    new_key: value,
                    attribute: attribute.clone(),
                },
                Method::Remove => Operate::Remove {
                    attribute: attribute.clone(),
                },
            }
        })
        .collect::<Vec<_>>()
}
