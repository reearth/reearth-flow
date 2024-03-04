use std::collections::HashMap;

use reearth_flow_eval_expr::scope::Scope;

use crate::{ActionDataframe, ActionValue};

pub fn inject_variables_to_scope(inputs: &ActionDataframe, scope: &Scope) -> anyhow::Result<()> {
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
