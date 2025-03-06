#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while running action: {0}")]
    InternalRuntime(String),

    #[error("InputError: {0}")]
    Input(String),

    #[allow(dead_code)]
    #[error("OutputError: {0}")]
    Output(String),

    #[error("InputError: {0}")]
    Validate(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("ConversionError: {0}")]
    Conversion(String),

    #[allow(dead_code)]
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

impl Error {
    pub fn internal_runtime<T: ToString>(message: T) -> Self {
        Self::InternalRuntime(message.to_string())
    }

    pub fn input<T: ToString>(message: T) -> Self {
        Self::Input(message.to_string())
    }

    #[allow(dead_code)]
    pub fn output<T: ToString>(message: T) -> Self {
        Self::Output(message.to_string())
    }

    pub fn validate<T: ToString>(message: T) -> Self {
        Self::Validate(message.to_string())
    }

    #[allow(dead_code)]
    pub fn io(source: std::io::Error) -> Self {
        Self::IO(source)
    }

    #[allow(dead_code)]
    pub fn unsupported_feature<T: ToString>(message: T) -> Self {
        Self::UnsupportedFeature(message.to_string())
    }
}

// implement Eq and PartialEq for Error so that we can compare errors in tests
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InternalRuntime(a), Self::InternalRuntime(b)) => a == b,
            (Self::Input(a), Self::Input(b)) => a == b,
            (Self::Output(a), Self::Output(b)) => a == b,
            (Self::Validate(a), Self::Validate(b)) => a == b,
            (Self::IO(a), Self::IO(b)) => a.kind() == b.kind(),
            (Self::UnsupportedFeature(a), Self::UnsupportedFeature(b)) => a == b,
            _ => false,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
