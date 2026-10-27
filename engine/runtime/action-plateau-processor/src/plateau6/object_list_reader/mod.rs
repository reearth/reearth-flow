//! PLATEAU 6 (CityGML 3.0) objectlist reader.
//!
//! The workbook is read in two passes: the model sheets into rows
//! ([`model_sheet`]), then `featureTypes` ([`feature_types`]) and `objectList`
//! ([`object_list`]) each from those rows.

mod feature_types;
mod gml_path;
mod model_sheet;
mod object_list;

use bytes::Bytes;

use self::feature_types::build_feature_types;
use self::model_sheet::read_model_sheets;
use self::object_list::build_object_list;
use crate::common::object_list_extractor::ObjectListReader;
use crate::object_list::{FeatureTypes, ObjectListMap, Result};

#[derive(Debug)]
pub(crate) struct Plateau6ObjectListReader;

impl ObjectListReader for Plateau6ObjectListReader {
    fn read(&self, bytes: Bytes) -> Result<(FeatureTypes, ObjectListMap)> {
        let sheets = read_model_sheets(bytes)?;
        Ok((build_feature_types(&sheets), build_object_list(&sheets)))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rust_xlsxwriter::Workbook as XlsxWriter;

    use super::*;

    /// One table row: prefix, feature type, five attribute levels, category,
    /// 作成対象 and 不明値.
    fn row(
        prefix: &str,
        feature_type: &str,
        attributes: [&str; 5],
        category: &str,
        create: &str,
        unknown: &str,
    ) -> Vec<String> {
        let mut columns = vec![String::new(); 16];
        columns[0] = prefix.to_string();
        columns[1] = feature_type.to_string();
        for (i, attribute) in attributes.iter().enumerate() {
            columns[2 + i] = attribute.to_string();
        }
        columns[7] = category.to_string();
        columns[9] = create.to_string();
        columns[14] = unknown.to_string();
        columns
    }

    fn header() -> Vec<Vec<String>> {
        vec![
            row(
                "モデルの接頭辞",
                "地物名",
                ["属性名／関連役割名", "", "", "", ""],
                "",
                "",
                "",
            ),
            row(
                "",
                "",
                ["", "", "", "", ""],
                "主題属性、空間属性、関連役割の区分",
                "作成対象",
                "不明な場合に不明値を入力する。",
            ),
        ]
    }

    fn workbook(sheets: Vec<(&str, Vec<Vec<String>>)>) -> Bytes {
        let mut workbook = XlsxWriter::new();
        for (name, rows) in sheets {
            let sheet = workbook.add_worksheet();
            sheet.set_name(name).unwrap();
            for (r, columns) in rows.iter().enumerate() {
                for (c, value) in columns.iter().enumerate() {
                    if !value.is_empty() {
                        sheet.write_string(r as u32, c as u16, value).unwrap();
                    }
                }
            }
        }
        Bytes::from(workbook.save_to_buffer().unwrap())
    }

    fn entry(
        object_list: &ObjectListMap,
        prefix: &str,
        feature_type: &str,
    ) -> (Vec<String>, Vec<String>) {
        let value = object_list.get(prefix).unwrap().get(feature_type).unwrap();
        (value.required.clone(), value.target.clone())
    }

    #[test]
    fn test_read() {
        let mut building = header();
        building.extend([
            row("bldg", "bldg:Building", ["", "", "", "", ""], "", "", ""),
            row(
                "bldg",
                "",
                ["core:featureID", "", "", "", ""],
                "主題",
                "○",
                "○",
            ),
            row("bldg", "", ["core:name", "", "", "", ""], "主題", "○", "○"),
            row(
                "bldg",
                "",
                ["core:description", "", "", "", ""],
                "主題",
                "",
                "○",
            ),
            row(
                "bldg",
                "",
                [
                    "bldg:adeOfAbstractBuilding.uro:BuildingIDAttribute",
                    "",
                    "",
                    "",
                    "",
                ],
                "主題",
                "○",
                "○",
            ),
            row(
                "bldg",
                "",
                ["", "uro:buildingID", "", "", ""],
                "主題",
                "○",
                "○",
            ),
            row(
                "bldg",
                "",
                ["", "uro:branchID", "", "", ""],
                "主題",
                "○",
                "",
            ),
            row(
                "bldg",
                "",
                [
                    "core:adeOfAbstractCityObject.uro:DataQualityAttribute",
                    "",
                    "",
                    "",
                    "",
                ],
                "主題",
                "○",
                "○",
            ),
            row(
                "bldg",
                "",
                [
                    "",
                    "uro:publicSurveyDataQualityAttribute.uro:PublicSurveyDataQualityAttribute",
                    "",
                    "",
                    "",
                ],
                "関連役割",
                "○",
                "",
            ),
            row(
                "bldg",
                "",
                ["", "", "uro:srcScaleLod0", "", ""],
                "主題",
                "○",
                "",
            ),
            row(
                "bldg",
                "",
                ["bldg:adeOfAbstractBuilding.uro:A", "", "", "", ""],
                "主題",
                "",
                "",
            ),
            row("bldg", "", ["", "uro:b.uro:B", "", "", ""], "主題", "", ""),
            row("bldg", "", ["", "", "uro:c.uro:C", "", ""], "主題", "", ""),
            row("bldg", "", ["", "", "", "uro:d.uro:D", ""], "主題", "", ""),
            row("bldg", "", ["", "", "", "", "uro:e"], "主題", "○", ""),
            row(
                "bldg",
                "",
                ["core:lod1Solid", "", "", "", ""],
                "空間",
                "○",
                "",
            ),
        ]);
        let mut disaster = header();
        disaster.extend([
            row(
                "fld/tnm/htd/ifld/rfld",
                "wtr:WaterBody",
                ["", "", "", "", ""],
                "",
                "",
                "",
            ),
            row(
                "fld/tnm/htd/ifld/rfld",
                "",
                ["core:creationDate", "", "", "", ""],
                "主題",
                "○",
                "○",
            ),
            row(
                "fld/tnm/htd/ifld/rfld",
                "",
                [
                    "core:adeOfAbstractCityObject.uro:RiverFloodingRiskAttribute",
                    "",
                    "",
                    "",
                    "",
                ],
                "主題",
                "○",
                "",
            ),
            row(
                "fld/tnm/htd/ifld/rfld",
                "",
                ["", "uro:rank", "", "", ""],
                "主題",
                "○",
                "",
            ),
            row(
                "fld/tnm/htd/ifld/rfld",
                "",
                [
                    "core:adeOfAbstractCityObject.uro:TsunamiRiskAttribute",
                    "",
                    "",
                    "",
                    "",
                ],
                "主題",
                "○",
                "",
            ),
        ]);
        let mut group = header();
        group.extend([
            row("", "grp:CityObjectGroup", ["", "", "", "", ""], "", "", ""),
            row(
                "",
                "",
                ["core:creationDate", "", "", "", ""],
                "主題",
                "○",
                "○",
            ),
        ]);
        let bytes = workbook(vec![
            ("表紙", vec![vec!["objectlist".to_string()]]),
            ("01. 建築物モデル", building),
            ("08. 災害リスクモデル", disaster),
            ("21. 都市オブジェクトグループモデル", group),
        ]);

        let (feature_types, object_list) = Plateau6ObjectListReader.read(bytes).unwrap();

        let mut prefixes = feature_types.into_inner().into_keys().collect::<Vec<_>>();
        prefixes.sort();
        assert_eq!(prefixes, vec!["bldg", "fld", "htd", "ifld", "rfld", "tnm"]);

        assert_eq!(
            entry(&object_list, "bldg", "bldg:Building"),
            (
                vec![
                    "gml:name".to_string(),
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute".to_string(),
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute/uro:buildingID".to_string(),
                    "core:adeOfAbstractCityObject/urc:DataQualityAttribute".to_string(),
                ],
                vec![
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute/uro:branchID".to_string(),
                    "core:adeOfAbstractCityObject/urc:DataQualityAttribute/urc:publicSurveyDataQualityAttribute/urc:PublicSurveyDataQualityAttribute".to_string(),
                    "core:adeOfAbstractCityObject/urc:DataQualityAttribute/urc:publicSurveyDataQualityAttribute/urc:PublicSurveyDataQualityAttribute/urc:srcScaleLod0".to_string(),
                    "bldg:adeOfAbstractBuilding/uro:A/uro:b/uro:B/uro:c/uro:C/uro:d/uro:D/uro:e".to_string(),
                ],
            )
        );
        assert_eq!(
            entry(&object_list, "fld", "wtr:WaterBody"),
            (
                vec!["core:creationDate".to_string()],
                vec![
                    "core:adeOfAbstractCityObject/urc:RiverFloodingRiskAttribute".to_string(),
                    "core:adeOfAbstractCityObject/urc:RiverFloodingRiskAttribute/urc:rank"
                        .to_string(),
                ],
            )
        );
        assert_eq!(
            entry(&object_list, "tnm", "wtr:WaterBody"),
            (
                vec!["core:creationDate".to_string()],
                vec!["core:adeOfAbstractCityObject/urc:TsunamiRiskAttribute".to_string()],
            )
        );
        assert_eq!(
            entry(&object_list, "rfld", "wtr:WaterBody").1,
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_read_conditional() {
        let required = |feature_type: &str, attribute: [&str; 5]| {
            row("bldg", feature_type, attribute, "主題", "○", "○")
        };
        let building_id = |feature_type: &str| {
            [
                required(
                    feature_type,
                    [
                        "bldg:adeOfAbstractBuilding.uro:BuildingIDAttribute",
                        "",
                        "",
                        "",
                        "",
                    ],
                ),
                required("", ["", "uro:buildingID", "", "", ""]),
            ]
        };
        let mut building = header();
        building.extend(building_id("bldg:Building"));
        building.extend([
            required("", ["core:creationDate", "", "", "", ""]),
            required("", ["con:height", "", "", "", ""]),
        ]);
        building.extend(building_id("bldg:BuildingPart"));
        building.extend([
            required("", ["core:creationDate", "", "", "", ""]),
            required("", ["bldg:class", "", "", "", ""]),
        ]);
        let bytes = workbook(vec![("01. 建築物モデル", building)]);

        let (_, object_list) = Plateau6ObjectListReader.read(bytes).unwrap();

        let entry = |feature_type: &str| {
            let value = object_list.get("bldg").unwrap().get(feature_type).unwrap();
            (value.required.clone(), value.conditional.clone())
        };
        // Each keeps its own required attributes, including the building ID,
        // and only the shared ones become conditional.
        assert_eq!(
            entry("bldg:Building"),
            (
                vec![
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute".to_string(),
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute/uro:buildingID".to_string(),
                    "con:height".to_string(),
                ],
                vec!["core:creationDate".to_string()],
            )
        );
        assert_eq!(
            entry("bldg:BuildingPart"),
            (
                vec![
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute".to_string(),
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute/uro:buildingID".to_string(),
                    "bldg:class".to_string(),
                ],
                vec!["core:creationDate".to_string()],
            )
        );
    }

    #[test]
    fn test_read_feature_id_only() {
        let mut building = header();
        building.extend([
            row(
                "bldg",
                "bldg:BuildingInstallation",
                ["core:featureID", "", "", "", ""],
                "主題",
                "○",
                "○",
            ),
            row(
                "bldg",
                "",
                ["core:lod2MultiSurface", "", "", "", ""],
                "空間",
                "○",
                "",
            ),
        ]);
        let bytes = workbook(vec![("01. 建築物モデル", building)]);

        let (feature_types, object_list) = Plateau6ObjectListReader.read(bytes).unwrap();

        // Still cut out and checked, with no attribute to check.
        assert_eq!(
            feature_types.into_inner().get("bldg"),
            Some(&vec!["bldg:BuildingInstallation".to_string()])
        );
        assert_eq!(
            entry(&object_list, "bldg", "bldg:BuildingInstallation"),
            (vec![], vec![])
        );
    }

    #[test]
    fn test_read_data_type() {
        let mut generics = header();
        generics.extend([
            row(
                "gen",
                "gen:GenericAttributeSet",
                ["gen:name", "", "", "", ""],
                "主題",
                "○",
                "○",
            ),
            row(
                "gen",
                "",
                ["gen:genericAttribute.gen:StringAttribute", "", "", "", ""],
                "関連役割",
                "○",
                "",
            ),
        ]);
        let bytes = workbook(vec![("12. 汎用都市オブジェクトモデル", generics)]);

        let (feature_types, object_list) = Plateau6ObjectListReader.read(bytes).unwrap();

        // Neither cut out nor checked, even when marked.
        assert!(feature_types.into_inner().is_empty());
        assert!(object_list.get("gen").is_none());
    }

    #[test]
    fn test_read_without_model_sheet() {
        let bytes = workbook(vec![(
            "A.3.1_取得項目一覧",
            vec![vec!["".to_string(), "地物名".to_string()]],
        )]);
        assert!(Plateau6ObjectListReader.read(bytes).is_err());
    }
}
