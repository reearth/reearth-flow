use std::collections::HashMap;

use reearth_flow_runtime::node::NodeKind;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionSchema {
    pub name: String,
    pub r#type: String,
    pub description: String,
    pub parameter: serde_json::Value,
    pub builtin: bool,
    pub input_ports: Vec<String>,
    pub output_ports: Vec<String>,
    pub categories: Vec<String>,
}

impl ActionSchema {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        r#type: String,
        description: String,
        parameter: serde_json::Value,
        builtin: bool,
        input_ports: Vec<String>,
        output_ports: Vec<String>,
        categories: Vec<String>,
    ) -> Self {
        Self {
            name,
            r#type,
            description,
            parameter,
            builtin,
            input_ports,
            output_ports,
            categories,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct I18nSchema {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) parameter: Option<serde_json::Value>,
}

pub(crate) fn create_action_schema(
    kind: &NodeKind,
    builtin: bool,
    i18n: &HashMap<String, I18nSchema>,
) -> ActionSchema {
    let (name, description, parameter, input_ports, output_ports, categories) = match kind {
        NodeKind::Source(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        if let Some(parameter) =
                            i18n_schema.and_then(|schema| schema.parameter.clone())
                        {
                            reearth_flow_common::json::json_merge_patch(
                                &mut parameter_schema,
                                &parameter,
                            );
                        }
                        parameter_schema
                    }),
                vec![],
                factory
                    .get_output_ports()
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
        NodeKind::Processor(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        if let Some(parameter) =
                            i18n_schema.and_then(|schema| schema.parameter.clone())
                        {
                            reearth_flow_common::json::json_merge_patch(
                                &mut parameter_schema,
                                &parameter,
                            );
                        }
                        parameter_schema
                    }),
                factory
                    .get_input_ports()
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory
                    .get_output_ports()
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
        NodeKind::Sink(factory) => {
            let i18n_schema = i18n.get(&factory.name().to_string());
            (
                factory.name().to_string(),
                i18n_schema
                    .map(|schema| schema.description.clone())
                    .unwrap_or(factory.description().to_string()),
                factory
                    .parameter_schema()
                    .map_or(serde_json::Value::Null, |schema| {
                        let mut parameter_schema: serde_json::Value =
                            serde_json::from_str(serde_json::to_string(&schema).unwrap().as_str())
                                .unwrap();
                        if let Some(parameter) =
                            i18n_schema.and_then(|schema| schema.parameter.clone())
                        {
                            reearth_flow_common::json::json_merge_patch(
                                &mut parameter_schema,
                                &parameter,
                            );
                        }
                        parameter_schema
                    }),
                factory
                    .get_input_ports()
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                vec![],
                factory.categories().iter().map(|c| c.to_string()).collect(),
            )
        }
    };

    ActionSchema::new(
        name,
        match kind {
            NodeKind::Source(_) => "source".to_string(),
            NodeKind::Processor(_) => "processor".to_string(),
            NodeKind::Sink(_) => "sink".to_string(),
        },
        description,
        parameter,
        builtin,
        input_ports,
        output_ports,
        categories,
    )
}
