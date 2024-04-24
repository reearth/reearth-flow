use reearth_flow_xml::{
    parser::read_xml,
    traits::{Element, Node},
};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AsyncAction, Attribute,
    AttributeValue, Dataframe, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XmlFormatter {
    attribute: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "XMLFormatter")]
impl AsyncAction for XmlFormatter {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let dataframe = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("no default"))?
            .clone();
        let new_default = dataframe
            .features
            .into_iter()
            .map(|mut x| {
                if let Some(AttributeValue::String(src)) =
                    x.attributes.get_mut(&Attribute::new(&self.attribute))
                {
                    *src = match {
                        || {
                            read_xml(src)?
                                .first_child()
                                .ok_or(reearth_flow_xml::error::Error::WrongDocument)?
                                .to_xml()
                        }
                    }() {
                        Ok(x) => x,
                        Err(e) => {
                            ctx.action_log(format!("XML error: {:?}", e));
                            src.to_string()
                        }
                    }
                };
                x
            })
            .collect::<Vec<_>>();
        let output = vec![(DEFAULT_PORT.clone(), Dataframe::new(new_default))]
            .into_iter()
            .collect();
        Ok(output)
    }
}
