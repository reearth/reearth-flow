use reearth_flow_common::uri::Uri;
use serde::{Deserialize, Serialize};

use crate::attribute::AttributeValue;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilePath {
    pub path: String,
    pub name: String,
    pub extension: String,
}

impl FilePath {
    pub fn new(path: String, name: String, extension: String) -> Self {
        Self {
            path,
            name,
            extension,
        }
    }
}

impl TryFrom<Uri> for FilePath {
    type Error = crate::error::Error;

    fn try_from(uri: Uri) -> Result<Self, Self::Error> {
        let path = uri.to_string();
        let name = uri
            .file_name()
            .ok_or(crate::error::Error::validate(format!(
                "invalid uri with {uri:?}"
            )))?;
        let extension = uri
            .extension()
            .ok_or(crate::error::Error::validate(format!(
                "invalid uri with {uri:?}"
            )))?;
        Ok(Self::new(
            path,
            name.to_str()
                .ok_or(crate::error::Error::validate(format!(
                    "invalid uri with {uri:?}"
                )))?
                .to_string(),
            extension.to_string(),
        ))
    }
}

impl TryFrom<FilePath> for AttributeValue {
    type Error = crate::error::Error;

    fn try_from(value: FilePath) -> Result<Self, Self::Error> {
        let value: serde_json::Value =
            serde_json::to_value(value).map_err(crate::error::Error::internal_runtime)?;
        Ok(value.into())
    }
}
