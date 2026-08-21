use indexmap::IndexMap;
use rust_xlsxwriter::{Formula, Url, Workbook, Worksheet};

use reearth_flow_types::{Attribute, AttributeValue, Feature};

/// Suffix marking an attribute that supplies a formula for another column's
/// cell rather than a value of its own.
const FORMULA_SUFFIX: &str = ".formula";
/// Suffix marking an attribute that supplies a hyperlink for another column's
/// cell rather than a value of its own.
const HYPERLINK_SUFFIX: &str = ".hyperlink";

pub(super) fn write_excel(
    output: &crate::SinkOutput,
    sheet_name: Option<&str>,
    features: &[Feature],
) -> Result<(), crate::errors::SinkError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name(sheet_name.unwrap_or("Sheet1"))
        .map_err(crate::errors::SinkError::excel_writer)?;

    let columns = columns_of(features);
    for (col_num, key) in columns.keys().enumerate() {
        worksheet
            .write_string(0, col_num as u16, key)
            .map_err(crate::errors::SinkError::excel_writer)?;
    }

    for (row_num, feature) in features.iter().enumerate() {
        let row = row_num as u32 + 1;
        for (key, value) in feature.iter() {
            let key = key.clone().into_inner();
            // Companion attributes have no column of their own; they are read
            // when the column they belong to is written.
            let Some(&col) = columns.get(&key) else {
                continue;
            };
            write_cell(worksheet, row, col as u16, &key, value, &feature.attributes)?;
        }
    }

    let buf = workbook
        .save_to_buffer()
        .map_err(crate::errors::SinkError::excel_writer)?;

    output
        .write(bytes::Bytes::from(buf))
        .map_err(crate::errors::SinkError::excel_writer)?;

    Ok(())
}

/// The sheet's columns, in the order the features first mention them.
///
/// Every feature contributes its keys, not just the first one: an attribute
/// that only some features carry is a column like any other, and dropping the
/// later ones would silently lose data. Companion attributes are excluded —
/// they configure another column's cell and are not data themselves.
fn columns_of(features: &[Feature]) -> IndexMap<String, usize> {
    let mut columns = IndexMap::new();
    for feature in features {
        for (key, _) in feature.iter() {
            let key = key.clone().into_inner();
            if is_companion(&key) {
                continue;
            }
            let next = columns.len();
            columns.entry(key).or_insert(next);
        }
    }
    columns
}

/// Whether the attribute configures another column's cell rather than holding
/// a value of its own.
fn is_companion(key: &str) -> bool {
    [FORMULA_SUFFIX, HYPERLINK_SUFFIX]
        .iter()
        .any(|suffix| key.ends_with(suffix))
}

/// Write one cell: a formula or a hyperlink when the column's companion
/// attributes ask for one, and otherwise the value in its own type.
fn write_cell(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    key: &str,
    value: &AttributeValue,
    attributes: &IndexMap<Attribute, AttributeValue>,
) -> Result<(), crate::errors::SinkError> {
    if let Some(AttributeValue::String(formula)) = companion(attributes, key, FORMULA_SUFFIX) {
        worksheet
            .write_formula(row, col, Formula::new(formula))
            .map_err(crate::errors::SinkError::excel_writer)?;
        return Ok(());
    }
    if let Some(AttributeValue::String(url)) = companion(attributes, key, HYPERLINK_SUFFIX) {
        worksheet
            .write_url(row, col, Url::new(url))
            .map_err(crate::errors::SinkError::excel_writer)?;
        return Ok(());
    }

    // Numbers and booleans are written in their own type rather than
    // stringified, so the sheet can sum, sort and filter them.
    match value {
        AttributeValue::String(s) => worksheet.write_string(row, col, s),
        AttributeValue::Number(n) => match n.as_f64() {
            Some(n) => worksheet.write_number(row, col, n),
            None => worksheet.write_string(row, col, n.to_string()),
        },
        AttributeValue::Bool(b) => worksheet.write_boolean(row, col, *b),
        AttributeValue::Null => worksheet.write_string(row, col, ""),
        // A cell holds one value, so a nested array or map is written as its
        // JSON rather than silently dropped.
        other => worksheet.write_string(row, col, json_text(other)),
    }
    .map_err(crate::errors::SinkError::excel_writer)?;

    Ok(())
}

/// The companion attribute `<key><suffix>`, when the feature carries one.
fn companion<'a>(
    attributes: &'a IndexMap<Attribute, AttributeValue>,
    key: &str,
    suffix: &str,
) -> Option<&'a AttributeValue> {
    attributes.get(&Attribute::new(format!("{key}{suffix}")))
}

/// A composite value as compact JSON.
fn json_text(value: &AttributeValue) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feature(pairs: &[(&str, AttributeValue)]) -> Feature {
        let mut attributes = IndexMap::new();
        for (key, value) in pairs {
            attributes.insert(Attribute::new(*key), value.clone());
        }
        Feature::new_with_attributes(attributes)
    }

    #[test]
    fn columns_span_every_feature_not_only_the_first() {
        let features = vec![
            feature(&[("a", AttributeValue::String("1".into()))]),
            feature(&[("b", AttributeValue::String("2".into()))]),
        ];
        let columns = columns_of(&features);
        assert_eq!(columns.get("a"), Some(&0));
        assert_eq!(columns.get("b"), Some(&1));
    }

    #[test]
    fn companion_attributes_get_no_column_of_their_own() {
        let features = vec![feature(&[
            ("a", AttributeValue::String("1".into())),
            ("a.formula", AttributeValue::String("=1+1".into())),
            (
                "a.hyperlink",
                AttributeValue::String("https://e.test".into()),
            ),
        ])];
        let columns = columns_of(&features);
        assert_eq!(columns.len(), 1);
        assert_eq!(columns.get("a"), Some(&0));
    }
}
