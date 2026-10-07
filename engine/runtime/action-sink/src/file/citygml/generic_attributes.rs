//! Flow-computed values written as CityGML generic attributes (`gen:`).

use std::collections::HashSet;

use reearth_flow_common::datetime::DateTime;
use reearth_flow_types::{Attribute, AttributeValue, Attributes};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::properties::{format_number, Skip, XmlProperty};

/// # Generic Attribute
/// One computed value to write as a CityGML generic attribute (`gen:`).
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenericAttribute {
    /// # Name
    /// Name written to the file as the generic attribute's `name`. A generic attribute the source already had under this name is replaced.
    pub name: String,
    /// # Attribute
    /// Feature attribute holding the value. A feature without it, or with an empty value, writes nothing for this entry. A number is written as `gen:intAttribute` or `gen:doubleAttribute`, a date as `gen:dateAttribute`, and text, true/false and date-times as `gen:stringAttribute`; lists and maps are not written.
    pub attribute: Attribute,
    /// # Unit
    /// Unit of measure, such as `kWh` or `m`. When set, the value must be a number or numeric text, and is written as `gen:measureAttribute`; any other value is not written.
    #[serde(default)]
    pub uom: Option<String>,
}

/// Reject entries that cannot be written, naming the first offending field.
pub fn validate(entries: &[GenericAttribute]) -> Result<(), String> {
    let mut names = HashSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let blank = |field: &str| format!("`genericAttributes[{index}].{field}` is blank");
        if entry.name.trim().is_empty() {
            return Err(blank("name"));
        }
        if entry.attribute.as_str().trim().is_empty() {
            return Err(blank("attribute"));
        }
        if entry
            .uom
            .as_deref()
            .is_some_and(|uom| uom.trim().is_empty())
        {
            return Err(blank("uom"));
        }
        if !names.insert(entry.name.as_str()) {
            return Err(format!(
                "`genericAttributes` names `{}` twice; each name may appear once",
                entry.name
            ));
        }
    }
    Ok(())
}

/// The configured entries this feature has values for, as `gen:` elements.
pub fn to_properties(
    attributes: &Attributes,
    entries: &[GenericAttribute],
) -> (Vec<XmlProperty>, Vec<Skip>) {
    let mut properties = Vec::new();
    let mut skipped = Vec::new();
    for entry in entries {
        let Some(value) = attributes.get(entry.attribute.as_str()) else {
            continue;
        };
        match typed(value, entry.uom.as_deref()) {
            Typed::Nothing => {}
            Typed::Unsupported => skipped.push(Skip::NotPlaced),
            Typed::Value { element, text } => {
                let mut value = XmlProperty::new("gen", "value");
                value.text = Some(text);
                if let Some(uom) = &entry.uom {
                    value.attrs.push(("uom".to_owned(), uom.clone()));
                }
                let mut property = XmlProperty::new("gen", element);
                property.attrs.push(("name".to_owned(), entry.name.clone()));
                property.children.push(value);
                properties.push(property);
            }
        }
    }
    (properties, skipped)
}

enum Typed {
    Nothing,
    Unsupported,
    Value { element: &'static str, text: String },
}

fn typed(value: &AttributeValue, uom: Option<&str>) -> Typed {
    let value_of = |element, text| Typed::Value { element, text };
    match (value, uom) {
        (AttributeValue::Null, _) => Typed::Nothing,
        (AttributeValue::String(text), _) if text.trim().is_empty() => Typed::Nothing,
        (AttributeValue::Number(number), Some(_)) => {
            value_of("measureAttribute", format_number(number))
        }
        (AttributeValue::String(text), Some(_)) if text.trim().parse::<f64>().is_ok() => {
            value_of("measureAttribute", text.trim().to_owned())
        }
        (_, Some(_)) => Typed::Unsupported,
        (AttributeValue::Number(number), None) if number.is_i64() || number.is_u64() => {
            value_of("intAttribute", format_number(number))
        }
        (AttributeValue::Number(number), None) => {
            value_of("doubleAttribute", format_number(number))
        }
        (AttributeValue::String(text), None) => value_of("stringAttribute", text.clone()),
        (AttributeValue::Bool(flag), None) => value_of("stringAttribute", flag.to_string()),
        (AttributeValue::DateTime(datetime), None) => match datetime {
            DateTime::NaiveDate(_) => value_of("dateAttribute", datetime.to_raw()),
            _ => value_of("stringAttribute", datetime.to_raw()),
        },
        (AttributeValue::Array(_) | AttributeValue::Map(_) | AttributeValue::Bytes(_), None) => {
            Typed::Unsupported
        }
    }
}

/// Fold computed attributes into a feature's round-tripped properties: one
/// replaces a `gen:` attribute of the same name where it stands, so running an
/// enrichment twice does not write the value twice; the rest follow in order.
pub fn merge(properties: &mut Vec<XmlProperty>, computed: Vec<XmlProperty>) {
    for property in computed {
        let name = property.attr("name").map(str::to_owned);
        let existing = properties
            .iter_mut()
            .find(|held| held.prefix == "gen" && held.attr("name").map(str::to_owned) == name);
        match existing {
            Some(held) => *held = property,
            None => properties.push(property),
        }
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_types::{Attribute, AttributeValue, Attributes};
    use serde_json::json;

    use super::*;

    fn entry(name: &str, attribute: &str, uom: Option<&str>) -> GenericAttribute {
        GenericAttribute {
            name: name.to_owned(),
            attribute: Attribute::new(attribute.to_owned()),
            uom: uom.map(str::to_owned),
        }
    }

    fn attrs(value: serde_json::Value) -> Attributes {
        match AttributeValue::from(value) {
            AttributeValue::Map(map) => map,
            other => panic!("expected a map, got {other:?}"),
        }
    }

    fn written(value: serde_json::Value, uom: Option<&str>) -> (Vec<XmlProperty>, Vec<Skip>) {
        to_properties(&attrs(json!({ "v": value })), &[entry("n", "v", uom)])
    }

    fn kind(value: serde_json::Value, uom: Option<&str>) -> String {
        let (properties, _) = written(value, uom);
        properties[0].local.clone()
    }

    #[test]
    fn the_element_follows_the_value() {
        assert_eq!(kind(json!(1234.5), Some("kWh")), "measureAttribute");
        assert_eq!(kind(json!("1234.5"), Some("kWh")), "measureAttribute");
        assert_eq!(kind(json!(3), None), "intAttribute");
        assert_eq!(kind(json!(3.5), None), "doubleAttribute");
        assert_eq!(kind(json!("text"), None), "stringAttribute");
        assert_eq!(kind(json!(true), None), "stringAttribute");
    }

    #[test]
    fn a_measure_writes_name_value_and_unit() {
        let (properties, skipped) = written(json!(1234.5), Some("kWh"));
        let p = &properties[0];
        assert_eq!((p.prefix.as_str(), p.attr("name")), ("gen", Some("n")));
        assert_eq!(p.children[0].local, "value");
        assert_eq!(p.children[0].attr("uom"), Some("kWh"));
        assert_eq!(p.children[0].text.as_deref(), Some("1234.5"));
        assert!(skipped.is_empty());
    }

    #[test]
    fn dates_and_datetimes() {
        let date = AttributeValue::DateTime("2024-03-15".parse().unwrap());
        let instant = AttributeValue::DateTime("2024-03-15T09:30:00Z".parse().unwrap());
        let mut map = Attributes::new();
        map.insert(Attribute::new("d".to_owned()), date);
        map.insert(Attribute::new("t".to_owned()), instant);
        let (properties, _) = to_properties(&map, &[entry("d", "d", None), entry("t", "t", None)]);
        assert_eq!(properties[0].local, "dateAttribute");
        assert_eq!(
            properties[0].children[0].text.as_deref(),
            Some("2024-03-15")
        );
        assert_eq!(properties[1].local, "stringAttribute");
    }

    #[test]
    fn missing_or_null_writes_nothing_and_unsupported_values_are_reported() {
        let (properties, skipped) = to_properties(
            &attrs(json!({ "n": null, "m": { "a": 1 }, "s": "abc" })),
            &[
                entry("a", "missing", None),
                entry("b", "n", None),
                entry("c", "m", None),
                entry("d", "s", Some("m")),
            ],
        );
        assert!(properties.is_empty());
        assert_eq!(skipped, vec![Skip::NotPlaced, Skip::NotPlaced]);
    }

    #[test]
    fn validation_rejects_blank_and_duplicate_entries() {
        assert!(validate(&[entry("a", "x", None), entry("b", "y", Some("m"))]).is_ok());
        assert!(validate(&[entry(" ", "x", None)])
            .unwrap_err()
            .contains("genericAttributes[0].name"));
        assert!(validate(&[entry("a", " ", None)])
            .unwrap_err()
            .contains("genericAttributes[0].attribute"));
        assert!(validate(&[entry("a", "x", Some(""))])
            .unwrap_err()
            .contains("genericAttributes[0].uom"));
        assert!(validate(&[entry("a", "x", None), entry("a", "y", None)])
            .unwrap_err()
            .contains("`a`"));
    }

    #[test]
    fn a_computed_value_replaces_a_round_tripped_attribute_of_the_same_name() {
        let mut round_trip = XmlProperty::new("gen", "stringAttribute");
        round_trip.attrs.push(("name".to_owned(), "n".to_owned()));
        let other = XmlProperty::new("bldg", "class");
        let mut properties = vec![other.clone(), round_trip];
        let (computed, _) = written(json!(2.5), Some("m"));
        let fresh = written(json!("x"), None).0.into_iter().map(|mut p| {
            p.attrs[0].1 = "fresh".to_owned();
            p
        });
        merge(&mut properties, computed.into_iter().chain(fresh).collect());
        assert_eq!(properties[0], other);
        assert_eq!(properties[1].local, "measureAttribute");
        assert_eq!(properties[2].attr("name"), Some("fresh"));
        assert_eq!(properties.len(), 3);
    }
}
