use std::collections::HashMap;

use bytes::Bytes;
use quick_xml::events::{BytesDecl, BytesStart, Event};
use quick_xml::writer::Writer;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
use reearth_flow_types::{Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use crate::SinkOutput;

#[derive(Debug, Clone, Default)]
pub(crate) struct XmlWriterFactory;

impl SinkFactory for XmlWriterFactory {
    fn name(&self) -> &str {
        "XmlWriter"
    }

    fn description(&self) -> &str {
        "Writes features to XML files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(XmlWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["xml"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: XmlWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::XmlWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::XmlWriterFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(SinkError::XmlWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let output = params.output.compile().map_err(|e| {
            SinkError::XmlWriterFactory(format!("Failed to compile `output`: {e:?}"))
        })?;
        let sink = XmlWriter {
            output,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct XmlWriter {
    output: CompiledCode,
    pub(super) buffer: HashMap<String, (SinkOutput, Vec<Feature>)>,
}

/// # XmlWriter Parameters
///
/// Configuration for writing features to XML files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct XmlWriterParam {
    /// Output path or expression for the XML file to create
    pub(super) output: Code,
}

impl Sink for XmlWriter {
    fn name(&self) -> &str {
        "XmlWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let path = self
            .output
            .eval_string(&ctx.feature, ctx.env_vars.clone())
            .map_err(|e| SinkError::XmlWriter(format!("{e:?}")))?;
        let feature = ctx.feature.clone();
        let node_ctx: NodeContext = ctx.into();
        use std::collections::hash_map::Entry;
        match self.buffer.entry(path.clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().1.push(feature);
            }
            Entry::Vacant(e) => {
                let out =
                    SinkOutput::new(&node_ctx.sandbox_root, &path, &node_ctx.storage_resolver)
                        .map_err(|e| SinkError::XmlWriter(e.to_string()))?;
                e.insert((out, vec![feature]));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        for (out, features) in self.buffer.values() {
            write_xml(out, features)?;
        }
        Ok(())
    }
}

pub(super) fn write_xml(
    out: &SinkOutput,
    features: &[Feature],
) -> Result<(), crate::errors::SinkError> {
    let attributes = features
        .iter()
        .map(|f| {
            serde_json::Value::Object(
                f.attributes
                    .iter()
                    .map(|(k, v)| (k.clone().into_inner().to_string(), v.clone().into()))
                    .collect::<serde_json::Map<_, _>>(),
            )
        })
        .collect::<Vec<serde_json::Value>>();

    let mut writer = Writer::new(Vec::new());
    writer.write_event(Event::Decl(BytesDecl::new("1.2", None, None)))?;
    let start = BytesStart::new("features");
    let end = start.to_end();
    writer.write_event(Event::Start(start.clone()))?;
    attributes
        .iter()
        .try_for_each(|attribute| writer.write_serializable("feature", attribute))
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{e:?}")))?;
    writer.write_event(Event::End(end))?;

    let result = writer.into_inner();
    let xml = String::from_utf8(result)
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{e:?}")))?;
    out.write(Bytes::from(xml))
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{e:?}")))?;
    Ok(())
}
