mod attribute;
pub mod dataframe;
pub mod error;
pub mod feature;
pub mod geometry;
pub mod types;
pub mod utils;

pub use crate::dataframe::Dataframe;
pub use crate::feature::Feature;

use std::{collections::HashMap, str::FromStr, sync::Arc};

use error::Error;
use nutype::nutype;
use once_cell::sync::Lazy;

pub use attribute::{Attribute, AttributeValue};
use reearth_flow_action_log::{action_log, ActionLogger};
use reearth_flow_common::{
    collection,
    uri::{Uri, PROTOCOL_SEPARATOR},
};
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::{resolve::StorageResolver, storage::Storage};
use reearth_flow_workflow::id::Id;

pub static DEFAULT_PORT: Lazy<Port> = Lazy::new(|| Port::new("default"));
pub static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

pub type ActionDataframe = HashMap<Port, Dataframe>;
pub type FeatureIndex = HashMap<String, Vec<Feature>>;
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
pub trait AsyncAction: Send + Sync {
    async fn run(&self, ctx: ActionContext, input: ActionDataframe) -> ActionResult;
}

#[typetag::serde(tag = "action", content = "with")]
pub trait SyncAction: Send + Sync {
    fn run(&self, ctx: ActionContext, input: ActionDataframe) -> ActionResult;
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

    pub fn with_logger(&self, logger: ActionLogger) -> Self {
        Self {
            logger: Arc::new(logger),
            ..self.clone()
        }
    }

    pub fn with_span(&self, span: tracing::Span) -> Self {
        Self {
            root_span: span,
            ..self.clone()
        }
    }

    pub fn action_log<T: AsRef<str> + std::fmt::Display>(&self, msg: T) {
        action_log!(
            parent: &self.root_span, &self.logger, "{}", msg
        );
    }

    pub fn resolve_uri(&self, uri: &Uri) -> Result<Arc<Storage>> {
        self.storage_resolver
            .resolve(uri)
            .map_err(Error::internal_runtime)
    }

    pub fn get_contents_by_uris(
        &self,
        base_path: String,
        uris: &[String],
    ) -> HashMap<String, String> {
        collection::par_map(uris, |row| {
            let target = if !row.contains(PROTOCOL_SEPARATOR) && !row.starts_with('/') {
                format!("{}/{}", base_path, row)
            } else {
                row.clone()
            };
            let Ok(target) = Uri::from_str(target.as_str()) else {
                return (row.to_string(), "".to_string());
            };
            let Ok(storage) = self.storage_resolver.resolve(&target) else {
                return (row.to_string(), "".to_string());
            };
            let Ok(bytes) = storage.get_sync(target.path().as_path()) else {
                return (row.to_string(), "".to_string());
            };
            let Ok(contents) = String::from_utf8(bytes.to_vec()) else {
                return (row.to_string(), "".to_string());
            };
            (row.to_string(), contents)
        })
        .into_iter()
        .collect::<HashMap<_, _>>()
    }

    pub async fn get_expr_path<T: AsRef<str> + std::fmt::Display>(&self, path: &T) -> Result<Uri> {
        let scope = self.expr_engine.new_scope();
        let path = self
            .expr_engine
            .eval_scope::<String>(path.as_ref(), &scope)
            .map_or_else(|_| path.to_string(), |v| v);
        Uri::from_str(path.as_str()).map_err(error::Error::input)
    }
}
