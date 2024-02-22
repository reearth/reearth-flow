#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while running engine: {0}")]
    InternalRuntime(String),

    #[error("InputError: {0}")]
    Input(String),

    #[error("OutputError: {0}")]
    Output(String),

    #[error("Failed to initialize engine: {0}")]
    Init(String),

    #[error("Failed to convert value: {0}")]
    Convert(String),
}

impl Error {
    pub fn internal_runtime_error<T: ToString>(message: T) -> Self {
        Self::InternalRuntime(message.to_string())
    }

    pub fn input_error<T: ToString>(message: T) -> Self {
        Self::Input(message.to_string())
    }

    pub fn output_error<T: ToString>(message: T) -> Self {
        Self::Output(message.to_string())
    }

    pub fn init_error<T: ToString>(message: T) -> Self {
        Self::Init(message.to_string())
    }

    pub fn convert_error<T: ToString>(message: T) -> Self {
        Self::Convert(message.to_string())
    }
}

// implement Eq and PartialEq for Error so that we can compare errors in tests
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InternalRuntime(a), Self::InternalRuntime(b)) => a == b,
            (Self::Input(a), Self::Input(b)) => a == b,
            (Self::Output(a), Self::Output(b)) => a == b,
            (Self::Init(a), Self::Init(b)) => a == b,
            (Self::Convert(a), Self::Convert(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(
            Error::internal_runtime_error("hello"),
            Error::internal_runtime_error("hello")
        );
        assert_eq!(Error::input_error("hello"), Error::input_error("hello"));
        assert_eq!(Error::output_error("hello"), Error::output_error("hello"));
        assert_eq!(Error::init_error("hello"), Error::init_error("hello"));
        assert_eq!(Error::convert_error("hello"), Error::convert_error("hello"));
    }

    #[test]
    fn test_ne() {
        assert_ne!(
            Error::internal_runtime_error("hello"),
            Error::internal_runtime_error("world")
        );
        assert_ne!(Error::input_error("hello"), Error::input_error("world"));
        assert_ne!(Error::output_error("hello"), Error::output_error("world"));
        assert_ne!(Error::init_error("hello"), Error::init_error("world"));
        assert_ne!(Error::convert_error("hello"), Error::convert_error("world"));
    }
}
