use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use crate::universal::UniversalSourceFactory;

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([
        (
            "FileReader".to_string(),
            NodeKind::Source(Box::<UniversalSourceFactory>::default()),
        ),
        (
            "FilePathExtractor".to_string(),
            NodeKind::Source(Box::<UniversalSourceFactory>::default()),
        ),
    ])
});
