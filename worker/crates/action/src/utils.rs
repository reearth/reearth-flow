pub mod dir;
pub mod zip;

use std::collections::HashMap;

use reearth_flow_eval_expr::scope::Scope;

use crate::{ActionDataframe, ActionValue};

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
            scope.set(
                key.clone().into_inner().as_str(),
                inputs.get(key).unwrap().clone().into(),
            );
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
        .map(|key| {
            (
                key.clone().into_inner(),
                inputs.get(key).unwrap().clone().unwrap(),
            )
        })
        .collect::<HashMap<String, ActionValue>>()
}
