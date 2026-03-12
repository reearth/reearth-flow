use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::citygml_attribute_inserter::CityGmlAttributeInserterFactory;
use super::sun_position_calculator::SolarPositionCalculatorFactory;

pub(crate) static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<SolarPositionCalculatorFactory>::default(),
        Box::<CityGmlAttributeInserterFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
