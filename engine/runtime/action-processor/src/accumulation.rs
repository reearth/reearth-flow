//! Attribute accumulation for actions that merge a group of features into one.

use reearth_flow_types::{Attribute, AttributeValue, Attributes};
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
    /// Keeps the attributes of the first merged feature and discards the rest.
    #[default]
    UseOneFeature,
}

/// The attribute `collectAttributes` writes the per-feature maps to.
pub(crate) const MEMBERS_ATTRIBUTE: &str = "__members";

/// The attributes of a merged feature, gathered one merged feature at a time.
pub(crate) struct AttributeAccumulator<'a> {
    strategy: &'a AttributeAccumulation,
    group_by: &'a [Attribute],
    first: Option<Attributes>,
    members: Vec<AttributeValue>,
}

impl<'a> AttributeAccumulator<'a> {
    pub(crate) fn new(strategy: &'a AttributeAccumulation, group_by: &'a [Attribute]) -> Self {
        Self {
            strategy,
            group_by,
            first: None,
            members: Vec::new(),
        }
    }

    /// Take in the attributes of the next merged feature.
    pub(crate) fn push(&mut self, attributes: &Attributes) {
        if self.first.is_none() {
            self.first = Some(match self.strategy {
                AttributeAccumulation::UseOneFeature => attributes.clone(),
                _ => self
                    .group_by
                    .iter()
                    .filter_map(|attr| Some((attr.clone(), attributes.get(attr)?.clone())))
                    .collect(),
            });
        }
        if *self.strategy == AttributeAccumulation::CollectAttributes {
            self.members.push(AttributeValue::Map(
                attributes
                    .iter()
                    .filter(|(key, _)| !self.group_by.contains(key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            ));
        }
    }

    /// The merged feature's attributes. The grouping attributes, and every
    /// attribute under `useOneFeature`, come from the first merged feature.
    pub(crate) fn finish(self) -> Attributes {
        let mut attributes = self.first.unwrap_or_default();
        if *self.strategy == AttributeAccumulation::CollectAttributes {
            attributes.insert(
                Attribute::new(MEMBERS_ATTRIBUTE),
                AttributeValue::Array(self.members),
            );
        }
        attributes
    }
}
