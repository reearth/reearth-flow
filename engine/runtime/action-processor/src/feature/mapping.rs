use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    counter::FeatureCounterFactory, filter::FeatureFilterFactory, merger::FeatureMergerFactory,
    reader::FeatureReaderFactory, rhai::RhaiCallerFactory, sorter::FeatureSorterFactory,
    transformer::FeatureTransformerFactory,
};

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<FeatureMergerFactory>::default(),
        Box::<FeatureSorterFactory>::default(),
        Box::<FeatureFilterFactory>::default(),
        Box::<FeatureTransformerFactory>::default(),
        Box::<FeatureCounterFactory>::default(),
        Box::<FeatureReaderFactory>::default(),
        Box::<RhaiCallerFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
