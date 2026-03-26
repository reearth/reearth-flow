use nutype::nutype;

#[nutype(
    sanitize(trim),
    default = "",
    derive(
        Debug,
        Display,
        Clone,
        Default,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        AsRef,
        Serialize,
        Deserialize,
        Hash,
        JsonSchema
    )
)]
pub struct Expr(String);
