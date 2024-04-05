pub mod error;
pub mod types;
pub mod utils;
mod value;

use std::{collections::HashMap, sync::Arc};

use nutype::nutype;
use once_cell::sync::Lazy;

use reearth_flow_action_log::ActionLogger;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_workflow::id::Id;

pub use value::ActionValue;

pub static DEFAULT_PORT: Lazy<Port> = Lazy::new(|| Port::new("default"));
pub static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

pub type ActionDataframe = HashMap<Port, Option<ActionValue>>;
pub type ActionValueIndex = HashMap<String, HashMap<String, Vec<ActionValue>>>;
pub type ActionResult = std::result::Result<ActionDataframe, error::Error>;
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[nutype(
    sanitize(trim),
    derive(
        Debug,
        Clone,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        AsRef,
        Serialize,
        Deserialize,
        Hash
    )
)]
pub struct Port(String);

#[async_trait::async_trait]
#[typetag::serde(tag = "action", content = "with")]
pub trait Action: Send + Sync {
    async fn run(&self, ctx: ActionContext, input: Option<ActionDataframe>) -> ActionResult;
}

#[derive(Debug, Clone)]
pub struct ActionContext {
    pub job_id: Id,
    pub workflow_id: Id,
    pub node_id: Id,
    pub node_name: String,
    pub expr_engine: Arc<Engine>,
    pub storage_resolver: Arc<StorageResolver>,
    pub logger: Arc<ActionLogger>,
    pub root_span: tracing::Span,
}

impl Default for ActionContext {
    fn default() -> Self {
        Self {
            job_id: Default::default(),
            workflow_id: Default::default(),
            node_id: Default::default(),
            node_name: "".to_owned(),
            expr_engine: Arc::new(Engine::new()),
            storage_resolver: Arc::new(StorageResolver::new()),
            logger: Arc::new(ActionLogger::root(
                reearth_flow_action_log::Discard,
                reearth_flow_action_log::o!(),
            )),
            root_span: tracing::Span::current(),
        }
    }
}

impl ActionContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        job_id: Id,
        workflow_id: Id,
        node_id: Id,
        node_name: String,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        logger: ActionLogger,
        root_span: tracing::Span,
    ) -> Self {
        Self {
            job_id,
            workflow_id,
            node_id,
            node_name,
            expr_engine,
            storage_resolver,
            logger: Arc::new(logger),
            root_span,
        }
    }
}
