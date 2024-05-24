use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use super::{
    coordinate_system_setter::CoordinateSystemSetterFactory, extruder::ExtruderFactory,
    three_dimention_box_replacer::ThreeDimentionBoxReplacerFactory,
};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    HashMap::from([
        (
            "CoordinateSystemSetter".to_string(),
            NodeKind::Processor(Box::<CoordinateSystemSetterFactory>::default()),
        ),
        (
            "Extruder".to_string(),
            NodeKind::Processor(Box::<ExtruderFactory>::default()),
        ),
        (
            "ThreeDimentionBoxReplacer".to_string(),
            NodeKind::Processor(Box::<ThreeDimentionBoxReplacerFactory>::default()),
        ),
    ])
});
