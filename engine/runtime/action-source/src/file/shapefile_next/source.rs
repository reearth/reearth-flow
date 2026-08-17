//! The Shapefile Reader action.

use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, FEATURES_PORT},
};
use reearth_flow_types::Feature;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use super::archive;
use super::geometry::ShapeConverter;
use super::record;
use crate::{
    errors::{ShapefileError, SourceError},
    file::reader::runner::{get_content, FileReaderCommonParam, FileReaderCompiledParam},
};

#[derive(Debug, Clone, Default)]
pub(crate) struct ShapefileReaderFactory;

impl SourceFactory for ShapefileReaderFactory {
    fn name(&self) -> &str {
        "Shapefile Reader"
    }

    fn description(&self) -> &str {
        "Reads geographic features from Shapefile archives (.zip containing .shp, .dbf, .shx files)."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ShapefileReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Input"]
    }

    fn tags(&self) -> &[&'static str] {
        &["shapefile", "vector"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let params: ShapefileReaderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::ShapefileReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::ShapefileReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::ShapefileReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let compiled_params = ShapefileReaderCompiledParam {
            common: params.common_property.compile(&ctx).map_err(|e| {
                SourceError::ShapefileReaderFactory(format!("Failed to compile params: {e:?}"))
            })?,
            encoding: params.encoding,
            force_2d: params.force_2d,
            allow_empty_path: params.allow_empty_path,
        };
        Ok(Box::new(ShapefileReader {
            params: compiled_params,
        }))
    }
}

#[derive(Debug, Clone)]
struct ShapefileReaderCompiledParam {
    common: FileReaderCompiledParam,
    encoding: Option<String>,
    force_2d: bool,
    allow_empty_path: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ShapefileReader {
    params: ShapefileReaderCompiledParam,
}

/// # ShapefileReader Parameters
///
/// Configuration for reading Shapefile archives as geographic features.
/// Expects a ZIP archive containing the required Shapefile components (.shp, .dbf, .shx).
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct ShapefileReaderParam {
    #[serde(flatten)]
    pub(super) common_property: FileReaderCommonParam,
    /// # Character Encoding
    /// Character encoding for attribute data in the DBF file, such as "UTF-8", "Shift-JIS", or "Windows-1252"; labels are case-insensitive. When omitted, the encoding is taken from the .cpg file if present, otherwise UTF-8 (UTF-16 is not supported).
    pub(super) encoding: Option<String>,
    /// # Force 2D
    /// If true, forces all geometries to be 2D (ignoring Z values).
    #[serde(default, rename = "force2D", alias = "force2d")]
    pub(super) force_2d: bool,
    /// # Allow Null Path
    /// If true, a null dataset path produces zero features instead of an error, allowing optional shapefile inputs.
    #[serde(default, alias = "allowEmptyPath")]
    pub(super) allow_empty_path: bool,
}

#[async_trait::async_trait]
impl Source for ShapefileReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "Shapefile Reader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);

        // When allow_empty_path is set and the dataset resolved to null at build time,
        // treat as "no input" and skip silently.
        if self.params.allow_empty_path
            && self.params.common.dataset.is_none()
            && self.params.common.inline.is_none()
        {
            return Ok(());
        }

        let content = get_content(&self.params.common, storage_resolver).await?;
        read_shapefile(&content, &self.params, sender)
            .await
            .map_err(Into::<BoxedError>::into)
    }
}

/// Read the shapefile in `content`, sending one feature per record.
///
/// `content` must be a ZIP archive holding the shapefile's components; a bare
/// `.shp` carries neither the attribute table nor the CRS.
async fn read_shapefile(
    content: &Bytes,
    params: &ShapefileReaderCompiledParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    if !archive::is_zip(content) {
        return Err(ShapefileError::DirectBytesNotSupported.into());
    }
    let mut archive = archive::open(content, &params.encoding)?;
    let converter = ShapeConverter::new(archive.epsg, params.force_2d);

    // Records are converted and sent one at a time rather than collected first,
    // so the whole table never has to be held at once.
    let mut features = Vec::new();
    for record in archive.records() {
        let (shape, record) = record.map_err(|e| {
            SourceError::shapefile_reader(format!("Failed to read shape and record: {e}"))
        })?;
        let geometry = converter.convert(shape)?;
        features.push(Feature::new_with_attributes_and_geometry(
            record::to_attributes(record),
            geometry,
        ));
    }
    converter.report_discarded_measures();

    for feature in features {
        sender
            .send((
                FEATURES_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| SourceError::shapefile_reader(format!("Failed to send feature: {e}")))?;
    }
    Ok(())
}
