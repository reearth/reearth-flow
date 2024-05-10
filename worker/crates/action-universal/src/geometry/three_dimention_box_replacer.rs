use reearth_flow_action::geometry::{Geometry, GeometryValue};
use reearth_flow_geometry::types::geometry::Geometry as FlowGeometry;
use reearth_flow_geometry::types::point::Point;
use reearth_flow_geometry::types::rectangle::Rectangle;
use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, Attribute, AttributeValue,
    Dataframe, Feature, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThreeDimentionBoxReplacer {
    min_x: Attribute,
    min_y: Attribute,
    min_z: Attribute,
    max_x: Attribute,
    max_y: Attribute,
    max_z: Attribute,
}

#[async_trait::async_trait]
#[typetag::serde(name = "3DBoxReplacer")]
impl AsyncAction for ThreeDimentionBoxReplacer {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;
        let result = input
            .features
            .iter()
            .flat_map(|feature| {
                ctx.action_log(format!("Processing  feature: {}", feature.id));
                let attributes = &feature.attributes;
                let min_x = parse_f64(attributes.get(&self.min_x))?;
                let min_y = parse_f64(attributes.get(&self.min_y))?;
                let min_z = parse_f64(attributes.get(&self.min_z))?;
                let max_x = parse_f64(attributes.get(&self.max_x))?;
                let max_y = parse_f64(attributes.get(&self.max_y))?;
                let max_z = parse_f64(attributes.get(&self.max_z))?;
                let min = Point::new_(min_x, min_y, min_z);
                let max = Point::new_(max_x, max_y, max_z);
                let rectangle = Rectangle::new(min, max);
                let geometry = Geometry::with_value(GeometryValue::FlowGeometry(
                    FlowGeometry::Rectangle(rectangle),
                ));
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

fn parse_f64(value: Option<&AttributeValue>) -> Option<f64> {
    if let Some(AttributeValue::Number(min_x)) = value {
        min_x.as_f64()
    } else {
        None
    }
}
