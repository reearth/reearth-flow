use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use crate::{attribute, feature, geometry, plateau, xml};

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut mapping = HashMap::new();
    mapping.extend(attribute::mapping::ACTION_MAPPINGS.clone());
    mapping.extend(feature::mapping::ACTION_MAPPINGS.clone());
    mapping.extend(geometry::mapping::ACTION_MAPPINGS.clone());
    mapping.extend(plateau::mapping::ACTION_MAPPINGS.clone());
    mapping.extend(xml::mapping::ACTION_MAPPINGS.clone());
    mapping
});
