use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use crate::file::writer::FileWriterSinkFactory;

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([(
        "FileWriter".to_string(),
        NodeKind::Sink(Box::<FileWriterSinkFactory>::default()),
    )])
});
