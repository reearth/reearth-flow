use reearth_flow_xml;
use reearth_flow_xml::{
    parser::read_xml,
    traits::{Element, Node},
};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, ActionValue, AsyncAction,
    DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XmlFormatter {
    attribute: String,
}

#[async_trait::async_trait]
#[typetag::serde(name = "XMLFormatter")]
impl AsyncAction for XmlFormatter {
    async fn run(&self, _ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("no inputs"))?;
        let default = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("no default"))?
            .clone();
        let new_default = match default {
            Some(ActionValue::Array(xs)) => Ok(ActionValue::Array(
                xs.into_iter()
                    .map(|x| match x {
                        ActionValue::Map(mut kv) => {
                            if let Some(v) = kv.get_mut(&self.attribute) {
                                if let ActionValue::String(src) = v {
                                    *src =
                                        match {
                                            || {
                                                read_xml(src)?
                                            .first_child()
                                            .ok_or(reearth_flow_xml::error::Error::WrongDocument)?
                                            .to_xml()
                                            }
                                        }() {
                                            Ok(x) => x,
                                            Err(_) => src.to_string(),
                                        }
                                }
                            };
                            ActionValue::Map(kv.clone())
                        }
                        x => x.clone(),
                    })
                    .collect(),
            )),
            None => Err(Error::input("no default value")),
            _ => Err(Error::input("inputs must be Array of Map")),
        }?;
        let output = vec![(DEFAULT_PORT.clone(), Some(new_default))]
            .into_iter()
            .collect();
        Ok(output)
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[allow(dead_code)]
    const SRC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
                <breakfast_menu>
                <food>
                <name>Belgian Waffles</name>
                <price>$5.95</price>
                <description>Two of our famous Belgian Waffles with plenty of real maple syrup</description>
                <calories>650</calories>
                </food><food><name>Strawberry Belgian Waffles</name>
                <price>$7.95</price>
                    <description>Light Belgian waffles covered with strawberries and whipped cream</description>
                        <calories>900</calories>
                </food>
                <food><name>Berry-Berry Belgian Waffles</name>
                <price>$8.95</price><description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description>
                <calories>900</calories>
                </food>
                <food>
                <name>French Toast</name>
                <price>$4.50</price>
                <description>Thick slices made from our homemade sourdough bread</description>
                <calories>600</calories>
                </food>
                <food>
                <name>Homestyle Breakfast</name>
                <price>$6.95</price>
                <description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description>
                <calories>950</calories>
                </food>
                </breakfast_menu>
                "#;

    #[allow(dead_code)]
    const EXPECTED: &str = "<breakfast_menu>\
<food>\
<name>Belgian Waffles</name>\
<price>$5.95</price>\
<description>Two of our famous Belgian Waffles with plenty of real maple syrup</description>\
<calories>650</calories>\
</food>\
<food>\
<name>Strawberry Belgian Waffles</name>\
<price>$7.95</price>\
<description>Light Belgian waffles covered with strawberries and whipped cream</description>\
<calories>900</calories>\
</food>\
<food>\
<name>Berry-Berry Belgian Waffles</name>\
<price>$8.95</price>\
<description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description>\
<calories>900</calories>\
</food>\
<food>\
<name>French Toast</name>\
<price>$4.50</price>\
<description>Thick slices made from our homemade sourdough bread</description>\
<calories>600</calories>\
</food>\
<food>\
<name>Homestyle Breakfast</name>\
<price>$6.95</price>\
<description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description>\
<calories>950</calories>\
</food>\
</breakfast_menu>";

    #[tokio::test]
    async fn test_format_xml() {
        let inputs = Some(
            vec![(
                DEFAULT_PORT.clone(),
                Some(ActionValue::Array(vec![ActionValue::Map(
                    vec![(
                        "field1".to_owned(),
                        ActionValue::String(SRC.to_string().to_owned()),
                    )]
                    .into_iter()
                    .collect(),
                )])),
            )]
            .into_iter()
            .collect::<ActionDataframe>(),
        );
        let xml_formatter = XmlFormatter {
            attribute: "field1".to_string(),
        };
        let ctx = ActionContext::default();
        let result = xml_formatter.run(ctx, inputs).await.unwrap();
        let default = result.get(&DEFAULT_PORT).unwrap().as_ref().unwrap();
        match default {
            ActionValue::Array(xs) => match &xs[0] {
                ActionValue::Map(kv) => {
                    if let ActionValue::String(actual) = kv.get("field1").unwrap() {
                        println!("{}", actual);
                        assert_eq!(actual, EXPECTED)
                    } else {
                        panic!("field1 must be String")
                    }
                }
                _ => panic!("Array must include Map"),
            },
            _ => panic!("output must be Array of Map"),
        };
    }
}
