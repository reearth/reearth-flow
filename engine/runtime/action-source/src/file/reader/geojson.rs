use std::sync::Arc;

use geojson::FeatureCollection;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;
use tokio::sync::mpsc::Sender;

pub(crate) async fn read_geojson(
    input_path: Uri,
    storage_resolver: Arc<StorageResolver>,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{:?}", e)))?;
    let result = storage
        .get(input_path.path().as_path())
        .await
        .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{:?}", e)))?;
    let byte = result
        .bytes()
        .await
        .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{:?}", e)))?;
    let text = String::from_utf8(byte.to_vec())
        .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{:?}", e)))?;
    let value: FeatureCollection = serde_json::from_str(&text)
        .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{:?}", e)))?;
    for feature in value.features {
        let feature: Feature = feature
            .try_into()
            .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{:?}", e)))?;
        sender
            .send((
                DEFAULT_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{:?}", e)))?;
    }
    Ok(())
}
