use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::profile::PLATEAU4;
use super::{
    attribute_flattener::processor::AttributeFlattenerFactory,
    building_installation_geometry_type_checker::BuildingInstallationGeometryTypeCheckerFactory,
    city_code_extractor::CityCodeExtractorFactory, citygml_mesh_builder::CityGmlMeshBuilderFactory,
    composite_surface_continuity_filter::CompositeSurfaceContinuityFilterFactory,
    destination_mesh_code_extractor::DestinationMeshCodeExtractorFactory,
    face_extractor::FaceExtractorFactory,
    flooding_area_surface_generator::FloodingAreaSurfaceGeneratorFactory,
    gml_name_code_space_validator::GmlNameCodeSpaceValidatorFactory,
    max_lod_extractor::MaxLodExtractorFactory,
    tran_xlink_detector::TransportationXlinkDetectorFactory,
    unshared_edge_detector::UnsharedEdgeDetectorFactory,
};
use crate::common::building_part_connectivity_checker::BuildingPartConnectivityCheckerFactory;
use crate::common::building_usage_attribute_validator::BuildingUsageAttributeValidatorFactory;
use crate::common::domain_of_definition_validator::DomainOfDefinitionValidatorFactory;
use crate::common::missing_attribute_detector::MissingAttributeDetectorFactory;
use crate::common::object_list_extractor::ObjectListExtractorFactory;
use crate::common::solid_intersection_test_pair_creator::SolidIntersectionTestPairCreatorFactory;
use crate::common::udx_folder_extractor::UDXFolderExtractorFactory;
use crate::common::unmatched_xlink_detector::UnmatchedXlinkDetectorFactory;

use super::unmatched_xlink_strategy::Plateau4XlinkStrategy;

pub(crate) static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::new(UDXFolderExtractorFactory::new(&PLATEAU4)),
        Box::<MaxLodExtractorFactory>::default(),
        Box::<AttributeFlattenerFactory>::default(),
        Box::<BuildingInstallationGeometryTypeCheckerFactory>::default(),
        Box::new(BuildingPartConnectivityCheckerFactory::new(&PLATEAU4)),
        Box::new(BuildingUsageAttributeValidatorFactory::new(&PLATEAU4)),
        Box::<CityCodeExtractorFactory>::default(),
        Box::<CityGmlMeshBuilderFactory>::default(),
        Box::<DestinationMeshCodeExtractorFactory>::default(),
        Box::<FloodingAreaSurfaceGeneratorFactory>::default(),
        Box::<GmlNameCodeSpaceValidatorFactory>::default(),
        Box::new(ObjectListExtractorFactory::new(&PLATEAU4)),
        Box::new(MissingAttributeDetectorFactory::new(&PLATEAU4)),
        Box::new(DomainOfDefinitionValidatorFactory::new(&PLATEAU4)),
        Box::new(UnmatchedXlinkDetectorFactory::new(
            &PLATEAU4,
            &Plateau4XlinkStrategy,
        )),
        Box::new(SolidIntersectionTestPairCreatorFactory::new(&PLATEAU4)),
        Box::<TransportationXlinkDetectorFactory>::default(),
        Box::<FaceExtractorFactory>::default(),
        Box::<UnsharedEdgeDetectorFactory>::default(),
        Box::<CompositeSurfaceContinuityFilterFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
