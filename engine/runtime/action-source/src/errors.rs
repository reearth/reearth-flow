use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("File Reader Factory error: {0}")]
    FileReaderFactory(String),
    #[error("File Reader error: {0}")]
    FileReader(String),
    #[error("CityGmlFileReader error: {0}")]
    CityGmlFileReader(String),
    #[error("CsvFileReader error: {0}")]
    CsvFileReader(String),
    #[error("JsonFileReader error: {0}")]
    JsonFileReader(String),
    #[error("GeoJsonFileReader error: {0}")]
    GeoJsonFileReader(String),
    #[error("ShapefileReader error: {0}")]
    ShapefileReader(String),
    #[error("CzmlReader error: {0}")]
    CzmlReader(String),
    #[error("GeoPackageReader error: {0}")]
    GeoPackageReader(String),
    #[error("GltfReader error: {0}")]
    GltfReader(String),
    #[error("File Path Extractor Factory error: {0}")]
    FilePathExtractorFactory(String),
    #[error("File Path Extractor error: {0}")]
    FilePathExtractor(String),
    #[error("FeatureCreator Factory error: {0}")]
    FeatureCreatorFactory(String),
    #[error("FeatureCreator error: {0}")]
    FeatureCreator(String),
    #[error("SqlReader Factory error: {0}")]
    SqlReaderFactory(String),
    #[error("SqlReader error: {0}")]
    SqlReader(String),
}

pub type Result<T, E = SourceError> = std::result::Result<T, E>;

impl SourceError {
    pub(crate) fn file_path_extractor<T: ToString>(message: T) -> Self {
        Self::FilePathExtractor(message.to_string())
    }

    pub(crate) fn sql_reader<T: ToString>(message: T) -> Self {
        Self::SqlReader(message.to_string())
    }
}
