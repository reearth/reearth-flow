use std::{collections::HashMap, sync::Arc};

use reearth_flow_action::{Dataframe, Feature};
use reearth_flow_common::color;
use reearth_flow_eval_expr::engine::Engine;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{error::Error, ActionDataframe, ActionResult, Attribute, AttributeValue};

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
    inputs: ActionDataframe,
) -> ActionResult {
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
    let mut output = ActionDataframe::new();
    for (port, data) in inputs {
        let processed_items = data
            .features
            .iter()
            .filter_map(|item| mapper(item, &ast, Arc::clone(&expr_engine)))
            .collect::<Vec<_>>();
        output.insert(port, Dataframe::new(processed_items));
    }
    Ok(output)
}

fn mapper(item: &Feature, ast: &HslAST, expr_engine: Arc<Engine>) -> Option<Feature> {
    let mut processed_item = HashMap::<Attribute, AttributeValue>::new();
    let scope = expr_engine.new_scope();
    for (k, v) in item.attributes.iter() {
        scope.set(k.inner().as_str(), v.clone().into());
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
        Attribute::new("red"),
        AttributeValue::Number((rgba.0 as i64).into()),
    );
    processed_item.insert(
        Attribute::new("green"),
        AttributeValue::Number((rgba.1 as i64).into()),
    );
    processed_item.insert(
        Attribute::new("blue"),
        AttributeValue::Number((rgba.2 as i64).into()),
    );
    processed_item.insert(
        Attribute::new("alpha"),
        AttributeValue::Number((rgba.3 as i64).into()),
    );
    Some(Feature::new_with_attributes(processed_item))
}
