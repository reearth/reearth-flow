use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub(super) enum AttributeProcessorError {
    #[error("Attribute Keeper error: {0}")]
    KeeperFactory(String),
    #[error("Attribute Keeper error: {0}")]
    Keeper(String),
    #[error("Attribute Keeper error: {0}")]
    ManagerFactory(String),
    #[error("Attribute Manager error: {0}")]
    Manager(String),
    #[error("Attribute Aggregator Factory error: {0}")]
    AggregatorFactory(String),
    #[error("Attribute Aggregator error: {0}")]
    Aggregator(String),
    #[error("Attribute Duplicate Filter Factory error: {0}")]
    DuplicateFilterFactory(String),
    #[error("Attribute DuplicateFilter error: {0}")]
    DuplicateFilter(String),
    #[error("Bulk Attribute Renamer Factory error: {0}")]
    BulkRenamerFactory(String),
    #[error("Bulk Attribute Renamer error: {0}")]
    BulkRenamer(String),
    #[error("Attribute File Path Info Factory error: {0}")]
    FilePathInfoExtractorFactory(String),
    #[error("Attribute FilePathInfoExtractor error: {0}")]
    FilePathInfoExtractor(String),
    #[error("StatisticsCalculator Factory error: {0}")]
    StatisticsCalculatorFactory(String),
    #[error("Attribute Mapper Factory error: {0}")]
    MapperFactory(String),
    #[error("Attribute Flattener error: {0}")]
    FlattenerFactory(String),
    #[error("Attribute Flattener error: {0}")]
    Flattener(String),
    #[error("Attribute ConversionTable error: {0}")]
    ConversionTableFactory(String),
    #[error("Attribute ConversionTable error: {0}")]
    ConversionTable(String),
}

#[allow(dead_code)]
pub(super) type Result<T, E = AttributeProcessorError> = std::result::Result<T, E>;
