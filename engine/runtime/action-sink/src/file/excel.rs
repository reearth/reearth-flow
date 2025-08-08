use std::path::Path;
use std::sync::Arc;

use indexmap::IndexMap;
use reearth_flow_types::Attribute;
use rust_xlsxwriter::{Format, FormatAlign, FormatUnderline, Formula, Url, Workbook, Worksheet};

use reearth_flow_storage::resolve::StorageResolver;

use reearth_flow_common::uri::Uri;

use reearth_flow_types::{AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # ExcelWriter Parameters
/// 
/// Configuration for writing features to Microsoft Excel format.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExcelWriterParam {
    pub(super) sheet_name: Option<String>,
}

pub(super) fn write_excel(
    output: &Uri,
    params: &ExcelWriterParam,
    features: &[Feature],
    storage_resolver: &Arc<StorageResolver>,
) -> Result<(), crate::errors::SinkError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let sheet_name = params
        .sheet_name
        .clone()
        .unwrap_or_else(|| "Sheet1".to_string());
    worksheet
        .set_name(sheet_name)
        .map_err(crate::errors::SinkError::file_writer)?;

    let mut title_map = std::collections::HashMap::new();
    if let Some(first_row) = features.first() {
        for (col_num, (key, _)) in first_row.iter().enumerate() {
            worksheet
                .write_string_with_format(
                    0,
                    col_num as u16,
                    key.clone().into_inner(),
                    &Default::default(),
                )
                .map_err(crate::errors::SinkError::file_writer)?;
            title_map.insert(key.clone().into_inner(), col_num);
        }
    }

    let row_index = 1;
    for (row_num, row) in features.iter().enumerate() {
        for (key, value) in row.iter() {
            match title_map.get(&key.clone().into_inner()) {
                Some(&col_num) => {
                    write_cell_value(worksheet, row_num + row_index, col_num, value)?;
                    write_cell_formatting(
                        worksheet,
                        row_num + row_index,
                        col_num,
                        key.clone().into_inner(),
                        &row.attributes,
                    )?;
                    write_cell_formula(
                        worksheet,
                        row_num + row_index,
                        col_num,
                        key.clone().into_inner(),
                        &row.attributes,
                    )?;
                    write_cell_hyperlink(
                        worksheet,
                        row_num + row_index,
                        col_num,
                        key.clone().into_inner(),
                        &row.attributes,
                    )?;
                }
                None => {
                    return Err(crate::errors::SinkError::file_writer(format!(
                        "Key '{}' not found in title_map",
                        key.clone().into_inner()
                    )));
                }
            }
        }
    }
    let buf = workbook
        .save_to_buffer()
        .map_err(crate::errors::SinkError::file_writer)?;

    let storage = storage_resolver
        .resolve(output)
        .map_err(crate::errors::SinkError::file_writer)?;
    let uri_path = output.path();
    let path = Path::new(&uri_path);

    storage
        .put_sync(path, bytes::Bytes::from(buf))
        .map_err(crate::errors::SinkError::file_writer)?;

    Ok(())
}

fn write_cell_value(
    worksheet: &mut Worksheet,
    row: usize,
    col: usize,
    value: &AttributeValue,
) -> Result<(), crate::errors::SinkError> {
    let cell_value = match value {
        AttributeValue::String(s) => s.clone(),
        AttributeValue::Number(n) => n.to_string(),
        _ => "".to_string(),
    };

    worksheet
        .write_string_with_format(row as u32, col as u16, &cell_value, &Default::default())
        .map_err(crate::errors::SinkError::file_writer)?;

    Ok(())
}

fn write_cell_formatting(
    worksheet: &mut Worksheet,
    row: usize,
    col: usize,
    key: String,
    row_data: &IndexMap<Attribute, AttributeValue>,
) -> Result<(), crate::errors::SinkError> {
    if let Some(AttributeValue::String(formatting_str)) =
        row_data.get(&Attribute::new(format!("{key}.formatting")))
    {
        let format = parse_formatting(formatting_str.as_str())?;
        worksheet
            .write_string_with_format(row as u32, col as u16, "", &format)
            .map_err(crate::errors::SinkError::file_writer)?;
    }

    Ok(())
}

fn write_cell_formula(
    worksheet: &mut Worksheet,
    row: usize,
    col: usize,
    key: String,
    row_data: &IndexMap<Attribute, AttributeValue>,
) -> Result<(), crate::errors::SinkError> {
    if let Some(AttributeValue::String(formula_str)) =
        row_data.get(&Attribute::new(format!("{key}.formula")))
    {
        worksheet
            .write_formula(row as u32, col as u16, Formula::new(formula_str))
            .map_err(crate::errors::SinkError::file_writer)?;
    }

    Ok(())
}

fn write_cell_hyperlink(
    worksheet: &mut Worksheet,
    row: usize,
    col: usize,
    key: String,
    row_data: &IndexMap<Attribute, AttributeValue>,
) -> Result<(), crate::errors::SinkError> {
    if let Some(AttributeValue::String(hyperlink_str)) =
        row_data.get(&Attribute::new(format!("{key}.hyperlink")))
    {
        worksheet
            .write_url(row as u32, col as u16, Url::new(hyperlink_str))
            .map_err(crate::errors::SinkError::file_writer)?;
    }

    Ok(())
}

#[allow(dead_code)]
fn write_map_entry(
    worksheet: &mut Worksheet,
    row_index: &mut usize,
    key: String,
    value: AttributeValue,
) -> Result<(), crate::errors::SinkError> {
    worksheet
        .write_string_with_format(*row_index as u32, 0, &key, &Default::default())
        .map_err(crate::errors::SinkError::file_writer)?;

    match value {
        AttributeValue::String(s) => {
            worksheet
                .write_string_with_format(*row_index as u32, 1, &s, &Default::default())
                .map_err(crate::errors::SinkError::file_writer)?;
        }
        AttributeValue::Number(n) => {
            if let Some(num) = n.as_f64() {
                worksheet
                    .write_number(*row_index as u32, 1, num)
                    .map_err(crate::errors::SinkError::file_writer)?;
            } else {
                worksheet
                    .write_string_with_format(
                        *row_index as u32,
                        1,
                        n.to_string(),
                        &Default::default(),
                    )
                    .map_err(crate::errors::SinkError::file_writer)?;
            }
        }
        AttributeValue::Bool(b) => {
            worksheet
                .write_boolean(*row_index as u32, 1, b)
                .map_err(crate::errors::SinkError::file_writer)?;
        }
        AttributeValue::Array(arr) => {
            for (col_num, value) in arr.iter().enumerate() {
                match value {
                    AttributeValue::String(s) => {
                        worksheet
                            .write_string_with_format(
                                *row_index as u32,
                                col_num as u16 + 1,
                                s,
                                &Default::default(),
                            )
                            .map_err(crate::errors::SinkError::file_writer)?;
                    }
                    AttributeValue::Number(n) => {
                        if let Some(num) = n.as_f64() {
                            worksheet
                                .write_number(*row_index as u32, col_num as u16 + 1, num)
                                .map_err(crate::errors::SinkError::file_writer)?;
                        } else {
                            worksheet
                                .write_string_with_format(
                                    *row_index as u32,
                                    col_num as u16 + 1,
                                    n.to_string(),
                                    &Default::default(),
                                )
                                .map_err(crate::errors::SinkError::file_writer)?;
                        }
                    }
                    AttributeValue::Bool(b) => {
                        worksheet
                            .write_boolean(*row_index as u32, col_num as u16 + 1, *b)
                            .map_err(crate::errors::SinkError::file_writer)?;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    *row_index += 1;

    Ok(())
}

struct FormatBuilder {
    format: Format,
}

impl FormatBuilder {
    fn new() -> Self {
        Self {
            format: Format::new(),
        }
    }

    fn set_font_name(mut self, value: String) -> Self {
        self.format = self.format.set_font_name(value);
        self
    }

    fn set_font_size(mut self, size: f64) -> Self {
        self.format = self.format.set_font_size(size);
        self
    }

    fn set_font_color(mut self, value: &str) -> Self {
        self.format = self.format.set_font_color(value);
        self
    }

    fn set_bold(mut self) -> Self {
        self.format = self.format.set_bold();
        self
    }

    fn set_italic(mut self) -> Self {
        self.format = self.format.set_italic();
        self
    }

    fn set_underline(mut self, value: FormatUnderline) -> Self {
        self.format = self.format.set_underline(value);
        self
    }

    fn set_background_color(mut self, value: &str) -> Self {
        self.format = self.format.set_background_color(value);
        self
    }

    fn set_align(mut self, value: FormatAlign) -> Self {
        self.format = self.format.set_align(value);
        self
    }

    fn set_text_wrap(mut self) -> Self {
        self.format = self.format.set_text_wrap();
        self
    }

    fn build(self) -> Format {
        self.format
    }
}

fn parse_formatting(formatting_str: &str) -> Result<Format, crate::errors::SinkError> {
    let mut builder = FormatBuilder::new();
    for pair in formatting_str.split(';') {
        let mut parts = pair.splitn(2, ',');
        let key = parts
            .next()
            .ok_or_else(|| crate::errors::SinkError::file_writer("Invalid formatting key"))?;
        let value = parts
            .next()
            .ok_or_else(|| crate::errors::SinkError::file_writer("Invalid formatting value"))?;
        match key {
            "font" => builder = builder.set_font_name(value.to_string()),
            "size" => {
                let size = value
                    .parse::<f64>()
                    .map_err(|_| crate::errors::SinkError::file_writer("Invalid font size"))?;
                builder = builder.set_font_size(size);
            }
            "color" => builder = builder.set_font_color(value),
            "bold" => builder = builder.set_bold(),
            "italic" => builder = builder.set_italic(),
            "background_color" => builder = builder.set_background_color(value),
            "align" => {
                let align = ExcelFormatAlign::try_from(value)
                    .map_err(crate::errors::SinkError::file_writer)?;
                builder = builder.set_align(align.0);
            }
            "underline" => {
                let underline = ExcelFormatUnderline::try_from(value)
                    .map_err(crate::errors::SinkError::file_writer)?;
                builder = builder.set_underline(underline.0);
            }
            "wrap" => builder = builder.set_text_wrap(),
            _ => {
                return Err(crate::errors::SinkError::file_writer(
                    "Unknown formatting key",
                ))
            }
        };
    }
    Ok(builder.build())
}

// TODO: Row Format works with worksheet directly
// fn parse_row_formatting(row_formatting: &str, worksheet: &worksheet) -> Result<Format> {
//     let mut format = Format::new();
//     for pair in row_formatting.split(';') {
//         let mut parts = pair.splitn(2, ',');
//         let key = parts.next().ok_or_else(|| crate::errors::SinkError::file_writer("Invalid row formatting key"))?;
//         let value = parts.next().ok_or_else(|| crate::errors::SinkError::file_writer("Invalid row formatting value"))?;
//         match key {
//             "row_height" => format.set_row_height(value.parse().map_err(|_| crate::errors::SinkError::file_writer("Invalid row height"))?),
//             _ => return Err(crate::errors::SinkError::file_writer("Unknown row formatting key")),
//         }
//     }
//     Ok(format)
// }

pub struct ExcelFormatAlign(pub FormatAlign);

impl TryFrom<&str> for ExcelFormatAlign {
    type Error = crate::errors::SinkError;

    fn try_from(value: &str) -> Result<Self, crate::errors::SinkError> {
        match value {
            "General" => Ok(ExcelFormatAlign(FormatAlign::General)),
            "Left" => Ok(ExcelFormatAlign(FormatAlign::Left)),
            "Center" => Ok(ExcelFormatAlign(FormatAlign::Center)),
            "Right" => Ok(ExcelFormatAlign(FormatAlign::Right)),
            "Fill" => Ok(ExcelFormatAlign(FormatAlign::Fill)),
            "Justify" => Ok(ExcelFormatAlign(FormatAlign::Justify)),
            "CenterAcross" => Ok(ExcelFormatAlign(FormatAlign::CenterAcross)),
            "Distributed" => Ok(ExcelFormatAlign(FormatAlign::Distributed)),
            "Top" => Ok(ExcelFormatAlign(FormatAlign::Top)),
            "Bottom" => Ok(ExcelFormatAlign(FormatAlign::Bottom)),
            "VerticalCenter" => Ok(ExcelFormatAlign(FormatAlign::VerticalCenter)),
            "VerticalJustify" => Ok(ExcelFormatAlign(FormatAlign::VerticalJustify)),
            "VerticalDistributed" => Ok(ExcelFormatAlign(FormatAlign::VerticalDistributed)),
            _ => Err(crate::errors::SinkError::file_writer(format!(
                "Invalid alignment value: {value}"
            ))),
        }
    }
}

pub struct ExcelFormatUnderline(pub FormatUnderline);

impl TryFrom<&str> for ExcelFormatUnderline {
    type Error = crate::errors::SinkError;

    fn try_from(value: &str) -> Result<Self, crate::errors::SinkError> {
        match value {
            "None" => Ok(ExcelFormatUnderline(FormatUnderline::None)),
            "Single" => Ok(ExcelFormatUnderline(FormatUnderline::Single)),
            "Double" => Ok(ExcelFormatUnderline(FormatUnderline::Double)),
            "SingleAccounting" => Ok(ExcelFormatUnderline(FormatUnderline::SingleAccounting)),
            "DoubleAccounting" => Ok(ExcelFormatUnderline(FormatUnderline::DoubleAccounting)),
            _ => Err(crate::errors::SinkError::file_writer(format!(
                "Invalid underline value: {value}"
            ))),
        }
    }
}
