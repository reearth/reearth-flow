/// Why a schema set could not be built. Every variant names where it came from.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchemaError {
    #[error("{file}: {message}")]
    Read { file: String, message: String },
    #[error("{file}, line {line}: malformed XML: {message}")]
    Xml {
        file: String,
        line: usize,
        message: String,
    },
    #[error("{file}: {message}")]
    Invalid { file: String, message: String },
    #[error("{file}: `{value}` uses the undeclared namespace prefix `{prefix}`")]
    UnboundPrefix {
        file: String,
        prefix: String,
        value: String,
    },
    #[error("{file}: {construct} is not supported")]
    Unsupported {
        file: String,
        construct: &'static str,
    },
    #[error("{from}: cannot find the schema it imports from `{location}`")]
    MissingImport { from: String, location: String },
    #[error("{from}: {kind} `{name}` is not defined in any loaded schema")]
    Unresolved {
        from: String,
        kind: &'static str,
        name: String,
    },
    #[error("{kind} `{name}` refers back to itself")]
    Cycle { kind: &'static str, name: String },
}
