use std::{collections::HashMap, str::FromStr};
use std::path::Path;
use std::sync::Arc;

use reearth_flow_action::{
    error, Action, ActionContext, ActionDataframe, ActionResult, Result, ActionValue, DEFAULT_PORT,
};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use rust_xlsxwriter::{worksheet, Format, FormatAlign, FormatUnderline, Formula, Url, Workbook};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExcelFileWriter {
    file_path: String,
    worksheet_name: String,
    template_file: Option<String>,
    template_sheet: Option<String>,
    protection_level: Option<String>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "ExcelFileWriter")]
impl Action for ExcelFileWriter {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        write_excel(inputs, self, storage_resolver).await?;
        let mut output: ActionDataframe = HashMap::new();
        let summary = vec![(
            "output".to_owned(),
            ActionValue::String(self.file_path.clone()),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();
        output.insert(DEFAULT_PORT.clone(), Some(ActionValue::Map(summary)));
        Ok(output)
    }
}

async fn write_excel(
    inputs: Option<ActionDataframe>,
    props: &ExcelFileWriter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<ActionValue> {
    let uri = Uri::from_str(&props.file_path).map_err(error::Error::input)?;
    let storage = storage_resolver.resolve(&uri).map_err(error::Error::input)?;
    let uri_path = uri.path(); 
    let path = Path::new(&uri_path);
    
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.set_name(props.worksheet_name.clone());

    // Set protection level if specified
    // if let Some(protection_level) = &props.protection_level {
    //     worksheet.set_protection(protection_level)?;
    // }

    let mut row_index = 0;

    if let Some(inputs) = inputs {
        for (port, data) in inputs {
            let data = match data {
                Some(data) => data,
                None => continue,
            };

            match data {
                ActionValue::Array(rows) => {
                    for (row_num, row) in rows.iter().enumerate() {
                        match row {
                            ActionValue::Map(row_data) => {
                                for (col_num, (key, value)) in row_data.iter().enumerate() {
                                    let cell_value = match value {
                                        ActionValue::String(s) => s.clone(),
                                        ActionValue::Number(n) => n.to_string(),
                                        _ => "".to_string(),
                                    };

                                    // Check for formatting
                                    if let Some(formatting) = row_data.get(&format!("{}.formatting", key)) {
                                        if let ActionValue::String(formatting_str) = formatting {
                                            let format = parse_formatting(formatting_str)?;
                                            worksheet.write_string_with_format(
                                                row_num as u32 + row_index as u32,
                                                col_num as u16,
                                                &cell_value,
                                                &format,
                                            );
                                        } else {
                                            worksheet.write_string_with_format(
                                                row_num as u32 + row_index as u32,
                                                col_num as u16,
                                                &cell_value,
                                                &Default::default(),
                                            );
                                        }
                                    } else {
                                        worksheet.write_string_with_format(
                                            row_num as u32 + row_index as u32,
                                            col_num as u16,
                                            &cell_value,
                                            &Default::default(),
                                        );
                                    }

                                    // Check for formula
                                    if let Some(formula) = row_data.get(&format!("{}.formula", key)) {
                                        if let ActionValue::String(formula_str) = formula {
                                            worksheet.write_formula(
                                                row_num as u32 + row_index as u32,
                                                col_num as u16,
                                                Formula::new(formula_str)
                                            );
                                        }
                                    }

                                    // Check for hyperlink
                                    if let Some(hyperlink) = row_data.get(&format!("{}.hyperlink", key)) {
                                        if let ActionValue::String(hyperlink_str) = hyperlink {
                                            worksheet.write_url(
                                                row_num as u32 + row_index as u32,
                                                col_num as u16,
                                                Url::new(hyperlink_str)
                                            );
                                        }
                                    }
                                }
                            }
                            _ => continue,
                        }

                        // Check for row formatting
                        // if let Some(ActionValue::String(row_formatting)) = row.get("xlsx_row_formatting") {
                        //     let format = parse_row_formatting(&row_formatting, &workbook)?;
                        //     worksheet.set_row_format(row_num as u32 + row_index as u32, Some(&format))?;
                        // }
                    }
                    row_index += rows.len();
                }
                ActionValue::Map(map) => {
                    for (key, value) in map {
                        match value {
                            ActionValue::String(s) => {
                                worksheet.write_string_with_format(
                                    row_index as u32,
                                    0,
                                    &key,
                                    &Default::default(),
                                );
                                worksheet.write_string_with_format(
                                    row_index as u32,
                                    1,
                                    &s,
                                    &Default::default(),
                                );
                                row_index += 1;
                            }
                            ActionValue::Number(n) => {
                                worksheet.write_string_with_format(
                                    row_index as u32,
                                    0,
                                    &key,
                                    &Default::default(),
                                );
                                if let Some(num) = n.as_f64() {
                                    worksheet.write_number(
                                        row_index as u32,
                                        1,
                                        num,
                                    );
                                } else {
                                    worksheet.write_string_with_format(
                                        row_index as u32,
                                        1,
                                        &n.to_string(),
                                        &Default::default(),
                                    );
                                }
                                row_index += 1;
                            }
                            ActionValue::Bool(b) => {
                                worksheet.write_string_with_format(
                                    row_index as u32,
                                    0,
                                    &key,
                                    &Default::default(),
                                );
                                worksheet.write_boolean(
                                    row_index as u32,
                                    1,
                                    b,
                                );
                                row_index += 1;
                            }
                            ActionValue::Array(arr) => {
                                worksheet.write_string_with_format(
                                    row_index as u32,
                                    0,
                                    &key,
                                    &Default::default(),
                                );
                                for (col_num, value) in arr.iter().enumerate() {
                                    match value {
                                        ActionValue::String(s) => {
                                            worksheet.write_string_with_format(
                                                row_index as u32,
                                                col_num as u16 + 1,
                                                s,
                                                &Default::default(),
                                            );
                                        }
                                        ActionValue::Number(n) => {
                                            if let Some(num) = n.as_f64() {
                                                worksheet.write_number(
                                                    row_index as u32,
                                                    col_num as u16 + 1,
                                                    num,
                                                );
                                            } else {
                                                worksheet.write_string_with_format(
                                                    row_index as u32,
                                                    col_num as u16 + 1,
                                                    &n.to_string(),
                                                    &Default::default(),
                                                );
                                            }
                                        }
                                        ActionValue::Bool(b) => {
                                            worksheet.write_boolean(
                                                row_index as u32,
                                                col_num as u16 + 1,
                                                *b,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                                row_index += 1;
                            }
                            _ => {}
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    let data = std::fs::read(path).map_err(error::Error::input)?;
    storage
        .put(path, bytes::Bytes::from(data))
        .await
        .map_err(error::Error::internal_runtime)?;

    Ok(ActionValue::Bool(true))
}

fn parse_formatting(formatting_str: &str) -> Result<Format> {
    let mut format = Format::new();
    for pair in formatting_str.split(';') {
        let mut parts = pair.splitn(2, ',');
        let key = parts.next().ok_or_else(|| error::Error::internal_runtime("Invalid formatting key"))?;
        let value = parts.next().ok_or_else(|| error::Error::internal_runtime("Invalid formatting value"))?;
        match key {
            "font" => format.set_font_name(value),
            "size" => format.set_font_size(value.parse().map_err(|_| error::Error::internal_runtime("Invalid font size".to_string()))),
            "color" => format.set_font_color(value),
            "bold" => format.set_bold(),
            "italic" => format.set_italic(),
            "underline" => format.set_underline(str_to_format_underline(value)?),            
            "background_color" => format.set_background_color(value),
            "align" => format.set_align(str_to_format_align(value)?),
            "wrap" => format.set_text_wrap(),
            _ => return Err(error::Error::internal_runtime("Unknown formatting key")),
        };
    }
    Ok(format)
}

// Row Format works with worksheet directly
// fn parse_row_formatting(row_formatting: &str, worksheet: &worksheet) -> Result<Format> {
//     let mut format = Format::new();
//     for pair in row_formatting.split(';') {
//         let mut parts = pair.splitn(2, ',');
//         let key = parts.next().ok_or_else(|| error::Error::internal_runtime("Invalid row formatting key"))?;
//         let value = parts.next().ok_or_else(|| error::Error::internal_runtime("Invalid row formatting value"))?;
//         match key {
//             "row_height" => format.set_row_height(value.parse().map_err(|_| error::Error::internal_runtime("Invalid row height"))?),
//             _ => return Err(error::Error::internal_runtime("Unknown row formatting key")),
//         }
//     }
//     Ok(format)
// }

fn str_to_format_underline(s: &str) -> Result<FormatUnderline, error::Error> {
    match s {
        "None" => Ok(FormatUnderline::None),
        "Single" => Ok(FormatUnderline::Single),
        "Double" => Ok(FormatUnderline::Double),
        "SingleAccounting" => Ok(FormatUnderline::SingleAccounting),
        "DoubleAccounting" => Ok(FormatUnderline::DoubleAccounting),
        _ => Err(error::Error::internal_runtime("Invalid underline value")),
    }
}

fn str_to_format_align(s: &str) -> Result<FormatAlign, error::Error> {
    match s {
        "General" => Ok(FormatAlign::General),
        "Left" => Ok(FormatAlign::Left),
        "Center" => Ok(FormatAlign::Center),
        "Right" => Ok(FormatAlign::Right),
        "Fill" => Ok(FormatAlign::Fill),
        "Justify" => Ok(FormatAlign::Justify),
        "CenterAcross" => Ok(FormatAlign::CenterAcross),
        "Distributed" => Ok(FormatAlign::Distributed),
        "Top" => Ok(FormatAlign::Top),
        "Bottom" => Ok(FormatAlign::Bottom),
        "VerticalCenter" => Ok(FormatAlign::VerticalCenter),
        "VerticalJustify" => Ok(FormatAlign::VerticalJustify),
        "VerticalDistributed" => Ok(FormatAlign::VerticalDistributed),
        _ => Err(error::Error::internal_runtime("Invalid alignment value")),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_action::ActionValue;

    #[tokio::test]
    async fn test_write_excel() {
        let inputs = Some(
            vec![(
                DEFAULT_PORT.clone(),
                Some(ActionValue::Array(vec![
                    ActionValue::Map(
                        vec![
                            ("field1".to_owned(), ActionValue::String("value1".to_owned())),
                            ("field2".to_owned(), ActionValue::Number(10.0.into())),
                            ("field1.hyperlink".to_owned(), ActionValue::String("https://www.example.com".to_owned())),
                            ("field1.formatting".to_owned(), ActionValue::String("font,Arial;size,12;color,#FF0000;bold,true".to_owned())),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                    ActionValue::Map(
                        vec![
                            ("field1".to_owned(), ActionValue::String("value2".to_owned())),
                            ("field2".to_owned(), ActionValue::Number(20.0.into())),
                            ("field2.formula".to_owned(), ActionValue::String("=SUM(A1:A2)".to_owned())),
                            ("xlsx_row_formatting".to_owned(), ActionValue::String("row_height,30".to_owned())),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                ])),
            )]
            .into_iter()
            .collect::<ActionDataframe>(),
        );
        let props = ExcelFileWriter {
            file_path: "ram:///root/output.xlsx".to_owned(),
            worksheet_name: "Sheet1".to_owned(),
            template_file: None,
            template_sheet: None,
            protection_level: Some("password".to_owned()),
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_excel(inputs, &props, resolver).await;
        assert!(result.is_ok());
    }
}