//! A feature's attributes as CityGML property elements, before placement.
//!
//! This inverts the shapes the CityGML readers write (`parser_next.rs`
//! `node_to_attribute_value`, `codespace.rs`): `"$"` is text, `"@x"` an XML
//! attribute, an array repeats its element, a nested map nests, and the
//! `_uom`, `_code` and `_codeSpace` sibling keys fold back onto the element
//! they were split from. Nothing here knows the schema; `placement` decides
//! where, and whether, each top-level property goes.

use std::collections::HashSet;

use reearth_flow_citygml::schema::QName;
use reearth_flow_diagnostics::ErrorCode;
use reearth_flow_types::{AttributeValue, Attributes};

use super::writer::{is_ncname, namespace_for_prefix};

/// Keys a CityGML reader writes that the writer produces itself.
const NEVER_WRITTEN: &[&str] = &["gml:boundedBy", "app:appearance"];

const UOM_SUFFIX: &str = "_uom";
const CODE_SUFFIX: &str = "_code";
const CODE_SPACE_SUFFIX: &str = "_codeSpace";

/// One element built from an attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlProperty {
    /// One of the prefixes `namespace_for_prefix` knows.
    pub prefix: String,
    pub local: String,
    /// XML attributes by the qualified name written, such as `uom`,
    /// `codeSpace`, `gml:id` or `xlink:href`.
    pub attrs: Vec<(String, String)>,
    pub text: Option<String>,
    pub children: Vec<XmlProperty>,
}

impl XmlProperty {
    pub fn new(prefix: &str, local: &str) -> Self {
        Self {
            prefix: prefix.to_owned(),
            local: local.to_owned(),
            attrs: Vec::new(),
            text: None,
            children: Vec::new(),
        }
    }

    pub fn qname(&self) -> QName {
        QName::new(
            namespace_for_prefix(&self.prefix).expect("properties are built with known prefixes"),
            self.local.as_str(),
        )
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Why content was left out. Each maps to one registry code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    NotPlaced,
    OverLimit,
    NestedObject,
}

impl Skip {
    pub fn code(self) -> ErrorCode {
        match self {
            Skip::NotPlaced => ErrorCode::CitygmlAttributeNotPlaced,
            Skip::OverLimit => ErrorCode::CitygmlAttributeOverLimit,
            Skip::NestedObject => ErrorCode::CitygmlNestedObjectNotWritten,
        }
    }
}

#[derive(Debug, Default)]
pub struct Converted {
    pub properties: Vec<XmlProperty>,
    pub skipped: Vec<Skip>,
    /// Codelist URLs taken from `_codeSpace` siblings, first-seen order. Only
    /// these are staged: a `codeSpace` the source wrote itself is relative to
    /// a document the output no longer sits beside.
    pub code_spaces: Vec<String>,
}

/// The top-level attributes that are CityGML properties, as elements.
///
/// Unprefixed keys, `__` keys, `@`/`$` keys and `excluded` are pipeline data,
/// not properties, and are passed over without a report.
pub fn from_attributes(attributes: &Attributes, excluded: &HashSet<&str>) -> Converted {
    let mut out = Converted::default();
    for (key, value) in attributes {
        let key = key.as_str();
        if excluded.contains(key)
            || key.starts_with("__")
            || key.starts_with('@')
            || key == "$"
            || NEVER_WRITTEN.contains(&key)
            || is_sibling(key, attributes)
        {
            continue;
        }
        let Some((prefix, local)) = key.split_once(':') else {
            continue;
        };
        if !is_known_name(prefix, local) || is_geometry_property(local) {
            out.skipped.push(Skip::NotPlaced);
            continue;
        }
        let siblings = Siblings::of(key, attributes);
        let mut elements = Vec::new();
        push_elements(prefix, local, value, &siblings, &mut elements, &mut out);
        for element in elements {
            // `name` is required on every generic attribute, and a reader run
            // with `keepAttributes: false` drops it.
            if element.prefix == "gen" && element.attr("name").is_none() {
                out.skipped.push(Skip::NotPlaced);
            } else {
                out.properties.push(element);
            }
        }
    }
    out
}

/// Whether `prefix:local` is a name the writer can bind and write.
fn is_known_name(prefix: &str, local: &str) -> bool {
    namespace_for_prefix(prefix).is_some() && is_ncname(prefix) && is_ncname(local)
}

/// The reader's geometry properties: `lod<N>…` and `tin`. They are written
/// from geometry, never from text.
fn is_geometry_property(local: &str) -> bool {
    local == "tin"
        || local
            .strip_prefix("lod")
            .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))
}

/// Whether `key` is a `_uom`, `_code` or `_codeSpace` sibling of a key in `map`.
fn is_sibling(key: &str, map: &Attributes) -> bool {
    [CODE_SPACE_SUFFIX, CODE_SUFFIX, UOM_SUFFIX]
        .iter()
        .any(|suffix| {
            key.strip_suffix(suffix)
                .is_some_and(|base| map.contains_key(base))
        })
}

struct Siblings<'a> {
    uom: Option<&'a AttributeValue>,
    code: Option<&'a AttributeValue>,
    code_space: Option<&'a AttributeValue>,
}

impl<'a> Siblings<'a> {
    fn of(key: &str, map: &'a Attributes) -> Self {
        let get = |suffix: &str| map.get(format!("{key}{suffix}").as_str());
        Self {
            uom: get(UOM_SUFFIX),
            code: get(CODE_SUFFIX),
            code_space: get(CODE_SPACE_SUFFIX),
        }
    }
}

/// Builds the element for each of `value`'s repeats and pairs the siblings
/// with the values they were split from.
///
/// The reader writes `_code` and `_codeSpace` only for codes it resolved, so
/// they pair, in order, with the values that carry no `@codeSpace` of their
/// own. `_uom` pairs with the flattened numbers: one string applies to all of
/// them, an array pairs by position. When the counts of code entries and of
/// values that could take one disagree, as with `keepAttributes: false`, where
/// an unresolved code is a bare string that looks like a label, nothing says
/// which belongs to which, so the whole property is left out.
fn push_elements(
    prefix: &str,
    local: &str,
    value: &AttributeValue,
    siblings: &Siblings,
    into: &mut Vec<XmlProperty>,
    out: &mut Converted,
) {
    let items: Vec<&AttributeValue> = match value {
        AttributeValue::Array(items) => items.iter().collect(),
        _ => vec![value],
    };
    let takers = items.iter().filter(|item| takes_code(item)).count();
    let mut code = entries(siblings.code);
    let mut code_space = entries(siblings.code_space);
    if [&code, &code_space]
        .into_iter()
        .flatten()
        .any(|list| list.len() != takers)
    {
        out.skipped.push(Skip::NotPlaced);
        return;
    }
    let mut numbers = 0;
    for item in items {
        if matches!(item, AttributeValue::Array(_)) {
            out.skipped.push(Skip::NotPlaced);
            continue;
        }
        let mut own = Siblings {
            uom: None,
            code: None,
            code_space: None,
        };
        if takes_code(item) {
            own.code = code.as_mut().map(|list| list.remove(0));
            own.code_space = code_space.as_mut().map(|list| list.remove(0));
        }
        if matches!(item, AttributeValue::Number(_)) {
            own.uom = match siblings.uom {
                Some(AttributeValue::Array(uoms)) => uoms.get(numbers),
                other => other,
            };
            numbers += 1;
        }
        if let Some(element) = element(prefix, local, item, &own, out) {
            into.push(element);
        }
    }
}

/// The sibling values of one key as a list: an array is its entries, a single
/// value is a list of one.
fn entries(value: Option<&AttributeValue>) -> Option<Vec<&AttributeValue>> {
    match value? {
        AttributeValue::Array(items) => Some(items.iter().collect()),
        single => Some(vec![single]),
    }
}

/// Whether `value` can take a `_code`/`_codeSpace` sibling. A value that keeps
/// its own `@codeSpace` is a code the reader could not resolve, so it has none.
fn takes_code(value: &AttributeValue) -> bool {
    match value {
        AttributeValue::Array(_) | AttributeValue::Bytes(_) => false,
        AttributeValue::Map(map) => !map.contains_key("@codeSpace"),
        _ => true,
    }
}

/// One element, or `None` when the value has nothing to write.
fn element(
    prefix: &str,
    local: &str,
    value: &AttributeValue,
    siblings: &Siblings,
    out: &mut Converted,
) -> Option<XmlProperty> {
    let mut property = XmlProperty::new(prefix, local);
    match value {
        AttributeValue::Map(map) => fill_from_map(&mut property, map, out),
        AttributeValue::Bytes(_) | AttributeValue::Array(_) => {
            out.skipped.push(Skip::NotPlaced);
            return None;
        }
        scalar => property.text = scalar_text(scalar).filter(|text| !text.is_empty()),
    }
    if let Some(code) = siblings.code.and_then(scalar_text) {
        property.text = Some(code);
    }
    if let Some(url) = siblings.code_space.and_then(scalar_text) {
        if set_attr(&mut property, "codeSpace", &url) && !out.code_spaces.contains(&url) {
            out.code_spaces.push(url);
        }
    }
    if let Some(uom) = siblings.uom.and_then(scalar_text) {
        set_attr(&mut property, "uom", &uom);
    }
    let empty =
        property.text.is_none() && property.attrs.is_empty() && property.children.is_empty();
    (!empty).then_some(property)
}

/// Adds the attribute unless the value already carries one of that name,
/// which then wins. Returns whether it was added.
fn set_attr(property: &mut XmlProperty, name: &str, value: &str) -> bool {
    if property.attr(name).is_some() {
        return false;
    }
    property.attrs.push((name.to_owned(), value.to_owned()));
    true
}

fn fill_from_map(property: &mut XmlProperty, map: &Attributes, out: &mut Converted) {
    for (key, value) in map {
        let key = key.as_str();
        if key == "$" {
            property.text = scalar_text(value).filter(|text| !text.is_empty());
        } else if let Some(name) = key.strip_prefix('@') {
            if name == "xmlns" || name.starts_with("xmlns:") {
                continue;
            }
            match scalar_text(value) {
                Some(text) if is_writable_attribute(name) => {
                    property.attrs.push((name.to_owned(), text))
                }
                _ => out.skipped.push(Skip::NotPlaced),
            }
        } else if is_sibling(key, map) {
            continue;
        } else {
            match key.split_once(':') {
                Some((prefix, local)) if is_known_name(prefix, local) => {
                    let siblings = Siblings::of(key, map);
                    let mut children = std::mem::take(&mut property.children);
                    push_elements(prefix, local, value, &siblings, &mut children, out);
                    property.children = children;
                }
                _ => out.skipped.push(Skip::NotPlaced),
            }
        }
    }
}

/// Whether `name` can be written as an XML attribute: an `NCName`, or a
/// `prefix:local` pair of them whose prefix is bound. `xml` is bound by XML
/// itself and never declared, so `xml:lang` and its kin pass without a table
/// entry.
fn is_writable_attribute(name: &str) -> bool {
    match name.split_once(':') {
        Some((prefix, local)) => {
            is_ncname(prefix)
                && is_ncname(local)
                && (prefix == "xml" || namespace_for_prefix(prefix).is_some())
        }
        None => is_ncname(name),
    }
}

fn scalar_text(value: &AttributeValue) -> Option<String> {
    match value {
        AttributeValue::String(text) => Some(text.clone()),
        AttributeValue::Number(number) => Some(format_number(number)),
        AttributeValue::Bool(flag) => Some(flag.to_string()),
        AttributeValue::DateTime(datetime) => Some(datetime.to_raw()),
        _ => None,
    }
}

/// A number as `xs:integer` and `xs:double` both accept it: an integral value
/// prints without a fraction, so a reader's `3.0` goes back out as `3`.
pub fn format_number(number: &serde_json::Number) -> String {
    if let Some(integer) = number.as_i64() {
        return integer.to_string();
    }
    if let Some(integer) = number.as_u64() {
        return integer.to_string();
    }
    // `f64`'s `Display` never adds a fraction to an integral value, and a
    // `serde_json::Number` is never NaN or infinite.
    number
        .as_f64()
        .map(|float| float.to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use reearth_flow_types::{AttributeValue, Attributes};
    use serde_json::json;

    use super::*;

    fn attrs(value: serde_json::Value) -> Attributes {
        match AttributeValue::from(value) {
            AttributeValue::Map(map) => map,
            other => panic!("expected a map, got {other:?}"),
        }
    }

    fn convert(value: serde_json::Value) -> Converted {
        from_attributes(&attrs(value), &HashSet::new())
    }

    fn names(properties: &[XmlProperty]) -> Vec<String> {
        properties
            .iter()
            .map(|p| format!("{}:{}", p.prefix, p.local))
            .collect()
    }

    #[test]
    fn a_plain_text_property_becomes_one_element() {
        let out = convert(json!({ "bldg:storeysAboveGround": "3" }));
        assert_eq!(names(&out.properties), ["bldg:storeysAboveGround"]);
        assert_eq!(out.properties[0].text.as_deref(), Some("3"));
        assert!(out.skipped.is_empty());
    }

    #[test]
    fn unprefixed_reserved_and_never_written_keys_are_ignored_silently() {
        let out = convert(json!({
            "path": "a.gml", "lod": 2, "__citygml_gml_id": "b1", "@gml:id": "b1", "$": "x",
            "gml:boundedBy": { "gml:Envelope": {} }, "app:appearance": { "app:Appearance": {} }
        }));
        assert!(out.properties.is_empty());
        assert!(out.skipped.is_empty());
    }

    #[test]
    fn configured_keys_are_excluded() {
        let excluded: HashSet<&str> = ["bldg:kind"].into_iter().collect();
        let out = from_attributes(&attrs(json!({ "bldg:kind": "x" })), &excluded);
        assert!(out.properties.is_empty());
    }

    #[test]
    fn an_unknown_prefix_a_flattened_key_or_a_geometry_name_is_not_placed() {
        let out = convert(json!({
            "uro:buildingIDAttribute": { "uro:BuildingIDAttribute": {} },
            "uro:BuildingIDAttribute_uro:city": "x",
            "bldg:lod2Solid": "x",
            "dem:tin": "x"
        }));
        assert!(out.properties.is_empty());
        assert_eq!(out.skipped, vec![Skip::NotPlaced; 4]);
    }

    #[test]
    fn a_reader_map_inverts_text_and_xml_attributes() {
        let out = convert(json!({ "bldg:measuredHeight": { "@uom": "m", "$": "10.5" } }));
        let p = &out.properties[0];
        assert_eq!(p.attrs, vec![("uom".to_string(), "m".to_string())]);
        assert_eq!(p.text.as_deref(), Some("10.5"));
    }

    #[test]
    fn a_flattened_measure_takes_its_uom_sibling() {
        let out = convert(json!({ "bldg:measuredHeight": 10.0, "bldg:measuredHeight_uom": "m" }));
        assert_eq!(names(&out.properties), ["bldg:measuredHeight"]);
        let p = &out.properties[0];
        assert_eq!(p.text.as_deref(), Some("10"));
        assert_eq!(p.attr("uom"), Some("m"));
    }

    #[test]
    fn a_resolved_code_writes_the_code_and_its_codelist() {
        let out = convert(json!({
            "bldg:class": "普通建物",
            "bldg:class_code": "3001",
            "bldg:class_codeSpace": "file:///data/codelists/Building_class.xml"
        }));
        assert_eq!(names(&out.properties), ["bldg:class"]);
        let p = &out.properties[0];
        assert_eq!(p.text.as_deref(), Some("3001"));
        assert_eq!(
            p.attr("codeSpace"),
            Some("file:///data/codelists/Building_class.xml")
        );
        assert_eq!(
            out.code_spaces,
            ["file:///data/codelists/Building_class.xml"]
        );
    }

    #[test]
    fn a_code_without_a_kept_codelist_writes_the_code_alone() {
        let out = convert(json!({ "bldg:class": "普通建物", "bldg:class_code": "3001" }));
        let p = &out.properties[0];
        assert_eq!(p.text.as_deref(), Some("3001"));
        assert!(p.attrs.is_empty());
        assert!(out.code_spaces.is_empty());
    }

    #[test]
    fn an_unresolved_code_is_written_as_the_source_had_it_and_never_staged() {
        let out = convert(
            json!({ "bldg:class": { "@codeSpace": "../../codelists/Building_class.xml", "$": "3001" } }),
        );
        assert_eq!(
            out.properties[0].attr("codeSpace"),
            Some("../../codelists/Building_class.xml")
        );
        assert!(out.code_spaces.is_empty());
    }

    #[test]
    fn repeated_codes_pair_each_value_with_its_own_sibling() {
        let out = convert(json!({
            "bldg:usage": ["住宅", "店舗"],
            "bldg:usage_code": ["411", "401"],
            "bldg:usage_codeSpace": ["file:///c/Building_usage.xml", "file:///c/Building_usage.xml"]
        }));
        assert_eq!(names(&out.properties), ["bldg:usage", "bldg:usage"]);
        assert_eq!(out.properties[0].text.as_deref(), Some("411"));
        assert_eq!(out.properties[1].text.as_deref(), Some("401"));
        assert_eq!(out.code_spaces, ["file:///c/Building_usage.xml"]);
    }

    #[test]
    fn nested_maps_become_nested_elements_with_siblings_resolved_at_every_level() {
        let out = convert(json!({
            "gen:measureAttribute": { "@name": "高さ", "gen:value": 12.0, "gen:value_uom": "m" }
        }));
        let p = &out.properties[0];
        assert_eq!(p.attr("name"), Some("高さ"));
        assert_eq!(names(&p.children), ["gen:value"]);
        assert_eq!(p.children[0].attr("uom"), Some("m"));
        assert_eq!(p.children[0].text.as_deref(), Some("12"));
    }

    #[test]
    fn an_unknown_prefix_below_the_top_level_drops_only_that_element() {
        let out = convert(json!({
            "bldg:address": { "core:Address": {
                "foo:bar": "x",
                "core:xalAddress": { "xAL:AddressDetails": { "xAL:Country": { "xAL:CountryName": "日本" } } }
            } }
        }));
        let address = &out.properties[0].children[0];
        assert_eq!(names(&address.children), ["core:xalAddress"]);
        assert_eq!(out.skipped, vec![Skip::NotPlaced]);
    }

    #[test]
    fn a_generic_attribute_without_its_name_is_not_placed() {
        let out = convert(json!({
            "gen:stringAttribute": [{ "@name": "a", "gen:value": "x" }, { "gen:value": "y" }]
        }));
        assert_eq!(out.properties.len(), 1);
        assert_eq!(out.skipped, vec![Skip::NotPlaced]);
    }

    #[test]
    fn null_and_empty_values_write_nothing() {
        let out = convert(json!({ "bldg:function": null, "bldg:roofType": "" }));
        assert!(out.properties.is_empty());
        assert!(out.skipped.is_empty());
    }

    #[test]
    fn numbers_print_without_a_trailing_zero() {
        assert_eq!(format_number(&serde_json::Number::from(3)), "3");
        assert_eq!(
            format_number(&serde_json::Number::from_f64(3.0).unwrap()),
            "3"
        );
        assert_eq!(
            format_number(&serde_json::Number::from_f64(10.25).unwrap()),
            "10.25"
        );
    }

    #[test]
    fn xmlns_attributes_are_dropped_and_unknown_attribute_prefixes_are_not_placed() {
        let out = convert(
            json!({ "core:externalReference": { "@xmlns:foo": "u", "@foo:x": "1", "@xlink:href": "#a" } }),
        );
        let p = &out.properties[0];
        assert_eq!(p.attrs, vec![("xlink:href".to_string(), "#a".to_string())]);
        assert_eq!(out.skipped, vec![Skip::NotPlaced]);
    }

    #[test]
    fn an_attribute_name_that_is_not_a_valid_xml_name_is_not_placed() {
        let out = convert(json!({
            "core:externalReference": { "@gml:bad:name": "1", "@bad name": "2", "@xlink:href": "#a" }
        }));
        let p = &out.properties[0];
        assert_eq!(p.attrs, vec![("xlink:href".to_string(), "#a".to_string())]);
        assert_eq!(out.skipped, vec![Skip::NotPlaced, Skip::NotPlaced]);
    }

    #[test]
    fn the_predefined_xml_prefix_is_kept_without_a_declaration() {
        let out = convert(json!({ "gml:name": { "@xml:lang": "ja", "$": "東京" } }));
        assert_eq!(out.properties[0].attr("xml:lang"), Some("ja"));
        assert!(out.skipped.is_empty());
    }

    #[test]
    fn a_scalar_code_pairs_only_with_the_value_that_has_no_codespace_of_its_own() {
        let out = convert(json!({
            "bldg:usage": ["住宅", { "@codeSpace": "../../codelists/Building_usage.xml", "$": "999" }],
            "bldg:usage_code": "411",
            "bldg:usage_codeSpace": "file:///c/Building_usage.xml"
        }));
        assert_eq!(names(&out.properties), ["bldg:usage", "bldg:usage"]);
        let (first, second) = (&out.properties[0], &out.properties[1]);
        assert_eq!(first.text.as_deref(), Some("411"));
        assert_eq!(
            first.attr("codeSpace"),
            Some("file:///c/Building_usage.xml")
        );
        assert_eq!(second.text.as_deref(), Some("999"));
        assert_eq!(
            second.attrs,
            vec![(
                "codeSpace".to_string(),
                "../../codelists/Building_usage.xml".to_string()
            )]
        );
        assert_eq!(out.code_spaces, ["file:///c/Building_usage.xml"]);
        assert!(out.skipped.is_empty());
    }

    #[test]
    fn an_unresolved_value_in_the_middle_keeps_its_own_code() {
        let out = convert(json!({
            "bldg:usage": [
                "a",
                { "@codeSpace": "../c.xml", "$": "999" },
                "b"
            ],
            "bldg:usage_code": ["411", "401"],
            "bldg:usage_codeSpace": ["file:///c/one.xml", "file:///c/two.xml"]
        }));
        let texts: Vec<_> = out.properties.iter().map(|p| p.text.as_deref()).collect();
        assert_eq!(texts, [Some("411"), Some("999"), Some("401")]);
        assert_eq!(
            out.properties[0].attr("codeSpace"),
            Some("file:///c/one.xml")
        );
        assert_eq!(out.properties[1].attr("codeSpace"), Some("../c.xml"));
        assert_eq!(out.properties[1].attrs.len(), 1);
        assert_eq!(
            out.properties[2].attr("codeSpace"),
            Some("file:///c/two.xml")
        );
        assert_eq!(out.code_spaces, ["file:///c/one.xml", "file:///c/two.xml"]);
    }

    #[test]
    fn code_entries_that_do_not_match_the_values_leave_the_property_out() {
        let out = convert(json!({ "bldg:usage": ["住宅", "999"], "bldg:usage_code": "411" }));
        assert!(out.properties.is_empty());
        assert_eq!(out.skipped, vec![Skip::NotPlaced]);
    }

    #[test]
    fn a_unit_applies_to_flattened_numbers_and_never_to_a_map_with_its_own() {
        let out = convert(json!({
            "bldg:measuredHeight": [10.5, { "@uom": "ft", "$": "x" }],
            "bldg:measuredHeight_uom": "m"
        }));
        assert_eq!(out.properties.len(), 2);
        assert_eq!(
            out.properties[0].attrs,
            vec![("uom".to_string(), "m".to_string())]
        );
        assert_eq!(
            out.properties[1].attrs,
            vec![("uom".to_string(), "ft".to_string())]
        );
    }
}
