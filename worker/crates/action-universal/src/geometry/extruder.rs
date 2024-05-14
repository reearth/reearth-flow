use std::sync::Arc;

use reearth_flow_action::geometry::{Geometry, GeometryValue};
use reearth_flow_geometry::types::geometry::Geometry as FlowGeometry;
use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe, Feature, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Extruder {
    distance: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "Extruder")]
impl AsyncAction for Extruder {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let expr = &self.distance;
        let template_ast = expr_engine
            .compile(expr.as_str())
            .map_err(Error::internal_runtime)?;
        let result = input
            .features
            .iter()
            .flat_map(|feature| {
                ctx.action_log(format!("Processing feature: {}", feature.id));
                let scope = expr_engine.new_scope();
                for (k, v) in &feature.attributes {
                    scope.set(k.inner().as_str(), v.clone().into());
                }
                let Ok(height) = scope.eval_ast::<f64>(&template_ast) else {
                    return Some(feature.clone());
                };
                let Some(geometry) = &feature.geometry else {
                    return Some(feature.clone());
                };
                let geometry = geometry.clone();
                let GeometryValue::FlowGeometry(flow_geometry) = &geometry.value else {
                    return Some(feature.clone());
                };
                let FlowGeometry::Polygon(polygon) = flow_geometry else {
                    return Some(feature.clone());
                };
                let solid = polygon.extrude(height);
                let geometry = Geometry {
                    value: GeometryValue::FlowGeometry(FlowGeometry::Solid(solid)),
                    ..geometry
                };
                let mut feature = feature.clone();
                feature.geometry = Some(geometry);
                Some(feature)
            })
            .collect::<Vec<Feature>>();
        Ok(ActionDataframe::from([(
            DEFAULT_PORT.to_owned(),
            Dataframe::new(result),
        )]))
    }
}
