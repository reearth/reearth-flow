use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    attribute_flattener::AttributeFlattenerFactory,
    building_installation_geometry_type_extractor::BuildingInstallationGeometryTypeExtractorFactory,
    building_usage_attribute_validator::BuildingUsageAttributeValidatorFactory,
    dictionaries_initiator::DictionariesInitiatorFactory,
    domain_of_definition_validator::DomainOfDefinitionValidatorFactory,
    max_lod_extractor::MaxLodExtractorFactory, udx_folder_extractor::UdxFolderExtractorFactory,
    unmatched_xlink_detector::UnmatchedXlinkDetectorFactory,
    xml_attribute_extractor::XmlAttributeExtractorFactory,
};

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<UdxFolderExtractorFactory>::default(),
        Box::<DomainOfDefinitionValidatorFactory>::default(),
        Box::<DictionariesInitiatorFactory>::default(),
        Box::<XmlAttributeExtractorFactory>::default(),
        Box::<UnmatchedXlinkDetectorFactory>::default(),
        Box::<MaxLodExtractorFactory>::default(),
        Box::<AttributeFlattenerFactory>::default(),
        Box::<BuildingInstallationGeometryTypeExtractorFactory>::default(),
        Box::<BuildingUsageAttributeValidatorFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
