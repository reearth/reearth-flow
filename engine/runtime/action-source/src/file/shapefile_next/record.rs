//! The `.dbf` attribute table: one record per shape, read as feature attributes.

use indexmap::IndexMap;
use reearth_flow_common::datetime::{DateTime, NaiveDate};
use reearth_flow_types::{Attribute, AttributeValue};
use shapefile::dbase::{FieldValue, Record};

/// The attributes one record holds, in the order the table declares its fields.
///
/// A field with no value, and one whose value the field type cannot represent,
/// both read as [`AttributeValue::Null`] rather than being left out, so every
/// feature carries the same attributes.
pub(super) fn to_attributes(record: Record) -> IndexMap<Attribute, AttributeValue> {
    record
        .into_iter()
        .map(|(name, value)| (Attribute::new(name), to_attribute_value(value)))
        .collect()
}

/// The attribute value a field holds.
fn to_attribute_value(value: FieldValue) -> AttributeValue {
    match value {
        FieldValue::Character(Some(s)) => AttributeValue::String(s),
        FieldValue::Memo(s) => AttributeValue::String(s),
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

    // A memo is a string too long for a character field, not an absent value.
    #[test]
    fn a_memo_reads_as_a_string() {
        assert_eq!(
            to_attribute_value(FieldValue::Memo("long".into())),
            AttributeValue::String("long".into())
        );
    }

    #[test]
    fn every_numeric_field_type_reads_as_a_number() {
        for value in [
            FieldValue::Numeric(Some(1.5)),
            FieldValue::Float(Some(1.5)),
            FieldValue::Double(1.5),
            FieldValue::Currency(1.5),
        ] {
            assert_eq!(
                to_attribute_value(value),
                AttributeValue::Number(serde_json::Number::from_f64(1.5).unwrap())
            );
        }
        assert_eq!(
            to_attribute_value(FieldValue::Integer(7)),
            AttributeValue::Number(7.into())
        );
    }

    #[test]
    fn a_non_finite_number_has_no_counterpart() {
        assert_eq!(
            to_attribute_value(FieldValue::Numeric(Some(f64::NAN))),
            AttributeValue::Null
        );
        assert_eq!(
            to_attribute_value(FieldValue::Double(f64::INFINITY)),
            AttributeValue::Null
        );
    }

    // A date states no time of day, so it keeps its own type rather than becoming
    // an instant at an invented hour.
    #[test]
    fn a_date_reads_as_a_date() {
        assert_eq!(
            date(2025, 7, 17),
            AttributeValue::DateTime(DateTime::NaiveDate(
                NaiveDate::from_ymd_opt(2025, 7, 17).unwrap()
            ))
        );
    }

    #[test]
    fn an_absent_value_reads_as_null() {
        for value in [
            FieldValue::Character(None),
            FieldValue::Numeric(None),
            FieldValue::Float(None),
            FieldValue::Logical(None),
            FieldValue::Date(None),
        ] {
            assert_eq!(to_attribute_value(value), AttributeValue::Null);
        }
    }
}
