use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::domain_of_definition_validator::DomainOfDefinitionValidatorFactory;
use super::udx_folder_extractor::UDXFolderExtractorFactory;

pub(crate) static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::new(UDXFolderExtractorFactory),
        Box::new(DomainOfDefinitionValidatorFactory),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
