//! Attribute accumulation shared by the actions that merge a group of features
//! into one.

use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # Attribute Accumulation
/// Which attributes a merged feature keeps.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AttributeAccumulation {
    /// # Keep Group Attributes Only
    /// Keeps only the grouping attributes, discarding everything else.
    GroupAttributesOnly,
    /// # Collect Attributes
    /// Keeps the grouping attributes and writes the other attributes of every merged feature to
    /// `__members`, an array with one map per feature in arrival order.
    CollectAttributes,
    /// # Use Attributes From One Feature
    /// Keeps the attributes of a single feature from the group and discards the rest.
    #[default]
    UseOneFeature,
}

/// The attribute `collectAttributes` writes the per-feature maps to.
pub(crate) const MEMBERS_ATTRIBUTE: &str = "__members";

/// One feature's attributes not in `exclude`, as a map.
pub(crate) fn member(attributes: &Attributes, exclude: &[Attribute]) -> AttributeValue {
    AttributeValue::Map(
        attributes
            .iter()
            .filter(|(key, _)| !exclude.contains(key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

/// The attributes a group's merged feature keeps under `strategy`. The group
/// attributes and `useOneFeature` read from `representative`.
pub(crate) fn accumulate_attributes(
    strategy: &AttributeAccumulation,
    features: &[Feature],
    representative: Option<&Feature>,
    group_by: &[Attribute],
) -> Attributes {
    let group_attributes = || -> Attributes {
        let Some(representative) = representative else {
            return Attributes::new();
        };
        group_by
            .iter()
            .filter_map(|attr| Some((attr.clone(), representative.attributes.get(attr)?.clone())))
            .collect()
    };
    match strategy {
        AttributeAccumulation::GroupAttributesOnly => group_attributes(),
        AttributeAccumulation::CollectAttributes => {
            let mut attributes = group_attributes();
            attributes.insert(
                Attribute::new(MEMBERS_ATTRIBUTE),
                AttributeValue::Array(
                    features
                        .iter()
                        .map(|f| member(&f.attributes, group_by))
                        .collect(),
                ),
            );
            attributes
        }
        AttributeAccumulation::UseOneFeature => representative
            .map(|feature| (*feature.attributes).clone())
            .unwrap_or_default(),
    }
}
