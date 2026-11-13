use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;
use reearth_flow_types::{Attribute, AttributeValue};
use url::Url;

use super::utils::{ElementGroup, XmlChild, XmlNode, local_name};

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

    /// Returns the label of `code` in the codelist `code_space` names, resolved against `source_url`.
    fn lookup(&mut self, source_url: &Url, code_space: &str, code: &str) -> Option<String> {
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
        dict.get(code).cloned()
    }
}

/// A coded element found in its codelist.
pub struct Coded {
    label: String,
    code: String,
}

impl CodelistResolver {
    /// Looks `elem` up in the codelist its `codeSpace` names; `None` if it has no match.
    pub fn resolve(&mut self, elem: &XmlNode) -> Option<Coded> {
        let (_, code_space) = elem
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
        let label = self.lookup(&elem.source_url, code_space, code)?;
        Some(Coded {
            label,
            code: code.to_string(),
        })
    }
}

/// `hoist_attributes` with `codeSpace` in it when `keep_absolute_code_space` is set: keeping it means hoisting it.
pub(super) fn hoisting_code_space(
    hoist_attributes: &[String],
    keep_absolute_code_space: bool,
) -> Vec<String> {
    let mut hoist_attributes = hoist_attributes.to_vec();
    if keep_absolute_code_space && !hoist_attributes.iter().any(|a| a == "codeSpace") {
        hoist_attributes.push("codeSpace".to_string());
    }
    hoist_attributes
}

impl Coded {
    /// Adds the element with the label in place of its code, which goes in the `_code` sibling beside `hoisted`.
    pub(super) fn push_into(
        self,
        group: &mut ElementGroup,
        mut value: AttributeValue,
        mut hoisted: Vec<(String, String)>,
    ) {
        match &mut value {
            AttributeValue::Map(map) => {
                map.shift_remove("$");
                if map.is_empty() {
                    value = AttributeValue::String(self.label);
                } else {
                    map.insert(Attribute::new("$"), AttributeValue::String(self.label));
                }
            }
            _ => value = AttributeValue::String(self.label),
        }
        hoisted.push(("_code".to_string(), self.code));
        group.push(value, hoisted);
    }
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

    use reearth_flow_types::Attributes;

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

    /// A `bldg:Building` with one `bldg:usage` per entry of `codes`, coded against `code_space`.
    fn building_with_usages(codes: &[&str], code_space: &str, source_url: &Arc<Url>) -> XmlNode {
        let children = codes
            .iter()
            .map(|code| {
                XmlChild::Element(Arc::new(XmlNode {
                    name: ("bldg:usage".to_string(), EMPTY_NS_ID),
                    attrs: vec![
                        (
                            ("codeSpace".to_string(), EMPTY_NS_ID),
                            code_space.to_string(),
                        ),
                        (("xml:lang".to_string(), EMPTY_NS_ID), "ja".to_string()),
                    ],
                    children: vec![XmlChild::Text(code.to_string())],
                    source_url: source_url.clone(),
                }))
            })
            .collect();
        XmlNode {
            name: ("bldg:Building".to_string(), EMPTY_NS_ID),
            attrs: vec![],
            children,
            source_url: source_url.clone(),
        }
    }

    /// The attributes the parser builds from `node`, hoisting the attributes named in `hoist`.
    fn build(node: &XmlNode, hoist: &[&str], keep_absolute_code_space: bool) -> Attributes {
        let hoist: Vec<String> = hoist.iter().map(|a| a.to_string()).collect();
        let hoist = hoisting_code_space(&hoist, keep_absolute_code_space);
        let value = crate::parser::node_to_attribute_value(
            node,
            &hoist,
            &mut CodelistResolver::new(),
            keep_absolute_code_space,
            #[cfg(feature = "new-geometry")]
            &mut Vec::new(),
        );
        let AttributeValue::Map(map) = value else {
            panic!("expected a map, got {value:?}");
        };
        map
    }

    fn s(v: &str) -> AttributeValue {
        AttributeValue::String(v.to_string())
    }

    /// A codelist (`201`, `202`), a document two levels below it, and the relative `codeSpace`.
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

    // Keeping the code space hoists it, once, whether or not it was listed.
    #[test]
    fn keeping_the_code_space_adds_it_to_the_hoisted_attributes_once() {
        let listed = vec!["uom".to_string(), "codeSpace".to_string()];

        assert_eq!(hoisting_code_space(&listed, true), listed);
        assert_eq!(hoisting_code_space(&listed, false), listed);
        assert_eq!(
            hoisting_code_space(&["uom".to_string()], true),
            ["uom", "codeSpace"]
        );
        assert_eq!(hoisting_code_space(&["uom".to_string()], false), ["uom"]);
    }

    fn keys(map: &Attributes) -> Vec<&str> {
        map.keys().map(|k| k.as_str()).collect()
    }

    // The label replaces the code, which is kept in `{name}_code` right after it.
    #[test]
    fn resolves_label_and_code() {
        let (_dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["201"], &code_space, &source_url);

        let map = build(&node, &[], false);

        let entries: Vec<_> = map.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        assert_eq!(
            entries,
            vec![
                ("bldg:usage", s("Residential")),
                ("bldg:usage_code", s("201")),
            ]
        );
    }

    // A listed `codeSpace` is hoisted as written, whether or not its code resolved.
    #[test]
    fn hoists_the_listed_code_space_as_written_beside_the_code() {
        let (_dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["201", "999"], &code_space, &source_url);

        let map = build(&node, &["codeSpace"], false);

        assert_eq!(
            keys(&map),
            vec!["bldg:usage", "bldg:usage@codeSpace", "bldg:usage_code"]
        );
        assert_eq!(
            map["bldg:usage@codeSpace"],
            AttributeValue::Array(vec![s(&code_space), s(&code_space)])
        );
    }

    // Resolving the code does not cost the element the other attributes it was asked to hoist.
    #[test]
    fn a_resolved_element_keeps_its_other_hoisted_attributes() {
        let (_dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["201"], &code_space, &source_url);

        let map = build(&node, &["xml:lang"], false);

        assert_eq!(
            keys(&map),
            vec!["bldg:usage", "bldg:usage@xml:lang", "bldg:usage_code"]
        );
        assert_eq!(map["bldg:usage@xml:lang"], s("ja"));
    }

    // The flag hoists `codeSpace` without it being listed, and as the URL it resolves to.
    #[test]
    fn keeping_the_code_space_hoists_it_as_an_absolute_url() {
        let (dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["201"], &code_space, &source_url);

        let map = build(&node, &[], true);

        assert_eq!(
            keys(&map),
            vec!["bldg:usage", "bldg:usage@codeSpace", "bldg:usage_code"]
        );
        assert_eq!(
            map["bldg:usage@codeSpace"],
            s(Url::from_file_path(dict.path()).unwrap().as_str())
        );
    }

    // Listed and kept: one key, holding the absolute URL rather than the text as written.
    #[test]
    fn keeping_a_listed_code_space_replaces_its_value() {
        let (dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["201"], &code_space, &source_url);

        let map = build(&node, &["codeSpace"], true);

        assert_eq!(
            keys(&map),
            vec!["bldg:usage", "bldg:usage@codeSpace", "bldg:usage_code"]
        );
        assert_eq!(
            map["bldg:usage@codeSpace"],
            s(Url::from_file_path(dict.path()).unwrap().as_str())
        );
    }

    // An unresolved element keeps its value, and a lone one gets no `_code` at all.
    #[test]
    fn leaves_a_lone_unresolved_element_untouched() {
        let (_dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["999"], &code_space, &source_url);

        let map = build(&node, &[], false);

        assert_eq!(map.len(), 1);
        assert_eq!(map["bldg:usage"], s("999"));
    }

    // The codelist does not have to resolve the code for its location to be absolute.
    #[test]
    fn keeping_the_code_space_does_not_need_the_code_to_resolve() {
        let (dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["999"], &code_space, &source_url);

        let map = build(&node, &[], true);

        assert_eq!(keys(&map), vec!["bldg:usage", "bldg:usage@codeSpace"]);
        assert_eq!(map["bldg:usage"], s("999"));
        assert_eq!(
            map["bldg:usage@codeSpace"],
            s(Url::from_file_path(dict.path()).unwrap().as_str())
        );
    }

    // An unresolved element keeps its value and gets a `Null` in `_code`, so the arrays stay aligned.
    #[test]
    fn pads_unresolved_elements_with_null_to_stay_aligned() {
        let (dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["999", "201", "999", "202"], &code_space, &source_url);

        let map = build(&node, &[], true);

        assert_eq!(
            keys(&map),
            vec!["bldg:usage", "bldg:usage@codeSpace", "bldg:usage_code"]
        );
        assert_eq!(
            map["bldg:usage"],
            AttributeValue::Array(vec![s("999"), s("Residential"), s("999"), s("Commercial"),])
        );
        let null = AttributeValue::Null;
        assert_eq!(
            map["bldg:usage_code"],
            AttributeValue::Array(vec![null.clone(), s("201"), null.clone(), s("202")])
        );
        let url = s(Url::from_file_path(dict.path()).unwrap().as_str());
        assert_eq!(
            map["bldg:usage@codeSpace"],
            AttributeValue::Array(vec![url.clone(), url.clone(), url.clone(), url])
        );
    }
}
