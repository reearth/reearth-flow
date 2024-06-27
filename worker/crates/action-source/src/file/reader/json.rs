use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Feature};
use tokio::sync::mpsc::Sender;

pub(crate) async fn read_json(
    input_path: Uri,
    storage_resolver: Arc<StorageResolver>,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let result = storage
        .get(input_path.path().as_path())
        .await
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let byte = result
        .bytes()
        .await
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let text = String::from_utf8(byte.to_vec())
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let features: AttributeValue = value.into();
    match features {
        AttributeValue::Array(features) => {
            for feature in features {
                let AttributeValue::Map(feature) = feature else {
                    continue;
                };
                let feature = Feature::from(feature);
                sender
                    .send((
                        DEFAULT_PORT.clone(),
                        IngestionMessage::OperationEvent { feature },
                    ))
                    .await
                    .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
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
                .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
        }
        _ => Err(crate::errors::SourceError::FileReader(
            "Invalid JSON format".to_string(),
        ))?,
    }
    Ok(())
}
