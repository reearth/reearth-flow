use nutype::nutype;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[nutype(
    sanitize(trim),
    derive(
        Debug,
        Display,
        Clone,
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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StringOrExprType {
    /// Evaluated as a Flow expression at runtime
    Expr,
    /// Used as a plain string literal
    String,
}

/// A value that is either a Flow expression or a plain string literal
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StringOrExpr {
    #[serde(rename = "type")]
    pub kind: StringOrExprType,
    pub value: String,
}

impl StringOrExpr {
    pub fn to_rhai_src(&self) -> String {
        match self.kind {
            StringOrExprType::Expr => self.value.clone(),
            StringOrExprType::String => format!("{:?}", self.value),
        }
    }
}
