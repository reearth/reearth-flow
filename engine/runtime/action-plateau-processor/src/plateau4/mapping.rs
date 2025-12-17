use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    attribute_flattener::processor::AttributeFlattenerFactory,
    building_installation_geometry_type_checker::BuildingInstallationGeometryTypeCheckerFactory,
    building_part_connectivity_checker::BuildingPartConnectivityCheckerFactory,
    building_usage_attribute_validator::BuildingUsageAttributeValidatorFactory,
    city_code_extractor::CityCodeExtractorFactory, citygml_mesh_builder::CityGmlMeshBuilderFactory,
    destination_mesh_code_extractor::DestinationMeshCodeExtractorFactory,
    domain_of_definition_validator::DomainOfDefinitionValidatorFactory,
    face_extractor::FaceExtractorFactory, max_lod_extractor::MaxLodExtractorFactory,
    missing_attribute_detector::MissingAttributeDetectorFactory,
    object_list_extractor::ObjectListExtractorFactory,
    solid_intersection_test_pair_creator::SolidIntersectionTestPairCreatorFactory,
    tran_xlink_detector::TransportationXlinkDetectorFactory,
    udx_folder_extractor::UDXFolderExtractorFactory,
    unmatched_xlink_detector::UnmatchedXlinkDetectorFactory,
    unshared_edge_detector::UnsharedEdgeDetectorFactory,
};

pub(crate) static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<UDXFolderExtractorFactory>::default(),
        Box::<MaxLodExtractorFactory>::default(),
        Box::<AttributeFlattenerFactory>::default(),
        Box::<BuildingInstallationGeometryTypeCheckerFactory>::default(),
        Box::<BuildingPartConnectivityCheckerFactory>::default(),
        Box::<BuildingUsageAttributeValidatorFactory>::default(),
        Box::<CityCodeExtractorFactory>::default(),
        Box::<CityGmlMeshBuilderFactory>::default(),
        Box::<DestinationMeshCodeExtractorFactory>::default(),
        Box::<ObjectListExtractorFactory>::default(),
        Box::<MissingAttributeDetectorFactory>::default(),
        Box::<DomainOfDefinitionValidatorFactory>::default(),
        Box::<UnmatchedXlinkDetectorFactory>::default(),
        Box::<SolidIntersectionTestPairCreatorFactory>::default(),
        Box::<TransportationXlinkDetectorFactory>::default(),
        Box::<FaceExtractorFactory>::default(),
        Box::<UnsharedEdgeDetectorFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
