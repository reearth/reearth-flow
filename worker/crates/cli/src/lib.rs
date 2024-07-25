pub mod cli;
pub mod dot;
pub(crate) mod factory;
pub mod logger;
pub mod run;
pub mod schema_action;
pub mod schema_workflow;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ParseError: {0}")]
    Parse(String),

    #[error("Failed to initialize cli: {0}")]
    Init(String),

    #[error("Failed to run cli: {0}")]
    Run(String),

    #[error("UnknownCommand: {0}")]
    UnknownCommand(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub(crate) fn parse<T: ToString>(message: T) -> Self {
        Self::Parse(message.to_string())
    }

    pub(crate) fn init<T: ToString>(message: T) -> Self {
        Self::Init(message.to_string())
    }

    pub(crate) fn unknown_command<T: ToString>(message: T) -> Self {
        Self::UnknownCommand(message.to_string())
    }
}
