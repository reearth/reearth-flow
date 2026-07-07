use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::profile::PLATEAU6;
use crate::common::building_part_connectivity_checker::BuildingPartConnectivityCheckerFactory;
use crate::common::building_usage_attribute_validator::BuildingUsageAttributeValidatorFactory;
use crate::common::destination_mesh_code_extractor::DestinationMeshCodeExtractorFactory;
use crate::common::domain_of_definition_validator::DomainOfDefinitionValidatorFactory;
use crate::common::missing_attribute_detector::MissingAttributeDetectorFactory;
use crate::common::object_list_extractor::ObjectListExtractorFactory;
use crate::common::solid_intersection_test_pair_creator::SolidIntersectionTestPairCreatorFactory;
use crate::common::udx_folder_extractor::UDXFolderExtractorFactory;
use crate::common::unmatched_xlink_detector::UnmatchedXlinkDetectorFactory;

use super::building_usage_attribute_strategy::Plateau6BuildingUsageStrategy;
use super::unmatched_xlink_strategy::Plateau6XlinkStrategy;

pub(crate) static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::new(UDXFolderExtractorFactory::new(&PLATEAU6)),
        Box::new(DomainOfDefinitionValidatorFactory::new(&PLATEAU6)),
        Box::new(ObjectListExtractorFactory::new(&PLATEAU6)),
        Box::new(MissingAttributeDetectorFactory::new(&PLATEAU6)),
        Box::new(BuildingUsageAttributeValidatorFactory::new(
            &PLATEAU6,
            &Plateau6BuildingUsageStrategy,
        )),
        Box::new(DestinationMeshCodeExtractorFactory::new(&PLATEAU6)),
        Box::new(BuildingPartConnectivityCheckerFactory::new(&PLATEAU6)),
        Box::new(SolidIntersectionTestPairCreatorFactory::new(&PLATEAU6)),
        Box::new(UnmatchedXlinkDetectorFactory::new(
            &PLATEAU6,
            &Plateau6XlinkStrategy,
        )),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
