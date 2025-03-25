use std::sync::Arc;

use bytes::Bytes;
use quick_xml::events::{BytesDecl, BytesStart, Event};
use quick_xml::writer::Writer;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;

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
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{:?}", e)))?;
    writer.write_event(Event::End(end))?;

    let result = writer.into_inner();
    let xml = String::from_utf8(result)
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{:?}", e)))?;
    let storage = storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{:?}", e)))?;
    storage
        .put_sync(output.path().as_path(), Bytes::from(xml))
        .map_err(|e| crate::errors::SinkError::XmlWriter(format!("{:?}", e)))?;
    Ok(())
}
