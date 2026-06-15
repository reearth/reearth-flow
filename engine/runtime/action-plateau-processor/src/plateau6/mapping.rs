use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::profile::PLATEAU6;
use crate::common::domain_of_definition_validator::DomainOfDefinitionValidatorFactory;
use crate::common::missing_attribute_detector::MissingAttributeDetectorFactory;
use crate::common::object_list_extractor::ObjectListExtractorFactory;
use crate::common::udx_folder_extractor::UDXFolderExtractorFactory;

pub(crate) static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::new(UDXFolderExtractorFactory::new(&PLATEAU6)),
        Box::new(DomainOfDefinitionValidatorFactory::new(&PLATEAU6)),
        Box::new(ObjectListExtractorFactory::new(&PLATEAU6)),
        Box::new(MissingAttributeDetectorFactory::new(&PLATEAU6)),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
