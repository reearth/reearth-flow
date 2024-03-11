pub mod graph;
pub mod id;
pub mod workflow;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("WorkflowError: {0}")]
    Workflow(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
