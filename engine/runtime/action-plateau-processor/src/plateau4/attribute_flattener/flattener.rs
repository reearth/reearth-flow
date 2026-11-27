use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_types::{Attribute, AttributeValue, Attributes};

/// Sort key for river flooding risk schema columns, matching FME's fldAttrsSorter.
/// Sorted by (desc_code, admin_code, scale_code, order) where desc_code
/// is compared numerically when parseable as i64.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct FldSortEntry {
    /// `uro:description_code`: the river system identifier, e.g. "1" for "利根川水系利根川"
    pub desc_code: String,
    /// `uro:adminType_code`: the admin jurisdiction code, e.g. "1" for "国", "2" for "都道府県"
    pub admin_code: String,
    /// `uro:scale_code`: the flood scale code, e.g. "1" for "L1（計画規模）", "2" for "L2（想定最大規模）"
    pub scale_code: String,
    /// Position of the column within its group, `_name` first, then rank, rank_code, depth, duration
    pub order: u8,
    /// Full column name, e.g. `fld_1_1_2_rank`
    pub attr_name: String,
}

impl FldSortEntry {
    fn sort_key(&self) -> impl Ord + '_ {
        let desc_num = self.desc_code.parse::<i64>().ok();
        (
            desc_num,
            &self.desc_code,
            &self.admin_code,
            &self.scale_code,
            self.order,
        )
    }
}

/// Sort fld entries by (desc_code numeric, desc_code string, admin_code, scale_code, order)
pub(super) fn sort_fld_entries(entries: &mut [FldSortEntry]) {
    entries.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
}

/// Push a string, or each string of an array, onto `out`.
fn push_strings(value: &AttributeValue, out: &mut Vec<String>) {
    match value {
        AttributeValue::String(s) => out.push(s.clone()),
        AttributeValue::Array(arr) => {
            for v in arr {
                if let AttributeValue::String(s) = v {
                    out.push(s.clone());
                }
            }
        }
        _ => {}
    }
}

/// Returns the code of the code-typed attribute `key` in `obj`: its `{key}_code` companion,
/// or the value itself when the codelist was not resolved.
fn code_of(obj: &Attributes, key: &str) -> Option<String> {
    obj.get(format!("{key}_code").as_str())
        .or_else(|| obj.get(key))
        .map(|v| v.to_string())
}

/// Whether `code` can be used as a part of a column key.
fn is_key_code(code: &str) -> bool {
    !code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Returns the codes as they are if all of them can be used in a column key, logging a warning
/// that names the dropped risk entry otherwise.
fn key_codes<const N: usize>(tag: &str, codes: [Option<String>; N]) -> Option<[String; N]> {
    if codes.iter().all(|c| c.as_deref().is_some_and(is_key_code)) {
        return Some(codes.map(Option::unwrap_or_default));
    }
    tracing::warn!("Dropping {tag} entry whose codes {codes:?} cannot form a column key");
    None
}

/// The columns of one risk group: `{group}_name` followed by the value columns.
struct RiskGroup<'a> {
    /// Column key prefix, e.g. "fld_1_1_2"
    key: String,
    /// Value of the `{key}_name` column
    name: String,
    /// (column suffix, schema type, value) of each value column
    columns: Vec<(&'a str, AttributeValue, Option<AttributeValue>)>,
}

impl RiskGroup<'_> {
    /// Returns the (column key, schema type, value) of each present column, `_name` first,
    /// or nothing when no value column is present.
    fn into_columns(self) -> Vec<(String, AttributeValue, AttributeValue)> {
        if self.columns.iter().all(|(_, _, value)| value.is_none()) {
            return Vec::new();
        }
        let name = (
            format!("{}_name", self.key),
            AttributeValue::default_string(),
            AttributeValue::String(self.name),
        );
        let key = self.key;
        std::iter::once(name)
            .chain(self.columns.into_iter().filter_map(|(suffix, typ, value)| {
                value.map(|value| (format!("{key}_{suffix}"), typ, value))
            }))
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct Flattener {
    pub(super) risk_to_attribute_definitions: HashMap<String, IndexMap<String, AttributeValue>>,
    /// Unsorted entries for fld risk attributes; sorted once at schema generation time.
    pub(super) fld_sort_entries: Vec<FldSortEntry>,
}

impl Flattener {
    /// Flattens `uro:RiverFloodingRiskAttribute` into one `fld_{description}_{adminType}_{scale}`
    /// column group per entry, keyed by codes.
    pub(super) fn extract_fld_risk_attribute(
        &mut self,
        attributes: &Attributes,
    ) -> IndexMap<Attribute, AttributeValue> {
        let Some(disaster_risks) = attributes.get("uro:RiverFloodingRiskAttribute") else {
            return IndexMap::new();
        };
        let disaster_risks = match disaster_risks {
            AttributeValue::Array(disaster_risks) => disaster_risks,
            AttributeValue::Map(disaster_risks) => {
                &vec![AttributeValue::Map(disaster_risks.clone())]
            }
            _ => return IndexMap::new(),
        };
        let mut result = IndexMap::new();
        for risk in disaster_risks {
            let risk_obj = match risk.as_map() {
                Some(obj) => obj,
                None => continue,
            };
            let desc = risk_obj.get("uro:description").map(|v| v.to_string());
            let admin = risk_obj.get("uro:adminType").map(|v| v.to_string());
            let scale = risk_obj.get("uro:scale").map(|v| v.to_string());
            let (Some(desc), Some(admin), Some(scale)) = (desc, admin, scale) else {
                continue;
            };
            let Some([desc_code, admin_code, scale_code]) = key_codes(
                "uro:RiverFloodingRiskAttribute",
                [
                    code_of(&risk_obj, "uro:description"),
                    code_of(&risk_obj, "uro:adminType"),
                    code_of(&risk_obj, "uro:scale"),
                ],
            ) else {
                continue;
            };

            let mut rank = risk_obj.get("uro:rank").map(|v| v.to_string());
            let mut rank_code = risk_obj.get("uro:rank_code").map(|v| v.to_string());
            if rank.is_none() {
                rank = risk_obj.get("uro:rankOrg").map(|v| v.to_string());
                rank_code = risk_obj.get("uro:rankOrg_code").map(|v| v.to_string());
            }
            let depth = risk_obj.get("uro:depth").map(|v| v.to_string());
            let duration = risk_obj.get("uro:duration").map(|v| v.to_string());

            let group = RiskGroup {
                key: format!("fld_{desc_code}_{admin_code}_{scale_code}"),
                name: format!("{desc}（{admin}管理区間）_{scale}"),
                columns: vec![
                    ("rank", AttributeValue::default_string(), rank),
                    ("rank_code", AttributeValue::default_number(), rank_code),
                    ("depth", AttributeValue::default_float(), depth),
                    ("duration", AttributeValue::default_float(), duration),
                ]
                .into_iter()
                .map(|(suffix, typ, value)| (suffix, typ, value.map(AttributeValue::String)))
                .collect(),
            };
            for (order, (name, typ, value)) in group.into_columns().into_iter().enumerate() {
                result.insert(Attribute::new(name.clone()), value);
                self.risk_to_attribute_definitions
                    .entry("fld".to_string())
                    .or_default()
                    .insert(name.clone(), typ);
                self.fld_sort_entries.push(FldSortEntry {
                    desc_code: desc_code.clone(),
                    admin_code: admin_code.clone(),
                    scale_code: scale_code.clone(),
                    order: order as u8,
                    attr_name: name,
                });
            }
        }
        result
    }

    /// Flattens the tsunami, high tide, inland flooding and reservoir flooding risk attributes into
    /// one `{package}_{description}` column group per entry, keyed by the description code.
    pub(super) fn extract_tnm_htd_ifld_risk_attribute(
        &mut self,
        attributes: &Attributes,
    ) -> IndexMap<Attribute, AttributeValue> {
        let src = [
            ("uro:TsunamiRiskAttribute", "津波浸水想定", "tnm"),
            ("uro:HighTideRiskAttribute", "高潮浸水想定", "htd"),
            ("uro:InlandFloodingRiskAttribute", "内水浸水想定", "ifld"),
            (
                "uro:ReservoirFloodingRiskAttribute",
                "ため池浸水想定",
                "rfld",
            ),
        ];

        let mut result = IndexMap::new();
        for (tag, title, package) in src.iter() {
            let Some(disaster_risks) = attributes.get(*tag) else {
                continue;
            };
            let disaster_risks = match disaster_risks {
                AttributeValue::Array(disaster_risks) => disaster_risks,
                AttributeValue::Map(disaster_risks) => {
                    &vec![AttributeValue::Map(disaster_risks.clone())]
                }
                _ => return IndexMap::new(),
            };

            for risk_value in disaster_risks {
                let risk_obj = match risk_value.as_map() {
                    Some(obj) => obj,
                    None => continue,
                };

                let Some(desc) = risk_obj.get("uro:description").map(|v| v.to_string()) else {
                    continue;
                };
                let Some([desc_code]) = key_codes(tag, [code_of(&risk_obj, "uro:description")])
                else {
                    continue;
                };

                let mut rank_opt = risk_obj.get("uro:rank").map(|v| v.to_string());
                let mut rank_code_opt = risk_obj.get("uro:rank_code").map(|v| v.to_string());
                if rank_opt.is_none() {
                    rank_opt = risk_obj.get("uro:rankOrg").map(|v| v.to_string());
                    rank_code_opt = risk_obj.get("uro:rankOrg_code").map(|v| v.to_string());
                }
                let depth_opt = risk_obj.get("uro:depth").map(|v| v.to_string());

                let group = RiskGroup {
                    key: format!("{package}_{desc_code}"),
                    name: format!("{title}_{desc}"),
                    columns: vec![
                        ("rank", AttributeValue::default_string(), rank_opt),
                        ("rank_code", AttributeValue::default_number(), rank_code_opt),
                        ("depth", AttributeValue::default_float(), depth_opt),
                    ]
                    .into_iter()
                    .map(|(suffix, typ, value)| (suffix, typ, value.map(AttributeValue::String)))
                    .collect(),
                };
                for (name, typ, value) in group.into_columns() {
                    result.insert(Attribute::new(name.clone()), value);
                    self.risk_to_attribute_definitions
                        .entry(package.to_string())
                        .or_default()
                        .insert(name, typ);
                }
            }
        }
        result
    }

    /// Flattens `uro:LandSlideRiskAttribute` into one `lsld_{description}` column group per entry,
    /// keyed by the description code.
    pub(super) fn extract_lsld_risk_attribute(
        &mut self,
        attributes: &Attributes,
    ) -> IndexMap<Attribute, AttributeValue> {
        let Some(disaster_risks) = attributes.get("uro:LandSlideRiskAttribute") else {
            return IndexMap::new();
        };
        let disaster_risks = match disaster_risks {
            AttributeValue::Array(disaster_risks) => disaster_risks,
            AttributeValue::Map(disaster_risks) => {
                &vec![AttributeValue::Map(disaster_risks.clone())]
            }
            _ => return IndexMap::new(),
        };
        let mut result = IndexMap::new();
        for risk_value in disaster_risks {
            let risk_obj = match risk_value.as_map() {
                Some(obj) => obj,
                None => continue,
            };

            let desc_opt = risk_obj.get("uro:description").map(|v| v.to_string());
            let area_type_opt = risk_obj.get("uro:areaType").map(|v| v.to_string());
            let type_code_opt = risk_obj.get("uro:areaType_code").map(|v| v.to_string());

            let (Some(desc), Some(type_code_str)) = (desc_opt, type_code_opt) else {
                continue;
            };

            let type_code_int: i32 = match type_code_str.parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            if type_code_int > 2 {
                continue;
            }
            let Some([desc_code]) = key_codes(
                "uro:LandSlideRiskAttribute",
                [code_of(&risk_obj, "uro:description")],
            ) else {
                continue;
            };

            let group = RiskGroup {
                key: format!("lsld_{desc_code}"),
                name: format!("土砂災害リスク_{desc}"),
                columns: vec![
                    (
                        "area_type",
                        AttributeValue::default_string(),
                        area_type_opt
                            .filter(|v| !v.is_empty())
                            .map(AttributeValue::String),
                    ),
                    (
                        "area_type_code",
                        AttributeValue::default_number(),
                        Some(AttributeValue::Number(type_code_int.into())),
                    ),
                ],
            };
            for (name, typ, value) in group.into_columns() {
                result.insert(Attribute::new(name.clone()), value);
                self.risk_to_attribute_definitions
                    .entry("lsld".to_string())
                    .or_default()
                    .insert(name, typ);
            }
        }
        result
    }

    /// Extract bldg:address from core:Address nested structure: every xAL:LocalityName in order,
    /// then the xAL:DependentLocalityName and xAL:DependentLocalityNumber of the first
    /// xAL:DependentLocality whose Type is "district".
    /// core:Address[0].xAL:AddressDetails[0].xAL:Country[0].xAL:Locality[*] -> bldg:address
    pub(super) fn extract_address(attributes: &Attributes) -> Option<AttributeValue> {
        let address_array = attributes.get("core:Address")?;
        let address_list = address_array.as_vec()?;
        let address_obj = address_list.first()?.as_map()?;

        let details_array = address_obj.get("xAL:AddressDetails")?;
        let details_list = details_array.as_vec()?;
        let details_obj = details_list.first()?.as_map()?;

        let country_array = details_obj.get("xAL:Country")?;
        let country_list = country_array.as_vec()?;
        let country_obj = country_list.first()?.as_map()?;

        let localities = country_obj.get("xAL:Locality")?.as_vec()?;

        let mut parts: Vec<String> = Vec::new();
        let mut district: Option<Vec<String>> = None;
        for locality in localities.iter().filter_map(|l| l.as_map()) {
            if let Some(names) = locality.get("xAL:LocalityName") {
                push_strings(names, &mut parts);
            }
            if district.is_some() {
                continue;
            }
            let Some(dependents) = locality
                .get("xAL:DependentLocality")
                .and_then(|d| d.as_vec())
            else {
                continue;
            };
            for dependent in dependents.iter().filter_map(|d| d.as_map()) {
                let is_district = matches!(
                    dependent.get("locality_type"),
                    Some(AttributeValue::String(t)) if t == "district"
                );
                if !is_district {
                    continue;
                }
                let mut names = Vec::new();
                if let Some(v) = dependent.get("xAL:DependentLocalityName") {
                    push_strings(v, &mut names);
                }
                if let Some(v) = dependent.get("xAL:DependentLocalityNumber") {
                    push_strings(v, &mut names);
                }
                district = Some(names);
                break;
            }
        }
        parts.extend(district.unwrap_or_default());

        let joined = parts.join("");
        if joined.is_empty() {
            None
        } else {
            Some(AttributeValue::String(joined))
        }
    }
}

/// Column group assigned to one generic attribute name.
#[derive(Debug, Clone)]
struct GenericColumn {
    /// Group number, starting at 1 in the order the names are first seen
    index: usize,
    /// Generic attribute type of the values, or "string" when the name is seen with different types
    value_type: String,
    /// Whether some value of this name has a unit
    has_uom: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub(super) struct CommonAttributeProcessor {
    max_lod: i64,
    gml_path_to_max_lod: HashMap<String, i64>,
    /// Generic attribute name, prefixed by its attribute set names -> its column group
    generic_columns: IndexMap<String, GenericColumn>,
}

impl CommonAttributeProcessor {
    /// Schema of the generic attribute column groups seen so far: `gen_{i}_name`, `gen_{i}_value`
    /// and, for measures, `gen_{i}_uom`.
    pub(super) fn get_generic_schema(&self) -> IndexMap<Attribute, AttributeValue> {
        let mut result = IndexMap::new();
        for column in self.generic_columns.values() {
            let i = column.index;
            result.insert(
                Attribute::new(format!("gen_{i}_name")),
                AttributeValue::default_string(),
            );
            let value_type = match column.value_type.as_str() {
                "string" | "date" | "buffer" => Some(AttributeValue::default_string()),
                "int" => Some(AttributeValue::default_number()),
                "double" | "measure" => Some(AttributeValue::default_float()),
                _ => None,
            };
            if let Some(value_type) = value_type {
                result.insert(Attribute::new(format!("gen_{i}_value")), value_type);
            }
            if column.has_uom {
                result.insert(
                    Attribute::new(format!("gen_{i}_uom")),
                    AttributeValue::default_string(),
                );
            }
        }
        result
    }

    /// Returns the column group of the generic attribute `name` of type `typ`, assigning the next
    /// group number to a new name.
    fn generic_column(&mut self, name: &str, typ: &str, has_uom: bool) -> usize {
        let next_index = self.generic_columns.len() + 1;
        let column = self
            .generic_columns
            .entry(name.to_string())
            .or_insert_with(|| GenericColumn {
                index: next_index,
                value_type: typ.to_string(),
                has_uom: false,
            });
        if column.value_type != typ {
            column.value_type = "string".to_string();
        }
        column.has_uom |= has_uom;
        column.index
    }

    fn flatten_generic_attribute(
        &mut self,
        attribute: &Attributes,
        prefix: &str,
    ) -> IndexMap<Attribute, AttributeValue> {
        let mut result = IndexMap::new();
        if let (Some(AttributeValue::String(name)), Some(AttributeValue::String(typ))) =
            (attribute.get("name"), attribute.get("type"))
        {
            let name = format!("{prefix}{name}");
            let value = attribute.get("value").unwrap_or(&AttributeValue::Null);
            if typ == "attributeSet" {
                if let AttributeValue::Array(attribute_set) = value {
                    for attribute in attribute_set {
                        if let AttributeValue::Map(attribute) = attribute {
                            let prefix = format!("{name}_");
                            result
                                .extend(self.flatten_generic_attribute(attribute, prefix.as_str()));
                        }
                    }
                    return result;
                }
            } else if matches!(value, AttributeValue::Null) {
                return result;
            }
            let uom = attribute.get("uom").filter(|_| typ == "measure");
            let i = self.generic_column(&name, typ, uom.is_some());
            result.insert(
                Attribute::new(format!("gen_{i}_name")),
                AttributeValue::String(name),
            );
            result.insert(Attribute::new(format!("gen_{i}_value")), value.clone());
            if let Some(uom) = uom {
                result.insert(Attribute::new(format!("gen_{i}_uom")), uom.clone());
            }
        }
        result
    }

    /// Flattens `gen:genericAttribute` into `gen_{i}_name`, `gen_{i}_value` and `gen_{i}_uom`
    /// column groups, one per distinct name.
    pub(super) fn flatten_generic_attributes(
        &mut self,
        attributes: &Attributes,
    ) -> IndexMap<Attribute, AttributeValue> {
        let mut result = IndexMap::new();

        // Extract gen:genericAttribute array from the citygml attributes
        let Some(generic_attrs) = attributes.get("gen:genericAttribute") else {
            return result;
        };

        let generic_attrs = match generic_attrs {
            AttributeValue::Array(arr) => arr,
            AttributeValue::Map(map) => {
                // Single attribute case
                result.extend(self.flatten_generic_attribute(map, ""));
                return result;
            }
            _ => return result,
        };

        // Process each generic attribute in the array
        for attr in generic_attrs {
            if let AttributeValue::Map(attr_map) = attr {
                result.extend(self.flatten_generic_attribute(attr_map, ""));
            }
        }

        result
    }

    #[allow(dead_code)]
    fn extract_lod_types(
        &self,
        attrib: &Attributes,
        parent_tag: &str,
    ) -> HashMap<Attribute, AttributeValue> {
        let mut result = HashMap::new();
        let parent_attr = match attrib.get(parent_tag) {
            Some(AttributeValue::Array(parent_attr)) => parent_attr,
            _ => return result,
        };
        if parent_attr.is_empty() {
            return result;
        }
        let first_element = match &parent_attr[0] {
            AttributeValue::Map(first_element) => first_element,
            _ => return result,
        };
        let lod_types = match first_element.get("uro:lodType") {
            Some(AttributeValue::Array(lod_types)) => lod_types,
            _ => return result,
        };
        for lod_type in lod_types {
            let s = match lod_type.to_string().chars().next() {
                Some(s) => s,
                None => continue,
            };
            if s.to_string() == "2" || s.to_string() == "3" || s.to_string() == "4" {
                let key = format!("lod_type_{s}");
                let attribute_name = Attribute::new(key.clone());
                result.insert(attribute_name, lod_type.clone());
            }
        }
        result
    }
}

pub(super) fn get_value_from_json_path(
    paths: &[&str],
    attrib: &Attributes,
) -> Option<AttributeValue> {
    let key = paths.first()?;
    let value = attrib.get(*key)?;
    let value = if let AttributeValue::Array(array) = value {
        if let Some(AttributeValue::Map(map)) = &array.first() {
            get_value_from_json_path(&paths[1..], map)
        } else if *key == "uro:lodType" {
            // joined with a bare comma, as in the reference implementation
            let joined = array
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            Some(AttributeValue::String(joined))
        } else {
            // take first element
            Some(array.first()?.clone())
        }
    } else if let AttributeValue::Number(num) = value {
        match num.as_i64() {
            Some(num) if num == 9999 || num == -9999 => None,
            _ => Some(value.clone()),
        }
    } else if let AttributeValue::Map(value) = value {
        get_value_from_json_path(&paths[1..], value)
    } else {
        Some(value.clone())
    };
    value
}
