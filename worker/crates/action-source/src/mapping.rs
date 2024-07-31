use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use crate::{
    feature_creator::FeatureCreatorFactory,
    file::{path_extractor::FilePathExtractorFactory, reader::FileReaderFactory},
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([
        (
            "FileReader".to_string(),
            NodeKind::Source(Box::<FileReaderFactory>::default()),
        ),
        (
            "FilePathExtractor".to_string(),
            NodeKind::Source(Box::<FilePathExtractorFactory>::default()),
        ),
        (
            "FeatureCreator".to_string(),
            NodeKind::Source(Box::<FeatureCreatorFactory>::default()),
        ),
    ])
});
