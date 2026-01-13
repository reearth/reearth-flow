use std::sync::Arc;

use reearth_flow_state::State;

#[derive(Clone, Debug)]
pub struct IncrementalRunConfig {
    pub start_node_id: uuid::Uuid,
    pub previous_feature_state: Arc<State>,
}
