use std::path::Path;
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};

use rust_xlsxwriter::{
    Format, FormatAlign, FormatUnderline, Formula, ProtectionOptions, Url, Workbook, Worksheet,
};

use reearth_flow_storage::resolve::StorageResolver;
use serde::{Deserialize, Serialize};

use reearth_flow_common::uri::Uri;

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, ActionValue, AsyncAction, Result,
    DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExcelFileWriter {
    output: String,
    worksheet_name: String,
    template_file: Option<String>,
    template_sheet: Option<String>,
    protection_options: Option<ProtectionOptionsDTO>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "ExcelFileWriter")]
impl AsyncAction for ExcelFileWriter {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        write_excel(inputs, self, storage_resolver).await?;
        let output = vec![(
            "output".to_owned(),
            ActionValue::String(self.output.clone()),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();
        Ok([(DEFAULT_PORT.clone(), Some(ActionValue::Map(output)))].into())
    }
}

async fn write_excel(
    inputs: Option<ActionDataframe>,
    props: &ExcelFileWriter,
    storage_resolver: Arc<StorageResolver>,
) -> Result<ActionValue> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet
        .set_name(props.worksheet_name.clone())
        .map_err(Error::internal_runtime)?;

    if let Some(dto) = &props.protection_options {
        let protection_options = dto_to_protection_options(dto);
        worksheet.protect_with_options(&protection_options);
    }

    let mut row_index = 0;

    if let Some(inputs) = inputs {
        for (_port, data) in inputs {
            if let Some(data) = data {
                match data {
                    ActionValue::Array(rows) => {
                        for (row_num, row) in rows.iter().enumerate() {
                            if let ActionValue::Map(row_data) = row {
                                for (col_num, (key, value)) in row_data.iter().enumerate() {
                                    write_cell_value(
                                        worksheet,
                                        row_num + row_index,
                                        col_num,
                                        key,
                                        value,
                                    )?;
                                    write_cell_formatting(
                                        worksheet,
                                        row_num + row_index,
                                        col_num,
                                        key,
                                        row_data,
                                    )?;
                                    write_cell_formula(
                                        worksheet,
                                        row_num + row_index,
                                        col_num,
                                        key,
                                        row_data,
                                    )?;
                                    write_cell_hyperlink(
                                        worksheet,
                                        row_num + row_index,
                                        col_num,
                                        key,
                                        row_data,
                                    )?;
                                }
                            }
                        }
                        row_index += rows.len();
                    }
                    ActionValue::Map(map) => {
                        for (key, value) in map {
                            write_map_entry(worksheet, &mut row_index, key, value)?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let buf = workbook.save_to_buffer().map_err(Error::internal_runtime)?;

    let uri = Uri::from_str(&props.output).map_err(Error::input)?;
    let storage = storage_resolver.resolve(&uri).map_err(Error::input)?;
    let uri_path = uri.path();
    let path = Path::new(&uri_path);

    storage
        .put(path, bytes::Bytes::from(buf))
        .await
        .map_err(Error::internal_runtime)?;

    Ok(ActionValue::Bool(true))
}

fn write_cell_value(
    worksheet: &mut Worksheet,
    row: usize,
    col: usize,
    _key: &str,
    value: &ActionValue,
) -> Result<()> {
    let cell_value = match value {
        ActionValue::String(s) => s.clone(),
        ActionValue::Number(n) => n.to_string(),
        _ => "".to_string(),
    };

    worksheet
        .write_string_with_format(row as u32, col as u16, &cell_value, &Default::default())
        .map_err(Error::internal_runtime)?;

    Ok(())
}

fn write_cell_formatting(
    worksheet: &mut Worksheet,
    row: usize,
    col: usize,
    key: &str,
    row_data: &HashMap<String, ActionValue>,
) -> Result<()> {
    if let Some(ActionValue::String(formatting_str)) = row_data.get(&format!("{}.formatting", key))
    {
        let format = parse_formatting(formatting_str)?;
        worksheet
            .write_string_with_format(row as u32, col as u16, "", &format)
            .map_err(Error::internal_runtime)?;
    }

    Ok(())
}

fn write_cell_formula(
    worksheet: &mut Worksheet,
    row: usize,
    col: usize,
    key: &str,
    row_data: &HashMap<String, ActionValue>,
) -> Result<()> {
    if let Some(ActionValue::String(formula_str)) = row_data.get(&format!("{}.formula", key)) {
        worksheet
            .write_formula(row as u32, col as u16, Formula::new(formula_str))
            .map_err(Error::internal_runtime)?;
    }

    Ok(())
}

fn write_cell_hyperlink(
    worksheet: &mut Worksheet,
    row: usize,
    col: usize,
    key: &str,
    row_data: &HashMap<String, ActionValue>,
) -> Result<()> {
    if let Some(ActionValue::String(hyperlink_str)) = row_data.get(&format!("{}.hyperlink", key)) {
        worksheet
            .write_url(row as u32, col as u16, Url::new(hyperlink_str))
            .map_err(Error::internal_runtime)?;
    }

    Ok(())
}

fn write_map_entry(
    worksheet: &mut Worksheet,
    row_index: &mut usize,
    key: String,
    value: ActionValue,
) -> Result<()> {
    worksheet
        .write_string_with_format(*row_index as u32, 0, &key, &Default::default())
        .map_err(Error::internal_runtime)?;

    match value {
        ActionValue::String(s) => {
            worksheet
                .write_string_with_format(*row_index as u32, 1, &s, &Default::default())
                .map_err(Error::internal_runtime)?;
        }
        ActionValue::Number(n) => {
            if let Some(num) = n.as_f64() {
                worksheet
                    .write_number(*row_index as u32, 1, num)
                    .map_err(Error::internal_runtime)?;
            } else {
                worksheet
                    .write_string_with_format(
                        *row_index as u32,
                        1,
                        &n.to_string(),
                        &Default::default(),
                    )
                    .map_err(Error::internal_runtime)?;
            }
        }
        ActionValue::Bool(b) => {
            worksheet
                .write_boolean(*row_index as u32, 1, b)
                .map_err(Error::internal_runtime)?;
        }
        ActionValue::Array(arr) => {
            for (col_num, value) in arr.iter().enumerate() {
                match value {
                    ActionValue::String(s) => {
                        worksheet
                            .write_string_with_format(
                                *row_index as u32,
                                col_num as u16 + 1,
                                s,
                                &Default::default(),
                            )
                            .map_err(Error::internal_runtime)?;
                    }
                    ActionValue::Number(n) => {
                        if let Some(num) = n.as_f64() {
                            let _ = worksheet
                                .write_number(*row_index as u32, col_num as u16 + 1, num)
                                .map_err(Error::internal_runtime)?;
                        } else {
                            worksheet
                                .write_string_with_format(
                                    *row_index as u32,
                                    col_num as u16 + 1,
                                    &n.to_string(),
                                    &Default::default(),
                                )
                                .map_err(Error::internal_runtime)?;
                        }
                    }
                    ActionValue::Bool(b) => {
                        worksheet
                            .write_boolean(*row_index as u32, col_num as u16 + 1, *b)
                            .map_err(Error::internal_runtime)?;
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

fn parse_formatting(formatting_str: &str) -> Result<Format> {
    let mut builder = FormatBuilder::new();
    for pair in formatting_str.split(';') {
        let mut parts = pair.splitn(2, ',');
        let key = parts
            .next()
            .ok_or_else(|| Error::internal_runtime("Invalid formatting key"))?;
        let value = parts
            .next()
            .ok_or_else(|| Error::internal_runtime("Invalid formatting value"))?;
        match key {
            "font" => builder = builder.set_font_name(value.to_string()),
            "size" => {
                let size = value
                    .parse::<f64>()
                    .map_err(|_| Error::internal_runtime("Invalid font size"))?;
                builder = builder.set_font_size(size);
            }
            "color" => builder = builder.set_font_color(value),
            "bold" => builder = builder.set_bold(),
            "italic" => builder = builder.set_italic(),
            "underline" => builder = builder.set_underline(str_to_format_underline(value)?),
            "background_color" => builder = builder.set_background_color(value),
            "align" => builder = builder.set_align(str_to_format_align(value)?),
            "wrap" => builder = builder.set_text_wrap(),
            _ => return Err(Error::internal_runtime("Unknown formatting key")),
        };
    }
    Ok(builder.build())
}

// TODO: Row Format works with worksheet directly
// fn parse_row_formatting(row_formatting: &str, worksheet: &worksheet) -> Result<Format> {
//     let mut format = Format::new();
//     for pair in row_formatting.split(';') {
//         let mut parts = pair.splitn(2, ',');
//         let key = parts.next().ok_or_else(|| Error::internal_runtime("Invalid row formatting key"))?;
//         let value = parts.next().ok_or_else(|| Error::internal_runtime("Invalid row formatting value"))?;
//         match key {
//             "row_height" => format.set_row_height(value.parse().map_err(|_| Error::internal_runtime("Invalid row height"))?),
//             _ => return Err(Error::internal_runtime("Unknown row formatting key")),
//         }
//     }
//     Ok(format)
// }

fn str_to_format_underline(s: &str) -> Result<FormatUnderline, Error> {
    match s {
        "None" => Ok(FormatUnderline::None),
        "Single" => Ok(FormatUnderline::Single),
        "Double" => Ok(FormatUnderline::Double),
        "SingleAccounting" => Ok(FormatUnderline::SingleAccounting),
        "DoubleAccounting" => Ok(FormatUnderline::DoubleAccounting),
        _ => Err(Error::internal_runtime("Invalid underline value")),
    }
}

fn str_to_format_align(s: &str) -> Result<FormatAlign, Error> {
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
        _ => Err(Error::internal_runtime("Invalid alignment value")),
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProtectionOptionsDTO {
    pub select_locked_cells: Option<bool>,
    pub select_unlocked_cells: Option<bool>,
    pub format_cells: Option<bool>,
    pub format_columns: Option<bool>,
    pub format_rows: Option<bool>,
    pub insert_columns: Option<bool>,
    pub insert_rows: Option<bool>,
    pub insert_links: Option<bool>,
    pub delete_columns: Option<bool>,
    pub delete_rows: Option<bool>,
    pub sort: Option<bool>,
    pub use_autofilter: Option<bool>,
    pub use_pivot_tables: Option<bool>,
    pub edit_scenarios: Option<bool>,
    pub edit_objects: Option<bool>,
}

fn dto_to_protection_options(dto: &ProtectionOptionsDTO) -> ProtectionOptions {
    ProtectionOptions {
        select_locked_cells: dto.select_locked_cells.unwrap_or_default(),
        select_unlocked_cells: dto.select_unlocked_cells.unwrap_or_default(),
        format_cells: dto.format_cells.unwrap_or(false),
        format_columns: dto.format_columns.unwrap_or(false),
        format_rows: dto.format_rows.unwrap_or(false),
        insert_columns: dto.insert_columns.unwrap_or(false),
        insert_rows: dto.insert_rows.unwrap_or(false),
        insert_links: dto.insert_links.unwrap_or(false),
        delete_columns: dto.delete_columns.unwrap_or(false),
        delete_rows: dto.delete_rows.unwrap_or(false),
        sort: dto.sort.unwrap_or(false),
        use_autofilter: dto.use_autofilter.unwrap_or(false),
        use_pivot_tables: dto.use_pivot_tables.unwrap_or(false),
        edit_scenarios: dto.edit_scenarios.unwrap_or(false),
        edit_objects: dto.edit_objects.unwrap_or(false),
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[allow(unused_imports)]
    use serde_json::Number;

    #[tokio::test]
    async fn test_write_excel_with_formatting_and_hyperlink() {
        let inputs = Some(
            vec![(
                DEFAULT_PORT.clone(),
                Some(ActionValue::Array(vec![ActionValue::Map(
                    vec![
                        (
                            "field1".to_owned(),
                            ActionValue::String("value1".to_owned()),
                        ),
                        (
                            "field2".to_owned(),
                            ActionValue::Number(Number::from_f64(10.0).unwrap()),
                        ),
                        (
                            "field1.hyperlink".to_owned(),
                            ActionValue::String("https://www.example.com".to_owned()),
                        ),
                        (
                            "field1.formatting".to_owned(),
                            ActionValue::String(
                                "font,Arial;size,12;color,#FF0000;bold,true".to_owned(),
                            ),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                )])),
            )]
            .into_iter()
            .collect::<ActionDataframe>(),
        );
        let props = ExcelFileWriter {
            output: "ram:///root/output1.xlsx".to_owned(),
            worksheet_name: "Sheet1".to_owned(),
            template_file: None,
            template_sheet: None,
            protection_options: None,
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_excel(inputs, &props, resolver).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_excel_with_formula() {
        let inputs = Some(
            vec![(
                DEFAULT_PORT.clone(),
                Some(ActionValue::Array(vec![
                    ActionValue::Map(
                        vec![
                            (
                                "field1".to_owned(),
                                ActionValue::String("value1".to_owned()),
                            ),
                            (
                                "field2".to_owned(),
                                ActionValue::Number(Number::from_f64(10.0).unwrap()),
                            ),
                            (
                                "field2.formula".to_owned(),
                                ActionValue::String("=SUM(B1:B2)".to_owned()),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                    ActionValue::Map(
                        vec![
                            (
                                "field1".to_owned(),
                                ActionValue::String("value2".to_owned()),
                            ),
                            (
                                "field2".to_owned(),
                                ActionValue::Number(Number::from_f64(20.0).unwrap()),
                            ),
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
            output: "ram:///root/output2.xlsx".to_owned(),
            worksheet_name: "Sheet1".to_owned(),
            template_file: None,
            template_sheet: None,
            protection_options: None,
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_excel(inputs, &props, resolver).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_excel_with_map_data() {
        let inputs = Some(
            vec![(
                DEFAULT_PORT.clone(),
                Some(ActionValue::Map(
                    vec![
                        (
                            "field1".to_owned(),
                            ActionValue::String("value1".to_owned()),
                        ),
                        (
                            "field2".to_owned(),
                            ActionValue::Number(Number::from_f64(10.0).unwrap()),
                        ),
                        ("field3".to_owned(), ActionValue::Bool(true)),
                        (
                            "field4".to_owned(),
                            ActionValue::Array(vec![
                                ActionValue::String("array1".to_owned()),
                                ActionValue::Number(Number::from_f64(20.0).unwrap()),
                                ActionValue::Bool(false),
                            ]),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                )),
            )]
            .into_iter()
            .collect::<ActionDataframe>(),
        );
        let props = ExcelFileWriter {
            output: "ram:///root/output3.xlsx".to_owned(),
            worksheet_name: "Sheet1".to_owned(),
            template_file: None,
            template_sheet: None,
            protection_options: None,
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_excel(inputs, &props, resolver).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_excel_with_invalid_output_path() {
        let inputs = None;
        let props = ExcelFileWriter {
            output: "invalid_path".to_owned(),
            worksheet_name: "Sheet1".to_owned(),
            template_file: None,
            template_sheet: None,
            protection_options: None,
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_excel(inputs, &props, resolver).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_excel_with_invalid_formatting() {
        let inputs = Some(
            vec![(
                DEFAULT_PORT.clone(),
                Some(ActionValue::Array(vec![ActionValue::Map(
                    vec![
                        (
                            "field1".to_owned(),
                            ActionValue::String("value1".to_owned()),
                        ),
                        (
                            "field1.formatting".to_owned(),
                            ActionValue::String("invalid_formatting".to_owned()),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                )])),
            )]
            .into_iter()
            .collect::<ActionDataframe>(),
        );
        let props = ExcelFileWriter {
            output: "ram:///root/output4.xlsx".to_owned(),
            worksheet_name: "Sheet1".to_owned(),
            template_file: None,
            template_sheet: None,
            protection_options: None,
        };
        let resolver = Arc::new(StorageResolver::default());
        let result = write_excel(inputs, &props, resolver).await;
        assert!(result.is_err());
    }
}
