use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    attribute_flattener::processor::AttributeFlattenerFactory,
    city_code_extractor::CityCodeExtractorFactory,
    domain_of_definition_validator::DomainOfDefinitionValidatorFactory,
    max_lod_extractor::MaxLodExtractorFactory,
    missing_attribute_detector::MissingAttributeDetectorFactory,
    object_list_extractor::ObjectListExtractorFactory,
    udx_folder_extractor::UDXFolderExtractorFactory,
};

pub(crate) static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<UDXFolderExtractorFactory>::default(),
        Box::<MaxLodExtractorFactory>::default(),
        Box::<AttributeFlattenerFactory>::default(),
        Box::<CityCodeExtractorFactory>::default(),
        Box::<ObjectListExtractorFactory>::default(),
        Box::<MissingAttributeDetectorFactory>::default(),
        Box::<DomainOfDefinitionValidatorFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
