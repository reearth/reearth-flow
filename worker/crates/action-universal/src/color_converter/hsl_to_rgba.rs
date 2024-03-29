use std::{collections::HashMap, sync::Arc};

use reearth_flow_common::color;
use reearth_flow_eval_expr::engine::Engine;
use serde::{Deserialize, Serialize};

use reearth_flow_action::utils::convert_dataframe_to_scope_params;
use reearth_flow_action::{error::Error, ActionDataframe, ActionResult, ActionValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HslPropertySchema {
    hue: String,
    saturation: String,
    lightness: String,
    alpha: String,
}

struct HslAST {
    hue: rhai::AST,
    saturation: rhai::AST,
    lightness: rhai::AST,
    alpha: rhai::AST,
}

pub(super) async fn convert_hsl_to_rgba(
    expr_engine: Arc<Engine>,
    property: &HslPropertySchema,
    inputs: Option<ActionDataframe>,
) -> ActionResult {
    let inputs = inputs.ok_or(Error::input("No Input"))?;
    let ast = HslAST {
        hue: expr_engine
            .compile(property.hue.as_str())
            .map_err(Error::input)?,
        saturation: expr_engine
            .compile(property.saturation.as_str())
            .map_err(Error::input)?,
        lightness: expr_engine
            .compile(property.lightness.as_str())
            .map_err(Error::input)?,
        alpha: expr_engine
            .compile(property.alpha.as_str())
            .map_err(Error::input)?,
    };
    let params = convert_dataframe_to_scope_params(&inputs);

    let mut output = ActionDataframe::new();
    for (port, data) in inputs {
        let data = match data {
            Some(data) => data,
            None => continue,
        };
        let processed_data = match data {
            ActionValue::Array(data) => {
                let processed_items = data
                    .into_iter()
                    .filter_map(|item| mapper(&item, &ast, &params, Arc::clone(&expr_engine)))
                    .collect::<Vec<_>>();
                ActionValue::Array(processed_items)
            }
            ActionValue::Map(data) => {
                let processed_item = mapper(
                    &ActionValue::Map(data),
                    &ast,
                    &params,
                    Arc::clone(&expr_engine),
                );
                processed_item.ok_or(Error::internal_runtime("Failed to convert"))?
            }
            _ => data,
        };
        output.insert(port, Some(processed_data));
    }
    Ok(output)
}

fn mapper(
    item: &ActionValue,
    ast: &HslAST,
    params: &HashMap<String, ActionValue>,
    expr_engine: Arc<Engine>,
) -> Option<ActionValue> {
    match item {
        ActionValue::Map(item) => {
            let mut processed_item = HashMap::<String, ActionValue>::new();
            let scope = expr_engine.new_scope();
            for (k, v) in params {
                scope.set(k, v.clone().into());
            }
            for (k, v) in item {
                scope.set(k, v.clone().into());
            }
            let h = scope.eval_ast::<f64>(&ast.hue);
            let s = scope.eval_ast::<f64>(&ast.saturation);
            let l = scope.eval_ast::<f64>(&ast.lightness);
            let a = scope.eval_ast::<f64>(&ast.alpha);
            if h.is_err() || s.is_err() || l.is_err() || a.is_err() {
                return None;
            }
            let rgba = color::convert_hsl_to_rgba(h.unwrap(), s.unwrap(), l.unwrap(), a.unwrap());
            if rgba.is_err() {
                return None;
            }
            let rgba = rgba.unwrap();
            processed_item.insert(
                "red".to_string(),
                ActionValue::Number((rgba.0 as i64).into()),
            );
            processed_item.insert(
                "green".to_string(),
                ActionValue::Number((rgba.1 as i64).into()),
            );
            processed_item.insert(
                "blue".to_string(),
                ActionValue::Number((rgba.2 as i64).into()),
            );
            processed_item.insert(
                "alpha".to_string(),
                ActionValue::Number((rgba.3 as i64).into()),
            );
            Some(ActionValue::Map(processed_item))
        }
        _ => None,
    }
}
