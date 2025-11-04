use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use quick_xml::events::{BytesDecl, BytesStart, Event};
use quick_xml::writer::Writer;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;
use reearth_flow_storage::resolve::StorageResolver;

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
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
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
        let params = if let Some(with) = with {
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
        let sink = XmlWriter {
            params,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct XmlWriter {
    pub(super) params: XmlWriterParam,
    pub(super) buffer: HashMap<Uri, Vec<Feature>>,
}

/// # XmlWriter Parameters
///
/// Configuration for writing features to XML files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct XmlWriterParam {
    /// # Output Path
    /// Output path or expression for the XML file to create
    pub(super) output: Expr,
}

impl Sink for XmlWriter {
    fn name(&self) -> &str {
        "XmlWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();
        let output = &self.params.output;
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let uri = Uri::from_str(&path)?;
        self.buffer.entry(uri).or_default().push(ctx.feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        for (uri, features) in &self.buffer {
            write_xml(uri, features, &storage_resolver)?;
        }
        Ok(())
    }
}

pub(super) fn write_xml(
    output: &Uri,
    features: &[Feature],
    storage_resolver: &Arc<StorageResolver>,
) -> Result<(), crate::errors::SinkError> {
    let attributes = features
        .iter()
        .map(|f| {
            serde_json::Value::Object(
                f.attributes
                    .clone()
                    .into_iter()
                    .map(|(k, v)| (k.into_inner().to_string(), v.into()))
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
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{e:?}")))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(xml))
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{e:?}")))?;
    Ok(())
}
