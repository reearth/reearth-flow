#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while running engine: {0}")]
    ExprInternalRuntime(String),

    #[error("Error while compile engine: {0}")]
    ExprCompile(String),

    #[error("InputError: {0}")]
    ExprInput(String),

    #[error("OutputError: {0}")]
    ExprOutput(String),

    #[error("Failed to initialize engine: {0}")]
    ExprInit(String),

    #[error("Failed to convert value: {0}")]
    ExprConvert(String),
}

impl Error {
    pub fn internal_runtime_error<T: ToString>(message: T) -> Self {
        Self::ExprInternalRuntime(message.to_string())
    }

    pub fn input_error<T: ToString>(message: T) -> Self {
        Self::ExprInput(message.to_string())
    }

    pub fn output_error<T: ToString>(message: T) -> Self {
        Self::ExprOutput(message.to_string())
    }

    pub fn init_error<T: ToString>(message: T) -> Self {
        Self::ExprInit(message.to_string())
    }

    pub fn convert_error<T: ToString>(message: T) -> Self {
        Self::ExprConvert(message.to_string())
    }
}

// implement Eq and PartialEq for Error so that we can compare errors in tests
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ExprInternalRuntime(a), Self::ExprInternalRuntime(b)) => a == b,
            (Self::ExprInput(a), Self::ExprInput(b)) => a == b,
            (Self::ExprOutput(a), Self::ExprOutput(b)) => a == b,
            (Self::ExprInit(a), Self::ExprInit(b)) => a == b,
            (Self::ExprConvert(a), Self::ExprConvert(b)) => a == b,
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
