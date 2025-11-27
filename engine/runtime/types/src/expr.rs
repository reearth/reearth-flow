use nutype::nutype;

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

#[cfg(feature = "analyzer")]
impl reearth_flow_analyzer_core::DataSize for Expr {
    fn data_size(&self) -> usize {
        self.as_ref().len()
    }
}
