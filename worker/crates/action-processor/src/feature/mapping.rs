use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use super::{
    counter::FeatureCounterFactory, filter::FeatureFilterFactory, merger::FeatureMergerFactory,
    sorter::FeatureSorterFactory, transformer::FeatureTransformerFactory,
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([
        (
            "FeatureMerger".to_string(),
            NodeKind::Processor(Box::<FeatureMergerFactory>::default()),
        ),
        (
            "FeatureSorter".to_string(),
            NodeKind::Processor(Box::<FeatureSorterFactory>::default()),
        ),
        (
            "FeatureFilter".to_string(),
            NodeKind::Processor(Box::<FeatureFilterFactory>::default()),
        ),
        (
            "FeatureTransformer".to_string(),
            NodeKind::Processor(Box::<FeatureTransformerFactory>::default()),
        ),
        (
            "FeatureCounter".to_string(),
            NodeKind::Processor(Box::<FeatureCounterFactory>::default()),
        ),
    ])
});
