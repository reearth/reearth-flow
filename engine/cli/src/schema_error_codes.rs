use clap::Command;
use reearth_flow_diagnostics::{Disposition, ErrorCategory, ErrorCode};

pub fn build_schema_error_codes_command() -> Command {
    Command::new("schema-error-codes")
        .about("Show the error-code registry schema.")
        .long_about(
            "Dump the error-code registry (code, category, default disposition, message, help) \
             as JSON. Used to generate the frontend-consumable schema/error-codes.json.",
        )
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaErrorCodesCliCommand;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorCodeSchema {
    code: &'static str,
    category: ErrorCategory,
    default_disposition: Disposition,
    message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<&'static str>,
}

impl SchemaErrorCodesCliCommand {
    pub fn execute(&self) -> crate::Result<()> {
        let entries: Vec<ErrorCodeSchema> = ErrorCode::ALL
            .iter()
            .map(|code| ErrorCodeSchema {
                code: code.as_str(),
                category: code.category(),
                default_disposition: code.default_disposition(),
                message: code.default_message(),
                help: code.default_help(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries).unwrap());
        Ok(())
    }
}
