use bytes::Bytes;
use geojson::FeatureCollection;
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_types::Feature;
use tokio::sync::mpsc::Sender;

pub(crate) async fn read_geojson(
    content: &Bytes,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let text = String::from_utf8(content.to_vec())
        .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{e:?}")))?;
    let value: FeatureCollection = serde_json::from_str(&text)
        .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{e:?}")))?;
    for feature in value.features {
        let feature: Feature = feature
            .try_into()
            .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{e:?}")))?;
        sender
            .send((
                DEFAULT_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| crate::errors::SourceError::GeoJsonFileReader(format!("{e:?}")))?;
    }
    Ok(())
}
