use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub(super) enum FeatureProcessorError {
    #[error("Feature Merger Factory error: {0}")]
    MergerFactory(String),
    #[error("Feature Merger error: {0}")]
    Merger(String),
    #[error("Feature Sorter Factory error: {0}")]
    SorterFactory(String),
    #[error("Feature Sorter error: {0}")]
    Sorter(String),
    #[error("Feature Filter Factory error: {0}")]
    FilterFactory(String),
    #[error("Feature Filter error: {0}")]
    Filter(String),
    #[error("Feature Transformer Factory error: {0}")]
    TransformerFactory(String),
    #[error("Feature Transformer error: {0}")]
    Transformer(String),
    #[error("Feature Counter Factory error: {0}")]
    CounterFactory(String),
    #[error("Feature Counter error: {0}")]
    Counter(String),
    #[error("Feature File City Gml Reader Factory error: {0}")]
    FileCityGmlReaderFactory(String),
    #[error("Feature File City Gml Reader error: {0}")]
    FileCityGmlReader(String),
    #[error("Feature File Reader Factory error: {0}")]
    FileReaderFactory(String),
    #[error("Feature File Csv Reader error: {0}")]
    FileCsvReader(String),
    #[error("Feature File Json Reader error: {0}")]
    FileJsonReader(String),
    #[error("RhaiCallerFactory error: {0}")]
    RhaiCallerFactory(String),
    #[error("RhaiCaller error: {0}")]
    RhaiCaller(String),
    #[error("FeatureFilePathExtractorFactory error: {0}")]
    FilePathExtractorFactory(String),
    #[error("FeatureFilePathExtractor error: {0}")]
    FilePathExtractor(String),
    #[error("LodFilterFactory error: {0}")]
    LodFilterFactory(String),
    #[error("LodFilter error: {0}")]
    LodFilter(String),
    #[error("DuplicateFilterFactory error: {0}")]
    DuplicateFilterFactory(String),
    #[error("DuplicateFilter error: {0}")]
    DuplicateFilter(String),
    #[error("FeatureWriterFactory error: {0}")]
    FeatureWriterFactory(String),
    #[error("FeatureWriter error: {0}")]
    FeatureWriter(String),
}

pub(super) type Result<T, E = FeatureProcessorError> = std::result::Result<T, E>;
