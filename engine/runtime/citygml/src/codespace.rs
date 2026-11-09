use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;
use reearth_flow_types::{Attribute, AttributeValue};
use url::Url;

use super::utils::{local_name, ElementGroup, XmlChild, XmlNode};

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

/// A coded element found in its codelist.
pub struct Coded {
    label: String,
    code: String,
    /// The codelist the code was found in, resolved against the source document.
    dict_url: Url,
    /// The key the element's `codeSpace` has among its attributes (`@codeSpace`).
    code_space_key: String,
}

impl CodelistResolver {
    /// Looks `elem` up in the codelist its `codeSpace` names; `None` if it has no match.
    pub fn resolve(&mut self, elem: &XmlNode) -> Option<Coded> {
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
        let (label, dict_url) = self.lookup(&elem.source_url, code_space, code)?;
        Some(Coded {
            label,
            code: code.to_string(),
            dict_url,
            code_space_key: format!("@{qname}"),
        })
    }
}

impl Coded {
    /// Adds the element with the label in place of its code, hoisting `code` (and `codeSpace`).
    pub(super) fn push_into(
        self,
        group: &mut ElementGroup,
        mut value: AttributeValue,
        resolve_code_space: bool,
    ) {
        match &mut value {
            AttributeValue::Map(map) => {
                map.shift_remove(self.code_space_key.as_str());
                map.shift_remove("$");
                if map.is_empty() {
                    value = AttributeValue::String(self.label);
                } else {
                    map.insert(Attribute::new("$"), AttributeValue::String(self.label));
                }
            }
            _ => value = AttributeValue::String(self.label),
        }
        let hoisted = [
            ("code", self.code.as_str()),
            ("codeSpace", self.dict_url.as_str()),
        ];
        group.push(value, &hoisted[..if resolve_code_space { 2 } else { 1 }]);
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
                    attrs: vec![(
                        ("codeSpace".to_string(), EMPTY_NS_ID),
                        code_space.to_string(),
                    )],
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

    /// The attributes the parser builds from `node`.
    fn build(node: &XmlNode, resolve_code_space: bool) -> Attributes {
        let value = crate::parser::node_to_attribute_value(
            node,
            true,
            &[],
            &mut CodelistResolver::new(),
            resolve_code_space,
            #[cfg(feature = "new-geometry")]
            &mut Vec::new(),
        );
        let AttributeValue::Map(map) = value else {
            panic!("expected a map, got {value:?}");
        };
        map
    }

    /// An element `build` leaves as it was: its `codeSpace` still attached.
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

    // The label replaces the code, which is kept in `{name}_code` right after it.
    #[test]
    fn resolves_label_and_code() {
        let (_dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["201"], &code_space, &source_url);

        let map = build(&node, false);

        let entries: Vec<_> = map.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        assert_eq!(
            entries,
            vec![
                ("bldg:usage", s("Residential")),
                ("bldg:usage_code", s("201")),
            ]
        );
    }

    // What is kept is the URL the lookup opened, not the relative `codeSpace`.
    #[test]
    fn keeps_the_resolved_codelist_url_after_the_code() {
        let (dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["201"], &code_space, &source_url);

        let map = build(&node, true);

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
        let node = building_with_usages(&["999"], &code_space, &source_url);

        let map = build(&node, true);

        assert_eq!(map.len(), 1);
        assert_eq!(map["bldg:usage"], raw_usage("999", &code_space));
    }

    // An unresolved element keeps its value and gets a `Null` in `_code` and `_codeSpace`.
    #[test]
    fn pads_unresolved_elements_with_null_to_stay_aligned() {
        let (dict, source_url, code_space) = dataset();
        let node = building_with_usages(&["999", "201", "999", "202"], &code_space, &source_url);

        let map = build(&node, true);

        let keys: Vec<_> = map.keys().map(|k| k.as_str()).collect();
        assert_eq!(
            keys,
            vec!["bldg:usage", "bldg:usage_code", "bldg:usage_codeSpace"]
        );
        assert_eq!(
            map["bldg:usage"],
            AttributeValue::Array(vec![
                raw_usage("999", &code_space),
                s("Residential"),
                raw_usage("999", &code_space),
                s("Commercial"),
            ])
        );
        let null = AttributeValue::Null;
        assert_eq!(
            map["bldg:usage_code"],
            AttributeValue::Array(vec![null.clone(), s("201"), null.clone(), s("202")])
        );
        let url = s(Url::from_file_path(dict.path()).unwrap().as_str());
        assert_eq!(
            map["bldg:usage_codeSpace"],
            AttributeValue::Array(vec![null.clone(), url.clone(), null, url])
        );
    }

    // Resolving replaces the code with its label, not the element's other attributes.
    #[test]
    fn keeps_the_other_attributes_of_a_resolved_element() {
        let (_dict, source_url, code_space) = dataset();
        let mut node = building_with_usages(&["201"], &code_space, &source_url);
        let XmlChild::Element(usage) = &mut node.children[0] else {
            unreachable!()
        };
        Arc::get_mut(usage)
            .unwrap()
            .attrs
            .push((("note".to_string(), EMPTY_NS_ID), "n".to_string()));

        let map = build(&node, false);

        assert_eq!(
            map["bldg:usage"],
            AttributeValue::Map(Attributes::from_iter([
                (Attribute::new("@note"), s("n")),
                (Attribute::new("$"), s("Residential")),
            ]))
        );
    }
}
