use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use crate::plateau3;

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut mapping = HashMap::new();
    mapping.extend(plateau3::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping
});
