use std::collections::HashSet;

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
    for (key, &col) in &columns {
        worksheet
            .write_string(0, col as u16, key)
            .map_err(crate::errors::SinkError::excel_writer)?;
    }

    for (row_num, feature) in features.iter().enumerate() {
        let row = row_num as u32 + 1;
        for (col, cell) in row_of(&feature.attributes, &columns) {
            write_cell(worksheet, row, col, cell)?;
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
    let keys: Vec<String> = features
        .iter()
        .flat_map(|feature| feature.iter().map(|(key, _)| key.clone().into_inner()))
        .collect();
    let all: HashSet<&str> = keys.iter().map(String::as_str).collect();

    let mut columns = IndexMap::new();
    for key in &keys {
        if is_companion(key, &all) {
            continue;
        }
        let next = columns.len();
        columns.entry(key.clone()).or_insert(next);
    }
    columns
}

/// Whether the attribute configures another column's cell rather than holding a
/// value of its own.
///
/// The column it configures has to exist: an attribute that merely happens to
/// end in `.formula` with nothing named after it is ordinary data, and giving it
/// no column would drop it silently.
fn is_companion(key: &str, all_keys: &HashSet<&str>) -> bool {
    [FORMULA_SUFFIX, HYPERLINK_SUFFIX]
        .iter()
        .filter_map(|suffix| key.strip_suffix(suffix))
        .any(|base| all_keys.contains(base))
}

/// What a column's cell holds for one feature.
#[derive(Debug, PartialEq)]
enum Cell<'a> {
    /// The column's `.formula` companion asked for a formula.
    Formula(&'a str),
    /// The column's `.hyperlink` companion asked for a link.
    Hyperlink(&'a str),
    /// The feature's own value for the column.
    Value(&'a AttributeValue),
    /// The feature has nothing for this column.
    Empty,
}

/// What the column named `key` holds for a feature carrying `attributes`.
///
/// A companion wins over the plain value, and it applies whether or not the
/// feature also carries that value: a formula is usually the cell's content
/// rather than an annotation on it.
fn cell_of<'a>(attributes: &'a IndexMap<Attribute, AttributeValue>, key: &str) -> Cell<'a> {
    if let Some(AttributeValue::String(formula)) = companion(attributes, key, FORMULA_SUFFIX) {
        return Cell::Formula(formula);
    }
    if let Some(AttributeValue::String(url)) = companion(attributes, key, HYPERLINK_SUFFIX) {
        return Cell::Hyperlink(url);
    }
    match attributes.get(&Attribute::new(key)) {
        Some(value) => Cell::Value(value),
        None => Cell::Empty,
    }
}

/// One feature's row: what every column holds for it, in column order.
///
/// Walks the COLUMNS, not the feature's own attributes. A feature can carry
/// `a.formula` without carrying `a` — a formula is usually the cell's content
/// rather than an annotation on a value beside it — and walking the feature
/// would never reach column `a` to write it.
fn row_of<'a>(
    attributes: &'a IndexMap<Attribute, AttributeValue>,
    columns: &IndexMap<String, usize>,
) -> Vec<(u16, Cell<'a>)> {
    columns
        .iter()
        .map(|(key, &col)| (col as u16, cell_of(attributes, key)))
        .collect()
}

/// Write one cell in the type its content calls for.
fn write_cell(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    cell: Cell<'_>,
) -> Result<(), crate::errors::SinkError> {
    match cell {
        Cell::Formula(formula) => worksheet.write_formula(row, col, Formula::new(formula)),
        Cell::Hyperlink(url) => worksheet.write_url(row, col, Url::new(url)),
        // Numbers and booleans are written in their own type rather than
        // stringified, so the sheet can sum, sort and filter them.
        Cell::Value(AttributeValue::String(s)) => worksheet.write_string(row, col, s),
        Cell::Value(AttributeValue::Number(n)) => match n.as_f64() {
            Some(n) => worksheet.write_number(row, col, n),
            None => worksheet.write_string(row, col, n.to_string()),
        },
        Cell::Value(AttributeValue::Bool(b)) => worksheet.write_boolean(row, col, *b),
        Cell::Value(AttributeValue::Null) => worksheet.write_string(row, col, ""),
        // A cell holds one value, so a nested array or map is written as its
        // JSON rather than silently dropped.
        Cell::Value(other) => worksheet.write_string(row, col, json_text(other)),
        // Nothing to write leaves the cell untouched rather than blanking it.
        Cell::Empty => return Ok(()),
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
    fn a_formula_fills_its_column_even_with_no_value_beside_it() {
        // A formula is usually the cell's content, so a feature supplying only
        // `a.formula` must still fill column `a`. Building the row from the
        // feature's own attributes instead of from the columns skipped this row
        // entirely, dropping the formula with no trace.
        let with_value = feature(&[("a", AttributeValue::String("1".into()))]);
        let formula_only = feature(&[("a.formula", AttributeValue::String("=1+1".into()))]);
        let features = vec![with_value.clone(), formula_only.clone()];
        let columns = columns_of(&features);
        assert_eq!(columns.len(), 1, "`a.formula` is a companion, not a column");

        assert_eq!(
            row_of(&formula_only.attributes, &columns),
            vec![(0, Cell::Formula("=1+1"))]
        );
        assert_eq!(
            row_of(&with_value.attributes, &columns),
            vec![(0, Cell::Value(&AttributeValue::String("1".into())))]
        );
    }

    #[test]
    fn every_row_covers_every_column() {
        // Each row must have one entry per column regardless of which
        // attributes the feature happens to carry, so a sparse feature cannot
        // shift later columns or skip them.
        let features = vec![
            feature(&[("a", AttributeValue::String("1".into()))]),
            feature(&[("c", AttributeValue::String("3".into()))]),
            feature(&[("b", AttributeValue::String("2".into()))]),
        ];
        let columns = columns_of(&features);
        assert_eq!(columns.len(), 3);
        for f in &features {
            let row = row_of(&f.attributes, &columns);
            assert_eq!(row.len(), 3);
            assert_eq!(
                row.iter().map(|(col, _)| *col).collect::<Vec<_>>(),
                vec![0, 1, 2]
            );
            assert_eq!(row.iter().filter(|(_, c)| *c == Cell::Empty).count(), 2);
        }
    }

    #[test]
    fn a_hyperlink_fills_its_column_even_with_no_value_beside_it() {
        let f = feature(&[(
            "site.hyperlink",
            AttributeValue::String("https://e.test".into()),
        )]);
        // `site` is not a column here, but the resolution is what matters: were
        // any feature to carry `site`, this row would still link.
        assert_eq!(
            cell_of(&f.attributes, "site"),
            Cell::Hyperlink("https://e.test")
        );
    }

    #[test]
    fn a_companion_wins_over_the_value_beside_it() {
        let f = feature(&[
            ("a", AttributeValue::String("stale".into())),
            ("a.formula", AttributeValue::String("=1+1".into())),
        ]);
        assert_eq!(cell_of(&f.attributes, "a"), Cell::Formula("=1+1"));
    }

    #[test]
    fn a_column_the_feature_has_nothing_for_is_empty() {
        let f = feature(&[("a", AttributeValue::String("1".into()))]);
        assert_eq!(cell_of(&f.attributes, "b"), Cell::Empty);
    }

    #[test]
    fn a_companion_naming_no_existing_column_is_ordinary_data() {
        // Nothing is called `b`, so `b.formula` configures nothing and must keep
        // a column of its own rather than vanishing.
        let features = vec![feature(&[
            ("a", AttributeValue::String("1".into())),
            ("b.formula", AttributeValue::String("=1+1".into())),
        ])];
        let columns = columns_of(&features);
        assert_eq!(columns.len(), 2);
        assert_eq!(columns.get("b.formula"), Some(&1));
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
