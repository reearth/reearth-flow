//! The `.dbf` attribute table: one record per shape, read as feature attributes.

use indexmap::IndexMap;
use reearth_flow_common::datetime::{DateTime, NaiveDate};
use reearth_flow_types::{Attribute, AttributeValue};
use shapefile::dbase::{FieldValue, Record};

/// One field of the attribute table, as far as reading its values goes.
pub(super) struct Field {
    /// The name the table declares.
    pub(super) name: String,
    /// Whether the field is numeric with no decimal places, so that its values
    /// read as integers.
    pub(super) integral: bool,
}

/// The attributes one record holds, in the order of `fields`, which is the order
/// the table declares them in.
///
/// A field with no value, and one whose value the field type cannot represent,
/// both read as [`AttributeValue::Null`] rather than being left out, so every
/// feature carries the same attributes.
pub(super) fn to_attributes(
    mut record: Record,
    fields: &[Field],
) -> IndexMap<Attribute, AttributeValue> {
    fields
        .iter()
        .map(|field| {
            let value = record
                .remove(&field.name)
                .map(|value| to_attribute_value(value, field.integral))
                .unwrap_or(AttributeValue::Null);
            (Attribute::new(field.name.clone()), value)
        })
        .collect()
}

/// The attribute value a field holds; a numeric one reads as an integer when the
/// field is `integral` and the value has an integer counterpart.
fn to_attribute_value(value: FieldValue, integral: bool) -> AttributeValue {
    match value {
        FieldValue::Character(Some(s)) => AttributeValue::String(s),
        FieldValue::Memo(s) => AttributeValue::String(s),
        FieldValue::Numeric(Some(n)) if integral => integer(n),
        FieldValue::Float(Some(f)) if integral => integer(f as f64),
        FieldValue::Numeric(Some(n)) => number(n),
        FieldValue::Float(Some(f)) => number(f as f64),
        FieldValue::Double(d) => number(d),
        FieldValue::Currency(c) => number(c),
        FieldValue::Integer(i) => AttributeValue::Number(i.into()),
        FieldValue::Logical(Some(b)) => AttributeValue::Bool(b),
        FieldValue::Date(Some(d)) => date(d.year(), d.month(), d.day()),
        FieldValue::DateTime(d) => datetime(d),
        FieldValue::Character(None)
        | FieldValue::Numeric(None)
        | FieldValue::Float(None)
        | FieldValue::Logical(None)
        | FieldValue::Date(None) => AttributeValue::Null,
    }
}

/// The largest magnitude every integer up to which an `f64` holds exactly.
const EXACT_INTEGER_LIMIT: f64 = 9007199254740992.0;

/// A zero-decimal numeric field's value: the integer it holds, or the number as
/// read for a value that is not one, or too wide to be held exactly.
fn integer(n: f64) -> AttributeValue {
    if n.fract() == 0.0 && n.abs() < EXACT_INTEGER_LIMIT {
        return AttributeValue::Number((n as i64).into());
    }
    number(n)
}

/// A numeric field's value. A non-finite number has no attribute counterpart, so
/// it reads as null rather than as some other number.
fn number(n: f64) -> AttributeValue {
    match serde_json::Number::from_f64(n) {
        Some(number) => AttributeValue::Number(number),
        None => AttributeValue::Null,
    }
}

/// A date field's value, which states no time of day and so no instant.
fn date(year: u32, month: u32, day: u32) -> AttributeValue {
    match i32::try_from(year)
        .ok()
        .and_then(|year| NaiveDate::from_ymd_opt(year, month, day))
    {
        Some(date) => AttributeValue::DateTime(DateTime::NaiveDate(date)),
        None => AttributeValue::Null,
    }
}

/// A datetime field's value. The format states no zone, so the instant is read
/// as UTC.
fn datetime(value: shapefile::dbase::DateTime) -> AttributeValue {
    match reearth_flow_common::datetime::try_from_unix_s(value.to_unix_timestamp()) {
        Ok(utc) => AttributeValue::DateTime(DateTime::Utc(utc)),
        Err(_) => AttributeValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn a_zero_decimal_numeric_field_reads_as_an_integer() {
        assert_eq!(
            to_attribute_value(FieldValue::Numeric(Some(2019.0)), true),
            AttributeValue::Number(2019.into())
        );
        assert_eq!(
            to_attribute_value(FieldValue::Numeric(Some(2019.0)), false),
            AttributeValue::Number(serde_json::Number::from_f64(2019.0).unwrap())
        );
        assert_eq!(
            to_attribute_value(FieldValue::Numeric(Some(1e300)), true),
            AttributeValue::Number(serde_json::Number::from_f64(1e300).unwrap())
        );
        assert_eq!(
            to_attribute_value(FieldValue::Numeric(Some(1.5)), true),
            AttributeValue::Number(serde_json::Number::from_f64(1.5).unwrap())
        );
    }

    #[test]
    fn attributes_follow_the_table_field_order() {
        let mut record = Record::default();
        record.insert("b".to_string(), FieldValue::Integer(2));
        record.insert("a".to_string(), FieldValue::Integer(1));
        record.insert("c".to_string(), FieldValue::Integer(3));
        let fields: Vec<Field> = ["c", "a", "b"]
            .into_iter()
            .map(|name| Field {
                name: name.to_string(),
                integral: true,
            })
            .collect();
        let attributes = to_attributes(record, &fields);
        let order: Vec<String> = attributes.keys().map(|k| k.inner()).collect();
        assert_eq!(
            order,
            vec!["c".to_string(), "a".to_string(), "b".to_string()]
        );
    }
}
