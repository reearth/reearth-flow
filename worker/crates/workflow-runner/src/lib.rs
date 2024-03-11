mod action_runner;
pub mod dag;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get dag node with: {0}")]
    NodeNotFound(String),

    #[error("Error while running workflow: {0}")]
    #[allow(dead_code)]
    Execution(String),

    #[error("Failed to add edge with: {0}")]
    EdgeAlreadyExists(String),

    #[error("Failed to initialize dag: {0}")]
    #[allow(dead_code)]
    Init(String),

    #[error("Failed to run action: {0}, {1}")]
    #[allow(dead_code)]
    Action(reearth_flow_action::error::Error, String),
}

impl Error {
    pub fn node_not_found<T: ToString>(message: T) -> Self {
        Self::NodeNotFound(message.to_string())
    }

    #[allow(dead_code)]
    pub fn init<T: ToString>(message: T) -> Self {
        Self::Init(message.to_string())
    }

    #[allow(dead_code)]
    pub fn execution<T: ToString>(message: T) -> Self {
        Self::Execution(message.to_string())
    }

    pub fn edge_already_exists<T: ToString>(message: T) -> Self {
        Self::EdgeAlreadyExists(message.to_string())
    }

    pub fn action<T: ToString>(error: reearth_flow_action::error::Error, message: T) -> Self {
        Self::Action(error, message.to_string())
    }
}

// implement Eq and PartialEq for Error so that we can compare errors in tests
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NodeNotFound(a), Self::NodeNotFound(b)) => a == b,
            (Self::Init(a), Self::Init(b)) => a == b,
            (Self::Execution(a), Self::Execution(b)) => a == b,
            (Self::EdgeAlreadyExists(a), Self::EdgeAlreadyExists(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(Error::execution("hello"), Error::execution("hello"));
        assert_eq!(Error::init("hello"), Error::init("hello"));
        assert_eq!(
            Error::node_not_found("hello"),
            Error::node_not_found("hello")
        );
        assert_eq!(
            Error::edge_already_exists("hello"),
            Error::edge_already_exists("hello")
        );
    }
}
