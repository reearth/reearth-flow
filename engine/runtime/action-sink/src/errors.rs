use thiserror::Error;

#[derive(Error, Debug)]
pub enum SinkError {
    #[error("Build factory error: {0}")]
    BuildFactory(String),
    #[error("Csv Writer error: {0}")]
    CsvWriter(String),
    #[error("Csv Writer Factory error: {0}")]
    CsvWriterFactory(String),
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
    #[error("Json Writer Factory error: {0}")]
    JsonWriterFactory(String),
    #[error("Json Writer error: {0}")]
    JsonWriter(String),
    #[error("Mvt Writer Factory error: {0}")]
    MvtWriterFactory(String),
    #[error("Mvt Writer error: {0}")]
    MvtWriter(String),
    #[error("Gltf Writer Factory error: {0}")]
    GltfWriterFactory(String),
    #[error("Gltf Writer error: {0}")]
    GltfWriter(String),
    #[error("Czml Writer Factory error: {0}")]
    CzmlWriterFactory(String),
    #[error("Czml Writer error: {0}")]
    CzmlWriter(String),
    #[error("Shapefile Writer Factory error: {0}")]
    ShapefileWriterFactory(String),
    #[error("Shapefile Writer error: {0}")]
    ShapefileWriter(String),
    #[error("Shapefile I/O error: {0}")]
    ShapefileWriterIo(#[from] std::io::Error),
    #[error("Obj Writer Factory error: {0}")]
    ObjWriterFactory(String),
    #[error("Obj Writer error: {0}")]
    ObjWriter(String),
    #[error("Xml Writer error: {0}")]
    XmlWriterFactory(String),
    #[error("Xml Writer Factory error: {0}")]
    XmlWriter(String),
    #[error("ZipFile Writer Factory error: {0}")]
    ZipFileWriterFactory(String),
    #[error("ZipFile Writer error: {0}")]
    ZipFileWriter(String),
    #[error("Excel Writer Factory error: {0}")]
    ExcelWriterFactory(String),
    #[error("Excel Writer error: {0}")]
    ExcelWriter(String),
}

impl SinkError {
    pub fn file_writer<T: ToString>(message: T) -> Self {
        Self::FileWriter(message.to_string())
    }

    pub fn geojson_writer<T: ToString>(message: T) -> Self {
        Self::FileWriter(message.to_string())
    }

    pub fn cesium3dtiles_writer<T: ToString>(message: T) -> Self {
        Self::Cesium3DTilesWriter(message.to_string())
    }

    pub fn gltf_writer<T: ToString>(message: T) -> Self {
        Self::GltfWriter(message.to_string())
    }

    pub fn czml_writer<T: ToString>(message: T) -> Self {
        Self::CzmlWriter(message.to_string())
    }

    pub fn obj_writer<T: ToString>(message: T) -> Self {
        Self::ObjWriter(message.to_string())
    }
}

pub type Result<T, E = SinkError> = std::result::Result<T, E>;
