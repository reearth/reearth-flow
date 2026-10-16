//! `objectList`, the attributes the missing-attribute detector checks for each
//! package and feature type.

use std::collections::{BTreeSet, HashMap};

use super::gml_path::gml_path;
use super::model_sheet::ModelSheet;
use crate::object_list::{ObjectList, ObjectListMap, ObjectListValue};

/// The flood packages share the rows of `08. 災害リスクモデル`; each checks only
/// the target attributes of its own risk type.
const FLOODING_RISK_ATTRIBUTES: [(&str, &str); 5] = [
    (
        "fld",
        "core:adeOfAbstractCityObject/urc:RiverFloodingRiskAttribute",
    ),
    (
        "tnm",
        "core:adeOfAbstractCityObject/urc:TsunamiRiskAttribute",
    ),
    (
        "htd",
        "core:adeOfAbstractCityObject/urc:HighTideRiskAttribute",
    ),
    (
        "ifld",
        "core:adeOfAbstractCityObject/urc:InlandFloodingRiskAttribute",
    ),
    (
        "rfld",
        "core:adeOfAbstractCityObject/urc:ReservoirFloodingRiskAttribute",
    ),
];

/// `(package, root, part)`. The required attributes the root and its part
/// share are required on either one, so they become conditional.
const ROOT_PART_PAIRS: [(&str, &str, &str); 3] = [
    ("bldg", "bldg:Building", "bldg:BuildingPart"),
    ("brid", "brid:Bridge", "brid:BridgePart"),
    ("tun", "tun:Tunnel", "tun:TunnelPart"),
    // No `ubld` pair, for the reason given in `common/profile.rs`.
];

/// Never made conditional, so the root and the part each keep it required.
const BUILDING_ID_ATTRIBUTE: &str = "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute";

/// Per package and feature type (B: 地物名), the checked attributes, `required`
/// when O (不明値) is marked and `target` otherwise. A feature type whose
/// checked attributes are none of them GML elements keeps an empty entry.
pub(super) fn build_object_list(sheets: &[ModelSheet]) -> ObjectListMap {
    let mut by_package = HashMap::<String, HashMap<String, ObjectListValue>>::new();
    for row in sheets.iter().flat_map(|sheet| &sheet.rows) {
        if !row.is_checked() {
            continue;
        }
        let path = gml_path(&row.attribute_names);
        for package in row.packages() {
            let attributes = by_package
                .entry(package.clone())
                .or_default()
                .entry(row.feature_type.clone())
                .or_default();
            let Some(path) = &path else {
                continue;
            };
            if row.is_required {
                attributes.required.push(path.clone());
            } else if !is_other_flooding_risk(&package, path) {
                attributes.target.push(path.clone());
            }
        }
    }

    for (package, root, part) in ROOT_PART_PAIRS {
        if let Some(feature_types) = by_package.get_mut(package) {
            split_off_conditional(feature_types, root, part);
        }
    }

    ObjectListMap::new(
        by_package
            .into_iter()
            .map(|(package, feature_types)| (package, ObjectList::new(feature_types)))
            .collect(),
    )
}

/// The path is a flood risk attribute of a type other than the package's own.
fn is_other_flooding_risk(package: &str, path: &str) -> bool {
    let Some((_, own)) = FLOODING_RISK_ATTRIBUTES.iter().find(|(p, _)| *p == package) else {
        return false;
    };
    !path.starts_with(own)
        && FLOODING_RISK_ATTRIBUTES
            .iter()
            .any(|(_, risk)| path.starts_with(risk))
}

/// Moves the required attributes shared by the root and the part to
/// `conditional`. Each keeps its other required attributes.
fn split_off_conditional(
    feature_types: &mut HashMap<String, ObjectListValue>,
    root: &str,
    part: &str,
) {
    let required = |feature_type: &str| -> BTreeSet<String> {
        feature_types
            .get(feature_type)
            .map(|attributes| {
                attributes
                    .required
                    .iter()
                    .filter(|path| !path.starts_with(BUILDING_ID_ATTRIBUTE))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    };
    let shared = required(root)
        .intersection(&required(part))
        .cloned()
        .collect::<BTreeSet<_>>();
    if shared.is_empty() {
        return;
    }
    for feature_type in [root, part] {
        if let Some(attributes) = feature_types.get_mut(feature_type) {
            attributes.required.retain(|path| !shared.contains(path));
            attributes.conditional = shared.iter().cloned().collect();
        }
    }
}
