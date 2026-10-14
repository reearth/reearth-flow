//! Pass 2: `featureTypes`, the feature types the XML Fragmenter cuts out of
//! each package.

use std::collections::HashMap;

use super::gml_path::gml_path;
use super::model_sheet::ModelSheet;
use crate::object_list::FeatureTypes;

/// Per package, the B column (地物名) of every feature type with at least one
/// checked attribute that is a GML element.
pub(super) fn build_feature_types(sheets: &[ModelSheet]) -> FeatureTypes {
    let mut by_package = HashMap::<String, Vec<String>>::new();
    for row in sheets.iter().flat_map(|sheet| &sheet.rows) {
        if !row.is_checked() || gml_path(&row.attribute_names).is_none() {
            continue;
        }
        for package in row.packages() {
            let feature_types = by_package.entry(package).or_default();
            if !feature_types.contains(&row.feature_type) {
                feature_types.push(row.feature_type.clone());
            }
        }
    }
    FeatureTypes::new(by_package)
}
