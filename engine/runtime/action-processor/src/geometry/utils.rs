use num_traits::NumCast;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_types::{Attribute, AttributeValue, Attributes};

pub(super) fn finite_z<Z: CoordNum>(z: Z) -> Option<f64> {
    NumCast::from(z).filter(|z: &f64| z.is_finite())
}

/// The group a feature belongs to.
///
/// Every attribute in `group_by` contributes a slot, with `Null` where the
/// feature does not carry it, so a feature missing an attribute cannot collapse
/// into a group it does not belong to.
pub(super) fn group_key(
    attributes: &Attributes,
    group_by: &Option<Vec<Attribute>>,
) -> AttributeValue {
    match group_by {
        None => AttributeValue::Null,
        Some(attrs) => AttributeValue::Array(
            attrs
                .iter()
                .map(|a| attributes.get(a).cloned().unwrap_or(AttributeValue::Null))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_key_distinguishes_a_missing_attribute_from_a_present_one() {
        // With filter_map, {region: "north"} and {zone: "north"} would both key
        // to ["north"] and share a group.
        let group_by = vec![
            Attribute::new("region".to_string()),
            Attribute::new("zone".to_string()),
        ];

        let mut only_region = Attributes::default();
        only_region.insert(
            Attribute::new("region".to_string()),
            AttributeValue::String("north".to_string()),
        );

        let mut only_zone = Attributes::default();
        only_zone.insert(
            Attribute::new("zone".to_string()),
            AttributeValue::String("north".to_string()),
        );

        assert_ne!(
            group_key(&only_region, &Some(group_by.clone())),
            group_key(&only_zone, &Some(group_by)),
        );
    }
}
