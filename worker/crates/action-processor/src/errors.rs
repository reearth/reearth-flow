use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("UniversalProcessor factory error: {0}")]
    UniversalProcessorFactory(String),

    #[error("Attribute Manager error: {0}")]
    AttributeManagerFactory(String),
    #[error("Attribute Manager error: {0}")]
    AttributeManager(String),

    #[error("Feature Merger Factory error: {0}")]
    FeatureMergerFactory(String),
    #[error("Feature Merger error: {0}")]
    FeatureMerger(String),
    #[error("Feature Sorter Factory error: {0}")]
    FeatureSorterFactory(String),
    #[error("Feature Sorter error: {0}")]
    FeatureSorter(String),
    #[error("Feature Filter Factory error: {0}")]
    FeatureFilterFactory(String),
    #[error("Feature Filter error: {0}")]
    FeatureFilter(String),
    #[error("Feature Transformer Factory error: {0}")]
    FeatureTransformerFactory(String),
    #[error("Feature Transformer error: {0}")]
    FeatureTransformer(String),
    #[error("Feature Counter Factory error: {0}")]
    FeatureCounterFactory(String),
    #[error("Feature Counter error: {0}")]
    FeatureCounter(String),

    #[error("Xml Fragmenter Factory error: {0}")]
    XmlFragmenterFactory(String),
    #[error("Xml Fragmenter error: {0}")]
    XmlFragmenter(String),

    #[error("Xml Validator Factory error: {0}")]
    XmlValidatorFactory(String),
    #[error("Xml Validator error: {0}")]
    XmlValidator(String),
    #[error("Attribute Aggregator Factory error: {0}")]
    AttributeAggregatorFactory(String),
    #[error("Attribute Aggregator error: {0}")]
    AttributeAggregator(String),
    #[error("Attribute Duplicate Filter Factory error: {0}")]
    AttributeDuplicateFilterFactory(String),
    #[error("Attribute DuplicateFilter error: {0}")]
    AttributeDuplicateFilter(String),

    #[error("Extruder Factory error: {0}")]
    ExtruderFactory(String),
    #[error("Extruder error: {0}")]
    Extruder(String),
    #[error("ThreeDimentionBoxReplacer error: {0}")]
    ThreeDimentionBoxReplacer(String),

    #[error("Plateau error: {0}")]
    Plateau(String),
    #[error("UdxFolder Extractor Factory error: {0}")]
    UdxFolderExtractorFactory(String),
    #[error("UdxFolder Extractor error: {0}")]
    UdxFolderExtractor(String),
    #[error("Domain Of Definition Validator Factory error: {0}")]
    DomainOfDefinitionValidatorFactory(String),
    #[error("Domain Of Definition Validator error: {0}")]
    DomainOfDefinitionValidator(String),
    #[error("Dictionaries Initiator Factory error: {0}")]
    DictionariesInitiatorFactory(String),
    #[error("Dictionaries Initiator error: {0}")]
    DictionariesInitiator(String),
    #[error("XmlAttribute Extractor Factory error: {0}")]
    XmlAttributeExtractorFactory(String),
    #[error("XmlAttribute Extractor error: {0}")]
    XmlAttributeExtractor(String),
}

pub type Result<T, E = ProcessorError> = std::result::Result<T, E>;
