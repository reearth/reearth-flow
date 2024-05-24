use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use super::{
    aggregator::AttributeAggregatorFactory, duplicate_filter::AttributeDuplicateFilterFactory,
    file_path_info_extractor::AttributeFilePathInfoExtractorFactory,
    keeper::AttributeKeeperFactory, manager::AttributeManagerFactory,
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([
        (
            "AttributeKeeper".to_string(),
            NodeKind::Processor(Box::<AttributeKeeperFactory>::default()),
        ),
        (
            "AttributeManager".to_string(),
            NodeKind::Processor(Box::<AttributeManagerFactory>::default()),
        ),
        (
            "AttributeAggregator".to_string(),
            NodeKind::Processor(Box::<AttributeAggregatorFactory>::default()),
        ),
        (
            "AttributeDuplicateFilter".to_string(),
            NodeKind::Processor(Box::<AttributeDuplicateFilterFactory>::default()),
        ),
        (
            "AttributeFilePathInfoExtractor".to_string(),
            NodeKind::Processor(Box::<AttributeFilePathInfoExtractorFactory>::default()),
        ),
    ])
});
