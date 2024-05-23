use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub(super) enum PlateauProcessorError {
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

pub(super) type Result<T, E = PlateauProcessorError> = std::result::Result<T, E>;
