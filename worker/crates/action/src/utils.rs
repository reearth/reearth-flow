pub mod dir;
pub mod zip;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_eval_expr::scope::Scope;

use crate::{error::Error, ActionDataframe, ActionValue, Result};

pub fn inject_variables_to_scope(inputs: &ActionDataframe, scope: &Scope) -> crate::Result<()> {
    inputs
        .keys()
        .filter(|&key| {
            inputs.get(key).unwrap().is_some()
                && matches!(
                    inputs.get(key).unwrap().clone().unwrap(),
                    ActionValue::Bool(_)
                        | ActionValue::Number(_)
                        | ActionValue::String(_)
                        | ActionValue::Map(_)
                )
        })
        .for_each(|key| {
            scope.set(key, inputs.get(key).unwrap().clone().into());
        });
    Ok(())
}

pub fn convert_dataframe_to_scope_params(inputs: &ActionDataframe) -> HashMap<String, ActionValue> {
    inputs
        .keys()
        .filter(|&key| {
            inputs.get(key).unwrap().is_some()
                && matches!(
                    inputs.get(key).unwrap().clone().unwrap(),
                    ActionValue::Bool(_)
                        | ActionValue::Number(_)
                        | ActionValue::String(_)
                        | ActionValue::Map(_)
                )
        })
        .map(|key| (key.to_owned(), inputs.get(key).unwrap().clone().unwrap()))
        .collect::<HashMap<String, ActionValue>>()
}

pub async fn get_expr_path(
    path: &str,
    inputs: &ActionDataframe,
    expr_engine: Arc<Engine>,
) -> Result<Uri> {
    let scope = expr_engine.new_scope();
    inject_variables_to_scope(inputs, &scope)?;
    let path = expr_engine
        .eval_scope::<String>(path, &scope)
        .map_or_else(|_| path.to_string(), |v| v);
    Uri::from_str(path.as_str()).map_err(Error::input)
}
