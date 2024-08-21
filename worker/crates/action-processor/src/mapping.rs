use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::node::NodeKind;

use crate::{attribute, feature, file, geometry, xml};

pub static ACTION_FACTORY_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut mapping = HashMap::new();
    mapping.extend(attribute::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(feature::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(geometry::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(xml::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping.extend(file::mapping::ACTION_FACTORY_MAPPINGS.clone());
    mapping
});
