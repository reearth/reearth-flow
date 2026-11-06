//! `objectListDefinitions`, every feature type (地物型), attribute
//! (主題属性・空間属性) and association role (関連役割) the objectlist defines,
//! for L03(plateau6).

use std::collections::HashMap;

use super::gml_path::gml_path;
use super::model_sheet::ModelSheet;
use crate::object_list::{FeatureTypeDefinition, ObjectListDefinitions};

/// Model prefix (A: モデルの接頭辞) of the appearance rows at the end of a
/// model sheet. They belong to the package of the sheet.
const APPEARANCE_PREFIX: &str = "app";

/// Per package (A: モデルの接頭辞) and feature type (B: 地物名), whether the
/// feature type and each of its rows are creation targets (作成対象). A path
/// that appears on more than one row (a generic attribute type) is usable if
/// any row is.
/// Rows without a prefix (`21. 都市オブジェクトグループモデル`) are left out, as
/// in `objectList`.
pub(super) fn build_definitions(sheets: &[ModelSheet]) -> ObjectListDefinitions {
    let mut by_package = HashMap::<String, HashMap<String, FeatureTypeDefinition>>::new();
    for sheet in sheets {
        let mut packages = Vec::<String>::new();
        for row in &sheet.rows {
            if row.model_prefix != APPEARANCE_PREFIX {
                packages = row
                    .model_prefix
                    .split('/')
                    .filter(|package| !package.is_empty())
                    .map(str::to_string)
                    .collect();
            }
            let path = if row.attribute_names.is_empty() {
                None
            } else {
                match gml_path(&row.attribute_names) {
                    Some(path) => Some(path),
                    // `core:featureID` is not an element.
                    None => continue,
                }
            };
            for package in &packages {
                let definition = by_package
                    .entry(package.clone())
                    .or_default()
                    .entry(row.feature_type.clone())
                    .or_default();
                match &path {
                    None => definition.usable |= row.is_creation_target,
                    Some(path) => {
                        *definition.paths.entry(path.clone()).or_default() |=
                            row.is_creation_target;
                    }
                }
            }
        }
    }
    ObjectListDefinitions::new(by_package)
}
