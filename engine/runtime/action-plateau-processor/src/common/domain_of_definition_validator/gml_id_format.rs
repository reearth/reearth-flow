//! The format of the gml:id of a geographical object instance.

use once_cell::sync::Lazy;
use reearth_flow_types::{AttributeValue, Feature};
use regex::Regex;
use serde_json::Number;

use super::report::MemberReporter;

/// The gml:id of a city object group, `{any prefix}_{UUID}`.
static GML_ID_GROUP_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z]+_[\da-f]{8}(-[\da-f]{4}){3}-[\da-f]{12}$").unwrap());

/// The gml:id format check of a file.
#[derive(Debug)]
pub(super) struct GmlIdFormatCheck {
    /// `{package}_{UUID}`.
    pattern: Regex,
    not_well_formed: u32,
}

impl GmlIdFormatCheck {
    pub(super) fn new(package: &str) -> Self {
        let mut pattern = "^".to_string();
        pattern.push_str(package);
        pattern.push_str(r"_[\da-f]{8}(-[\da-f]{4}){3}-[\da-f]{12}$");
        Self {
            pattern: Regex::new(pattern.as_str()).unwrap(),
            not_well_formed: 0,
        }
    }

    /// Checks the gml:id of a member, whose element is `tag`.
    pub(super) fn check_member(
        &mut self,
        gml_id: &str,
        is_city_object_group: bool,
        tag: &str,
        report: &MemberReporter,
    ) {
        if self.pattern.is_match(gml_id)
            || (is_city_object_group && GML_ID_GROUP_PATTERN.is_match(gml_id))
        {
            return;
        }
        self.not_well_formed += 1;
        report.send("GMLID_NotWellFormed", [("tag", tag.to_string())]);
    }

    pub(super) fn write_summary(&self, summary: &mut Feature) {
        summary.insert(
            "gmlIdNotWellformed",
            AttributeValue::Number(Number::from(self.not_well_formed)),
        );
    }
}
