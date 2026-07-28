use clap::{ArgMatches, Command};

/// The `schema-feature` subcommand.
pub fn build_schema_feature_command() -> Command {
    Command::new("schema-feature")
        .about("Show the production intermediate-data (feature) JSON schema.")
        .long_about(
            "Emit the JSON schema for the production intermediate-data form of a feature: the \
             record each line of intermediate-data JSONL holds, i.e. its id, attributes and \
             geometry. Requires the CLI to be built with `--features schema`.",
        )
}

/// One record of intermediate-data JSONL: a feature's id, its attributes, and its
/// geometry. Must stay in step with the serialized form of
/// `reearth_flow_types::Feature`, which it mirrors for schema generation only.
#[cfg(feature = "schema")]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "Feature")]
#[allow(dead_code)]
struct FeatureSchema {
    id: uuid::Uuid,
    /// Open, string-keyed attribute values; untagged JSON, so unconstrained.
    attributes: std::collections::HashMap<String, serde_json::Value>,
    geometry: reearth_flow_geometry::Geometry,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaFeatureCliCommand;

impl SchemaFeatureCliCommand {
    pub fn parse_cli_args(_matches: ArgMatches) -> crate::Result<Self> {
        Ok(SchemaFeatureCliCommand)
    }

    #[cfg(feature = "schema")]
    pub fn execute(&self) -> crate::Result<()> {
        let schema = schemars::schema_for!(FeatureSchema);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        Ok(())
    }

    #[cfg(not(feature = "schema"))]
    pub fn execute(&self) -> crate::Result<()> {
        Err(crate::errors::Error::init(
            "rebuild the CLI with `--features schema` to generate the intermediate-data schema"
                .to_string(),
        ))
    }
}

#[cfg(all(test, feature = "schema"))]
mod tests {
    /// The committed schema must match the current wire form. If this fails, run
    /// `cargo make schema-feature` and commit the result.
    #[test]
    fn committed_schema_is_up_to_date() {
        let generated =
            serde_json::to_string_pretty(&schemars::schema_for!(super::FeatureSchema)).unwrap();
        let committed = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../schema/feature-intermediate.schema.json"
        ));
        assert_eq!(
            generated.trim(),
            committed.trim(),
            "intermediate-data schema is stale; run `cargo make schema-feature` and commit"
        );
    }
}
