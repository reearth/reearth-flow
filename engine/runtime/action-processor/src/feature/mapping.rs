use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    counter::FeatureCounterFactory, file_path_extractor::FeatureFilePathExtractorFactory,
    filter::FeatureFilterFactory, list_exploder::ListExploderFactory, merger::FeatureMergerFactory,
    reader::FeatureReaderFactory, rhai::RhaiCallerFactory, sorter::FeatureSorterFactory,
    transformer::FeatureTransformerFactory, type_filter::FeatureTypeFilterFactory,
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
        Box::<ListExploderFactory>::default(),
        Box::<FeatureTypeFilterFactory>::default(),
        Box::<FeatureFilePathExtractorFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
