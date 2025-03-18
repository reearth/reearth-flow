use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_types::{AttributeValue, Feature};
use tokio::sync::mpsc::Sender;

pub(crate) async fn read_json(
    content: &Bytes,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let text = String::from_utf8(content.to_vec())
        .map_err(|e| crate::errors::SourceError::JsonFileReader(format!("{:?}", e)))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| crate::errors::SourceError::JsonFileReader(format!("{:?}", e)))?;
    let features: AttributeValue = value.into();
    match features {
        AttributeValue::Array(features) => {
            for feature in features {
                let AttributeValue::Map(attributes) = feature else {
                    continue;
                };
                let feature = Feature::from(
                    attributes
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<IndexMap<_, _>>(),
                );
                sender
                    .send((
                        DEFAULT_PORT.clone(),
                        IngestionMessage::OperationEvent { feature },
                    ))
                    .await
                    .map_err(|e| crate::errors::SourceError::JsonFileReader(format!("{:?}", e)))?;
            }
        }
        AttributeValue::Map(_) => {
            let feature = Feature::from(features);
            sender
                .send((
                    DEFAULT_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| crate::errors::SourceError::JsonFileReader(format!("{:?}", e)))?;
        }
        _ => Err(crate::errors::SourceError::JsonFileReader(
            "Invalid JSON format".to_string(),
        ))?,
    }
    Ok(())
}
