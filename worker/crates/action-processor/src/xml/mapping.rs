use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use super::{fragmenter::XmlFragmenterFactory, validator::XmlValidatorFactory};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([
        (
            "XMLFragmenter".to_string(),
            NodeKind::Processor(Box::<XmlFragmenterFactory>::default()),
        ),
        (
            "XMLValidator".to_string(),
            NodeKind::Processor(Box::<XmlValidatorFactory>::default()),
        ),
    ])
});
