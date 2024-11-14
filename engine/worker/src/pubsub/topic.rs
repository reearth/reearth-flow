#[nutype::nutype(
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
        Hash,
        Display,
    )
)]
pub(crate) struct Topic(String);
