use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use crate::{
    attribute::{
        aggregator::AttributeAggregatorFactory, duplicate_filter::AttributeDuplicateFilterFactory,
        keeper::AttributeKeeperFactory, manager::AttributeManagerFactory,
    },
    feature::{
        counter::FeatureCounterFactory, filter::FeatureFilterFactory, merger::FeatureMergerFactory,
        sorter::FeatureSorterFactory, transformer::FeatureTransformerFactory,
    },
    geometry::{
        coordinate_system_setter::CoordinateSystemSetterFactory, extruder::ExtruderFactory,
        three_dimention_box_replacer::ThreeDimentionBoxReplacerFactory,
    },
    plateau::{
        dictionaries_initiator::DictionariesInitiatorFactory,
        domain_of_definition_validator::DomainOfDefinitionValidatorFactory,
        udx_folder_extractor::UdxFolderExtractorFactory,
        xml_attribute_extractor::XmlAttributeExtractorFactory,
    },
    xml::{fragmenter::XmlFragmenterFactory, validator::XmlValidatorFactory},
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([
        (
            "AttributeKeeper".to_string(),
            NodeKind::Processor(Box::<AttributeKeeperFactory>::default()),
        ),
        (
            "AttributeManager".to_string(),
            NodeKind::Processor(Box::<AttributeManagerFactory>::default()),
        ),
        (
            "AttributeAggregator".to_string(),
            NodeKind::Processor(Box::<AttributeAggregatorFactory>::default()),
        ),
        (
            "AttributeDuplicateFilter".to_string(),
            NodeKind::Processor(Box::<AttributeDuplicateFilterFactory>::default()),
        ),
        (
            "FeatureMerger".to_string(),
            NodeKind::Processor(Box::<FeatureMergerFactory>::default()),
        ),
        (
            "FeatureSorter".to_string(),
            NodeKind::Processor(Box::<FeatureSorterFactory>::default()),
        ),
        (
            "FeatureFilter".to_string(),
            NodeKind::Processor(Box::<FeatureFilterFactory>::default()),
        ),
        (
            "FeatureTransformer".to_string(),
            NodeKind::Processor(Box::<FeatureTransformerFactory>::default()),
        ),
        (
            "FeatureCounter".to_string(),
            NodeKind::Processor(Box::<FeatureCounterFactory>::default()),
        ),
        (
            "XMLFragmenter".to_string(),
            NodeKind::Processor(Box::<XmlFragmenterFactory>::default()),
        ),
        (
            "XMLValidator".to_string(),
            NodeKind::Processor(Box::<XmlValidatorFactory>::default()),
        ),
        (
            "CoordinateSystemSetter".to_string(),
            NodeKind::Processor(Box::<CoordinateSystemSetterFactory>::default()),
        ),
        (
            "Extruder".to_string(),
            NodeKind::Processor(Box::<ExtruderFactory>::default()),
        ),
        (
            "ThreeDimentionBoxReplacer".to_string(),
            NodeKind::Processor(Box::<ThreeDimentionBoxReplacerFactory>::default()),
        ),
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
