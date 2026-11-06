use std::collections::HashMap;

use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::Reader;
use reearth_flow_types::{Attribute, AttributeValue, Attributes};
use url::Url;

use super::utils::{local_name, XmlChild, XmlNode};

pub struct CodelistResolver {
    /// `None` marks a codelist that failed to load; the cause was already reported.
    cache: HashMap<Url, Option<HashMap<String, String>>>,
}

impl CodelistResolver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Returns the label and the URL of the codelist it came from, so a caller
    /// keeping the codelist gets the one that was actually opened.
    fn lookup(&mut self, source_url: &Url, code_space: &str, code: &str) -> Option<(String, Url)> {
        // codeSpace is joined without restricting absolute URLs or path traversal.
        // This is intentional: the engine assumes trusted input GML running in a controlled environment,
        // consistent with how xlink resolution and zip extraction are handled elsewhere. See PR #2066.
        let dict_url = match source_url.join(code_space) {
            Ok(u) => u,
            Err(e) => {
                tracing::error!(
                    source_url = source_url.as_str(),
                    code_space,
                    "failed to join codeSpace URL: {e}"
                );
                return None;
            }
        };
        let dict = self
            .cache
            .entry(dict_url.clone())
            .or_insert_with(|| load_dictionary(&dict_url));
        // The codelist itself is unavailable; reporting a missing code on top would only duplicate.
        let dict = dict.as_ref()?;
        if !dict.contains_key(code) {
            tracing::error!(
                dict_url = dict_url.as_str(),
                code,
                "code not found in codelist"
            );
        }
        dict.get(code).cloned().map(|label| (label, dict_url))
    }
}

/// Resolves the coded elements of `node` in the attributes already built from it
/// (`value`), in place.
///
/// For each element name, the node's same-named child elements line up one to one
/// with the values `value` holds under that name. A resolved element's value
/// becomes its label, and `{name}_code` (plus `{name}_codeSpace`, the codelist's
/// resolved URL, when `keep_code_space`) is inserted right after it with one entry
/// per element, `Null` for those that did not resolve. When none resolve, nothing
/// is added.
pub fn resolve_attributes(
    node: &XmlNode,
    value: &mut AttributeValue,
    resolver: &mut CodelistResolver,
    keep_code_space: bool,
) {
    let AttributeValue::Map(map) = value else {
        return;
    };
    let mut groups: IndexMap<&str, Vec<&XmlNode>> = IndexMap::new();
    for child in &node.children {
        if let XmlChild::Element(e) = child {
            groups.entry(e.name.0.as_str()).or_default().push(e);
        }
    }
    for (name, elems) in groups {
        let Some(slot) = map.get_mut(name) else {
            continue;
        };
        let values = match slot {
            AttributeValue::Array(values) => values.as_mut_slice(),
            single => std::slice::from_mut(single),
        };
        if values.len() != elems.len() {
            tracing::error!(name, "codeSpace: element and attribute counts differ");
            continue;
        }
        let mut codes = Vec::with_capacity(elems.len());
        let mut spaces = Vec::with_capacity(elems.len());
        for (elem, value) in elems.iter().zip(values) {
            match resolve_element(elem, resolver) {
                Some(coded) => {
                    set_label(value, coded.label, &coded.code_space_key);
                    codes.push(AttributeValue::String(coded.code));
                    spaces.push(AttributeValue::String(coded.dict_url.to_string()));
                }
                None => {
                    codes.push(AttributeValue::Null);
                    spaces.push(AttributeValue::Null);
                    resolve_attributes(elem, value, resolver, keep_code_space);
                }
            }
        }
        if codes.iter().all(|c| matches!(c, AttributeValue::Null)) {
            continue;
        }
        let at = map.get_index_of(name).map_or(map.len(), |i| i + 1);
        insert_column(map, at, format!("{name}_code"), codes);
        if keep_code_space {
            insert_column(map, at + 1, format!("{name}_codeSpace"), spaces);
        }
    }
}

struct CodedElement {
    label: String,
    code: String,
    dict_url: Url,
    /// The key the element's `codeSpace` attribute has in its attributes (`@codeSpace`).
    code_space_key: String,
}

/// `None` unless `elem` has a `codeSpace` and non-empty text that the codelist knows.
fn resolve_element(elem: &XmlNode, resolver: &mut CodelistResolver) -> Option<CodedElement> {
    let ((qname, _), code_space) = elem
        .attrs
        .iter()
        .find(|((q, _), _)| local_name(q) == "codeSpace")?;
    let text: String = elem
        .children
        .iter()
        .filter_map(|c| match c {
            XmlChild::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    let code = text.trim();
    if code.is_empty() {
        return None;
    }
    let (label, dict_url) = resolver.lookup(&elem.source_url, code_space, code)?;
    Some(CodedElement {
        label,
        code: code.to_string(),
        dict_url,
        code_space_key: format!("@{qname}"),
    })
}

/// Makes `label` the element's value, dropping the `codeSpace` it was resolved from.
fn set_label(value: &mut AttributeValue, label: String, code_space_key: &str) {
    if let AttributeValue::Map(map) = value {
        map.shift_remove(code_space_key);
        map.shift_remove("$");
        if !map.is_empty() {
            map.insert(Attribute::new("$"), AttributeValue::String(label));
            return;
        }
    }
    *value = AttributeValue::String(label);
}

/// One value per element, or just the value when there is a single element.
fn insert_column(map: &mut Attributes, at: usize, key: String, mut column: Vec<AttributeValue>) {
    let value = if column.len() == 1 {
        column.remove(0)
    } else {
        AttributeValue::Array(column)
    };
    map.shift_insert(at, Attribute::new(key), value);
}

fn load_dictionary(url: &Url) -> Option<HashMap<String, String>> {
    let path = match url.to_file_path() {
        Ok(p) => p,
        Err(_) => {
            tracing::error!(url = url.as_str(), "codeSpace URL is not a file path");
            return None;
        }
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(path = %path.display(), "failed to read codelist file: {e}");
            return None;
        }
    };
    let src = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(path = %path.display(), "codelist file is not valid UTF-8: {e}");
            return None;
        }
    };

    let mut reader = Reader::from_str(src);
    let mut map = HashMap::new();
    let mut name_buf = String::new();
    let mut desc_buf = String::new();
    let mut in_name = false;
    let mut in_desc = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match local_name(&String::from_utf8_lossy(e.name().as_ref())) {
                "name" => in_name = true,
                "description" => in_desc = true,
                "Definition" => {
                    name_buf.clear();
                    desc_buf.clear();
                }
                _ => {}
            },
            Ok(Event::Text(t)) => {
                if in_name {
                    match t.unescape() {
                        Ok(s) => name_buf = s.trim().to_string(),
                        Err(e) => tracing::error!(
                            url = url.as_str(),
                            "failed to unescape codelist <name> text: {e}"
                        ),
                    }
                } else if in_desc {
                    match t.unescape() {
                        Ok(s) => desc_buf = s.trim().to_string(),
                        Err(e) => tracing::error!(
                            url = url.as_str(),
                            "failed to unescape codelist <description> text: {e}"
                        ),
                    }
                }
            }
            Ok(Event::End(e)) => match local_name(&String::from_utf8_lossy(e.name().as_ref())) {
                "name" => in_name = false,
                "description" => in_desc = false,
                "Definition" if !name_buf.is_empty() && !desc_buf.is_empty() => {
                    map.insert(name_buf.clone(), desc_buf.clone());
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::error!(url = url.as_str(), "error parsing codelist XML: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Some(map)
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;

    use tempfile::NamedTempFile;

    use super::*;
    use crate::utils::EMPTY_NS_ID;

    fn write_dict(entries: &[(&str, &str)]) -> NamedTempFile {
        let defs: String = entries
            .iter()
            .map(|(name, desc)| {
                format!(
                    "<Definition><description>{desc}</description><name>{name}</name></Definition>"
                )
            })
            .collect();
        let xml = format!("<Dictionary>{defs}</Dictionary>");
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{xml}").unwrap();
        f
    }

    fn dict_url(f: &NamedTempFile) -> Url {
        Url::from_file_path(f.path()).unwrap()
    }

    // The key is <name> text and the value is <description> text — not the other way around.
    #[test]
    fn load_dictionary_key_is_name_value_is_description() {
        let f = write_dict(&[("201", "Residential"), ("202", "Commercial")]);
        let map = load_dictionary(&dict_url(&f)).unwrap();
        assert_eq!(map.get("201").map(String::as_str), Some("Residential"));
        assert_eq!(map.get("202").map(String::as_str), Some("Commercial"));
        assert_eq!(map.len(), 2);
    }

    // A Definition missing either <name> or <description> must be dropped.
    #[test]
    fn load_dictionary_drops_incomplete_definitions() {
        let xml = "<Dictionary>\
            <Definition><name>ok</name><description>Good</description></Definition>\
            <Definition><name>no-desc</name></Definition>\
            <Definition><description>No name</description></Definition>\
        </Dictionary>";
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{xml}").unwrap();
        let map = load_dictionary(&dict_url(&f)).unwrap();
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("ok"));
    }

    /// A `bldg:Building` holding one `bldg:usage` per entry of `codes`, each coded
    /// against `code_space`, and the attributes `node_to_attribute_value` builds
    /// from it (`{"@codeSpace": .., "$": code}` per element).
    fn building_with_usages(
        codes: &[&str],
        code_space: &str,
        source_url: &Arc<Url>,
    ) -> (XmlNode, AttributeValue) {
        let children = codes
            .iter()
            .map(|code| {
                XmlChild::Element(Arc::new(XmlNode {
                    name: ("bldg:usage".to_string(), EMPTY_NS_ID),
                    attrs: vec![(
                        ("codeSpace".to_string(), EMPTY_NS_ID),
                        code_space.to_string(),
                    )],
                    children: vec![XmlChild::Text(code.to_string())],
                    source_url: source_url.clone(),
                }))
            })
            .collect();
        let node = XmlNode {
            name: ("bldg:Building".to_string(), EMPTY_NS_ID),
            attrs: vec![],
            children,
            source_url: source_url.clone(),
        };
        let usages: Vec<AttributeValue> = codes.iter().map(|c| raw_usage(c, code_space)).collect();
        let usage = match usages.len() {
            1 => usages.into_iter().next().unwrap(),
            _ => AttributeValue::Array(usages),
        };
        let value = AttributeValue::Map(Attributes::from_iter([(
            Attribute::new("bldg:usage"),
            usage,
        )]));
        (node, value)
    }

    fn raw_usage(code: &str, code_space: &str) -> AttributeValue {
        AttributeValue::Map(Attributes::from_iter([
            (
                Attribute::new("@codeSpace"),
                AttributeValue::String(code_space.to_string()),
            ),
            (
                Attribute::new("$"),
                AttributeValue::String(code.to_string()),
            ),
        ]))
    }

    fn s(v: &str) -> AttributeValue {
        AttributeValue::String(v.to_string())
    }

    /// A codelist holding `201` and `202`, and a source document two levels below
    /// it that names it relatively, the way a PLATEAU dataset is laid out.
    /// Returns the dictionary file (kept alive), the document URL and the relative
    /// `codeSpace`.
    fn dataset() -> (NamedTempFile, Arc<Url>, String) {
        let dict_file = write_dict(&[("201", "Residential"), ("202", "Commercial")]);
        let name = dict_file.path().file_name().unwrap().to_str().unwrap();
        let parent = dict_file.path().parent().unwrap();
        let source_url = Arc::new(
            Url::from_file_path(parent.join("udx").join("bldg").join("source.gml")).unwrap(),
        );
        let relative = format!("../../{name}");
        (dict_file, source_url, relative)
    }

    fn building_map(value: &AttributeValue) -> &Attributes {
        let AttributeValue::Map(map) = value else {
            panic!("expected a map, got {value:?}");
        };
        map
    }

    // A resolved element's value becomes its label (its `codeSpace` dropped), with
    // the original code in `{name}_code` right after it.
    #[test]
    fn resolves_label_and_code() {
        let (_dict, source_url, code_space) = dataset();
        let (node, mut value) = building_with_usages(&["201"], &code_space, &source_url);

        resolve_attributes(&node, &mut value, &mut CodelistResolver::new(), false);

        let map = building_map(&value);
        let entries: Vec<_> = map.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        assert_eq!(
            entries,
            vec![
                ("bldg:usage", s("Residential")),
                ("bldg:usage_code", s("201")),
            ]
        );
    }

    // The codelist location is what a writer needs to re-emit `codeSpace`. The
    // relative form only means something beside the source document, so what is
    // kept is the URL the lookup actually opened.
    #[test]
    fn keeps_the_resolved_codelist_url_after_the_code() {
        let (dict, source_url, code_space) = dataset();
        let (node, mut value) = building_with_usages(&["201"], &code_space, &source_url);

        resolve_attributes(&node, &mut value, &mut CodelistResolver::new(), true);

        let map = building_map(&value);
        let keys: Vec<_> = map.keys().map(|k| k.as_str()).collect();
        assert_eq!(
            keys,
            vec!["bldg:usage", "bldg:usage_code", "bldg:usage_codeSpace"]
        );
        assert_eq!(
            map["bldg:usage_codeSpace"],
            s(Url::from_file_path(dict.path()).unwrap().as_str())
        );
    }

    // An unresolved element keeps its value, and a lone one gets no `_code` at all.
    #[test]
    fn leaves_a_lone_unresolved_element_untouched() {
        let (_dict, source_url, code_space) = dataset();
        let (node, mut value) = building_with_usages(&["999"], &code_space, &source_url);

        resolve_attributes(&node, &mut value, &mut CodelistResolver::new(), true);

        let map = building_map(&value);
        assert_eq!(map.len(), 1);
        assert_eq!(map["bldg:usage"], raw_usage("999", &code_space));
    }

    // `_code` stays index-aligned with its element: an unresolved one gets a `Null`
    // in the same position, and keeps its own value.
    #[test]
    fn pads_unresolved_elements_with_null_to_stay_aligned() {
        let (_dict, source_url, code_space) = dataset();
        let (node, mut value) =
            building_with_usages(&["201", "999", "202"], &code_space, &source_url);

        resolve_attributes(&node, &mut value, &mut CodelistResolver::new(), false);

        let map = building_map(&value);
        let keys: Vec<_> = map.keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["bldg:usage", "bldg:usage_code"]);
        assert_eq!(
            map["bldg:usage"],
            AttributeValue::Array(vec![
                s("Residential"),
                raw_usage("999", &code_space),
                s("Commercial"),
            ])
        );
        assert_eq!(
            map["bldg:usage_code"],
            AttributeValue::Array(vec![s("201"), AttributeValue::Null, s("202")])
        );
    }
}
