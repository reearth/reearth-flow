use thiserror::Error;

#[derive(Error, Debug)]
pub enum SinkError {
    #[error("Build factory error: {0}")]
    BuildFactory(String),
    #[error("File Writer error: {0}")]
    FileWriter(String),
    #[error("Cesium3DTiles Writer Factory error: {0}")]
    Cesium3DTilesWriterFactory(String),
    #[error("Cesium3DTiles Writer error: {0}")]
    Cesium3DTilesWriter(String),
    #[error("GeoJson Writer Factory error: {0}")]
    GeoJsonWriterFactory(String),
    #[error("GeoJson Writer error: {0}")]
    GeoJsonWriter(String),
    #[error("Mvt Writer error: {0}")]
    MvtWriter(String),
}

impl SinkError {
    pub fn file_writer<T: ToString>(message: T) -> Self {
        Self::FileWriter(message.to_string())
    }

    pub fn cesium3dtiles_writer<T: ToString>(message: T) -> Self {
        Self::Cesium3DTilesWriter(message.to_string())
    }
}

pub type Result<T, E = SinkError> = std::result::Result<T, E>;
