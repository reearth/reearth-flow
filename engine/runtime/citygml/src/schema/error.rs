/// Why a schema set could not be built. Every variant names where it came
/// from, except a compile error, which belongs to the set as a whole.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchemaError {
    #[error("{file}: {message}")]
    Read { file: String, message: String },
    #[error("{file}: {message}")]
    Parse { file: String, message: String },
    #[error("{from}: cannot find the schema it imports from `{location}`")]
    MissingImport { from: String, location: String },
    #[error("the schema set does not compile: {message}")]
    Compile { message: String },
    #[error("{kind} `{name}` refers back to itself")]
    Cycle { kind: &'static str, name: String },
}
