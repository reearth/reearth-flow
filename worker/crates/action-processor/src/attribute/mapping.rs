use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::{NodeKind, ProcessorFactory};

use super::{
    aggregator::AttributeAggregatorFactory, duplicate_filter::AttributeDuplicateFilterFactory,
    file_path_info_extractor::AttributeFilePathInfoExtractorFactory,
    keeper::AttributeKeeperFactory, manager::AttributeManagerFactory,
    statistics_calculator::StatisticsCalculatorFactory,
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let factories: Vec<Box<dyn ProcessorFactory>> = vec![
        Box::<AttributeKeeperFactory>::default(),
        Box::<AttributeManagerFactory>::default(),
        Box::<AttributeAggregatorFactory>::default(),
        Box::<AttributeDuplicateFilterFactory>::default(),
        Box::<AttributeFilePathInfoExtractorFactory>::default(),
        Box::<StatisticsCalculatorFactory>::default(),
    ];
    factories
        .into_iter()
        .map(|f| (f.name().to_string(), NodeKind::Processor(f)))
        .collect::<HashMap<_, _>>()
});
