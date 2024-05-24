use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use super::{
    dictionaries_initiator::DictionariesInitiatorFactory,
    domain_of_definition_validator::DomainOfDefinitionValidatorFactory,
    udx_folder_extractor::UdxFolderExtractorFactory,
    xml_attribute_extractor::XmlAttributeExtractorFactory,
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([
        (
            "PLATEAU.UDXFolderExtractor".to_string(),
            NodeKind::Processor(Box::<UdxFolderExtractorFactory>::default()),
        ),
        (
            "PLATEAU.DomainOfDefinitionValidator".to_string(),
            NodeKind::Processor(Box::<DomainOfDefinitionValidatorFactory>::default()),
        ),
        (
            "PLATEAU.DictionariesInitiator".to_string(),
            NodeKind::Processor(Box::<DictionariesInitiatorFactory>::default()),
        ),
        (
            "PLATEAU.XMLAttributeExtractor".to_string(),
            NodeKind::Processor(Box::<XmlAttributeExtractorFactory>::default()),
        ),
    ])
});
