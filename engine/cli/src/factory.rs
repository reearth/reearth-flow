use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_action_plateau_processor::mapping::ACTION_FACTORY_MAPPINGS as PLATEAU_MAPPINGS;
use reearth_flow_action_processor::mapping::ACTION_FACTORY_MAPPINGS as PROCESSOR_MAPPINGS;
use reearth_flow_action_python_processor::ACTION_FACTORY_MAPPINGS as PYTHON_MAPPINGS;
use reearth_flow_action_sink::mapping::ACTION_FACTORY_MAPPINGS as SINK_MAPPINGS;
use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS as SOURCE_MAPPINGS;
use reearth_flow_runtime::node::{NodeKind, SYSTEM_ACTION_FACTORY_MAPPINGS};

pub(crate) static BUILTIN_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    SOURCE_MAPPINGS
        .iter()
        .chain(PROCESSOR_MAPPINGS.iter())
        .chain(SINK_MAPPINGS.iter())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
});

pub(crate) static PLATEAU_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> =
    Lazy::new(|| PLATEAU_MAPPINGS.clone());

pub(crate) static PYTHON_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> =
    Lazy::new(|| PYTHON_MAPPINGS.clone());

pub(crate) static ALL_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    BUILTIN_ACTION_FACTORIES
        .iter()
        .chain(PLATEAU_ACTION_FACTORIES.iter())
        .chain(PYTHON_ACTION_FACTORIES.iter())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
});

/// Look up an action by name across all registries.
/// Returns `(NodeKind, is_builtin)`.
pub(crate) fn find_action_by_name(name: &str) -> Option<(NodeKind, bool)> {
    if let Some(kind) = BUILTIN_ACTION_FACTORIES.get(name) {
        return Some((kind.clone(), true));
    }
    if let Some(kind) = SYSTEM_ACTION_FACTORY_MAPPINGS.get(name) {
        return Some((kind.clone(), true));
    }
    if let Some(kind) = PLATEAU_ACTION_FACTORIES.get(name) {
        return Some((kind.clone(), false));
    }
    if let Some(kind) = PYTHON_ACTION_FACTORIES.get(name) {
        return Some((kind.clone(), false));
    }
    None
}
