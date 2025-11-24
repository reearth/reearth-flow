use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use crate::{
    attribute, echo::EchoProcessorFactory, feature, file, geometry, http,
    noop::NoopProcessorFactory, xml,
};

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut mapping = HashMap::new();
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<EchoProcessorFactory>::default(),
        Box::<NoopProcessorFactory>::default(),
    ];
    mapping.extend(
        factories
            .into_iter()
            .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
            .collect::<HashMap<_, _>>(),
    );
    mapping.extend(attribute::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(feature::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(geometry::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(http::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(xml::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(file::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping
});
