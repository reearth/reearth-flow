use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_action_plateau_processor::mapping::ACTION_MAPPINGS as PLATEAU_MAPPINGS;
use reearth_flow_action_processor::mapping::ACTION_MAPPINGS as PROCESSOR_MAPPINGS;
use reearth_flow_action_sink::mapping::ACTION_MAPPINGS as SINK_MAPPINGS;
use reearth_flow_action_source::mapping::ACTION_MAPPINGS as SOURCE_MAPPINGS;
use reearth_flow_runtime::node::NodeKind;

pub(crate) static BUILTIN_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut common = HashMap::new();
    let sink = SINK_MAPPINGS.clone();
    let source = SOURCE_MAPPINGS.clone();
    let processor = PROCESSOR_MAPPINGS.clone();
    common.extend(sink);
    common.extend(source);
    common.extend(processor);
    common
});

pub(crate) static PLATEAU_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> =
    Lazy::new(|| PLATEAU_MAPPINGS.clone());

pub(crate) static ALL_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut all = HashMap::new();
    all.extend(BUILTIN_ACTION_FACTORIES.clone());
    all.extend(PLATEAU_ACTION_FACTORIES.clone());
    all
});
