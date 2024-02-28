macro_rules! property_schema(
    ($name:ident) => (
        impl TryFrom<reearth_flow_workflow::graph::NodeProperty> for $name {
            type Error = anyhow::Error;

            fn try_from(node_property: reearth_flow_workflow::graph::NodeProperty) -> Result<Self, anyhow::Error> {
                serde_json::from_value(Value::Object(node_property)).map_err(|e| {
                    anyhow!(
                        "Failed to convert NodeProperty to PropertySchema with {}",
                        e
                    )
                })
            }
        }
    )
);
