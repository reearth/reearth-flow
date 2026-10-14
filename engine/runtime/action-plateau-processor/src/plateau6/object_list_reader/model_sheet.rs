//! Pass 1: reads the model sheets of the workbook into rows, without deciding
//! what they are used for.

use std::ops::Range as Columns;

use bytes::Bytes;
use calamine::{Data, Range, Reader};

use crate::object_list::{is_cell_marked, open_workbook, Error, Result};

/// A1 of a model sheet (`01. 建築物モデル`, `08. 災害リスクモデル`, ...).
/// Other sheets are skipped.
const MODEL_SHEET_HEADER: &str = "モデルの接頭辞";
/// Rows 1-2 hold the column headers.
const HEADER_ROWS: usize = 2;

// 0-based columns.
/// A: モデルの接頭辞
const MODEL_PREFIX: usize = 0;
/// B: 地物名
const FEATURE_TYPE: usize = 1;
/// C-G: 属性名／関連役割名
const ATTRIBUTE_NAMES: Columns<usize> = 2..7;
/// H: 主題属性、空間属性、関連役割の区分
const CATEGORY: usize = 7;
/// J: 作成対象
const CREATION_TARGET: usize = 9;
/// O: 不明な場合に不明値を入力する。
const UNKNOWN_VALUE: usize = 14;

const ATTRIBUTE_DEPTH: usize = ATTRIBUTE_NAMES.end - ATTRIBUTE_NAMES.start;

#[derive(Debug)]
pub(super) struct ModelSheet {
    pub(super) rows: Vec<AttributeRow>,
}

/// A data row of a model sheet. Columns B-G are filled in from the rows above,
/// since the sheet writes a feature type or a parent attribute only on its
/// first row.
#[derive(Debug)]
pub(super) struct AttributeRow {
    /// A: モデルの接頭辞. `fld/tnm/htd/ifld/rfld` for the flood packages, and
    /// empty in `21. 都市オブジェクトグループモデル`.
    pub(super) model_prefix: String,
    /// B: 地物名
    pub(super) feature_type: String,
    /// C-G: 属性名／関連役割名, outermost first. A parent cell reads
    /// `property.Type`, and parentheses mark attributes outside the standard
    /// spec.
    pub(super) attribute_names: Vec<String>,
    /// H: 主題属性、空間属性、関連役割の区分 (`主題`, `空間` or `関連役割`)
    pub(super) category: String,
    /// J: 作成対象 is marked.
    pub(super) is_creation_target: bool,
    /// O: 不明な場合に不明値を入力する。 is marked, so the attribute must be
    /// present.
    pub(super) is_required: bool,
}

impl AttributeRow {
    /// The row is an attribute or association role the data is checked for.
    pub(super) fn is_checked(&self) -> bool {
        self.is_creation_target && ["主題", "関連役割"].contains(&self.category.as_str())
    }

    /// Packages the row applies to. The flood packages share one row.
    pub(super) fn packages(&self) -> Vec<String> {
        if self.model_prefix.starts_with("fld/") {
            return ["fld", "tnm", "htd", "ifld", "rfld"]
                .iter()
                .map(|package| package.to_string())
                .collect();
        }
        if self.model_prefix.is_empty() {
            return vec![];
        }
        vec![self.model_prefix.clone()]
    }
}

pub(super) fn read_model_sheets(bytes: Bytes) -> Result<Vec<ModelSheet>> {
    let mut workbook = open_workbook(bytes)?;
    let mut sheets = Vec::new();
    for name in workbook.sheet_names() {
        let range = workbook
            .worksheet_range(&name)
            .map_err(|e| Error::Parse(format!("{e:?}")))?;
        if is_model_sheet(&range) {
            sheets.push(ModelSheet::from_range(&range));
        }
    }
    if sheets.is_empty() {
        return Err(Error::Parse("No model sheet found".to_string()));
    }
    Ok(sheets)
}

fn is_model_sheet(range: &Range<Data>) -> bool {
    matches!(range.get_value((0, 0)), Some(Data::String(s)) if s.trim() == MODEL_SHEET_HEADER)
}

impl ModelSheet {
    fn from_range(range: &Range<Data>) -> Self {
        let mut feature_type = String::new();
        let mut attribute_names: [Option<String>; ATTRIBUTE_DEPTH] = Default::default();
        let mut rows = Vec::new();
        for cells in range.rows().skip(HEADER_ROWS) {
            let cell = |column: usize| {
                cells
                    .get(column)
                    .map(|data| data.to_string())
                    .unwrap_or_default()
            };

            // A new feature type starts its attribute hierarchy over, and a new
            // attribute clears the levels below it.
            let feature_type_cell = cell(FEATURE_TYPE);
            if !feature_type_cell.is_empty() {
                feature_type = feature_type_cell;
                attribute_names.fill(None);
            }
            for (level, column) in ATTRIBUTE_NAMES.enumerate() {
                let name = cell(column);
                if !name.is_empty() {
                    attribute_names[level] = Some(name);
                    attribute_names[level + 1..].fill(None);
                }
            }

            rows.push(AttributeRow {
                model_prefix: cell(MODEL_PREFIX),
                feature_type: feature_type.clone(),
                attribute_names: attribute_names.iter().flatten().cloned().collect(),
                category: cell(CATEGORY),
                is_creation_target: is_cell_marked(&cell(CREATION_TARGET)),
                is_required: is_cell_marked(&cell(UNKNOWN_VALUE)),
            });
        }
        Self { rows }
    }
}
