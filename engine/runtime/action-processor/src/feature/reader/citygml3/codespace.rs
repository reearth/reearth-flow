use std::collections::HashMap;
use std::sync::Arc;

use quick_xml::events::Event;
use quick_xml::Reader;
use url::Url;

use super::utils::{local_name, XmlChild, XmlNode, EMPTY_NS_ID};

pub struct CodelistResolver {
    cache: HashMap<Url, HashMap<String, String>>,
}

impl CodelistResolver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn lookup(&mut self, source_url: &Url, code_space: &str, code: &str) -> Option<String> {
        let dict_url = match source_url.join(code_space) {
            Ok(u) => u,
            Err(e) => {
                tracing::error!(
                    source_url = source_url.as_str(),
                    code_space,
                    "citygml3: failed to join codeSpace URL: {e}"
                );
                return None;
            }
        };
        if !self.cache.contains_key(&dict_url) {
            let dict = match load_dictionary(&dict_url) {
                Some(d) => d,
                None => {
                    tracing::error!(
                        dict_url = dict_url.as_str(),
                        "citygml3: failed to load codelist dictionary"
                    );
                    HashMap::new()
                }
            };
            self.cache.insert(dict_url.clone(), dict);
        }
        let dict = self.cache.get(&dict_url)?;
        if !dict.contains_key(code) {
            tracing::error!(
                dict_url = dict_url.as_str(),
                code,
                "citygml3: code not found in codelist"
            );
        }
        dict.get(code).cloned()
    }
}

/// Walks the fully xlink-resolved `XmlNode` tree and resolves `codeSpace` attributes.
/// Because nodes carry their own `source_url`, no external URL tracking is needed.
pub fn resolve(nodes: Vec<Arc<XmlNode>>, resolver: &mut CodelistResolver) -> Vec<Arc<XmlNode>> {
    nodes
        .into_iter()
        .map(|node| resolve_node(node, resolver))
        .collect()
}

fn resolve_node(node: Arc<XmlNode>, resolver: &mut CodelistResolver) -> Arc<XmlNode> {
    match resolve_children(&node.children, resolver) {
        None => node,
        Some(children) => node.with_children(children),
    }
}

/// Returns `None` if no children changed (caller keeps original node unchanged).
fn resolve_children(
    children: &[XmlChild],
    resolver: &mut CodelistResolver,
) -> Option<Vec<XmlChild>> {
    let mut out: Option<Vec<XmlChild>> = None;

    for (i, child) in children.iter().enumerate() {
        match child {
            XmlChild::Element(e) => {
                let code_space = e
                    .attrs
                    .iter()
                    .find(|((q, _), _)| local_name(q) == "codeSpace")
                    .map(|(_, v)| v.clone());

                if let Some(cs) = code_space {
                    let text: String = e
                        .children
                        .iter()
                        .filter_map(|c| match c {
                            XmlChild::Text(t) => Some(t.as_str()),
                            _ => None,
                        })
                        .collect();
                    let trimmed = text.trim();

                    if !trimmed.is_empty() {
                        if let Some(label) = resolver.lookup(&e.source_url, &cs, trimmed) {
                            if out.is_none() {
                                out = Some(children[..i].to_vec());
                            }
                            let nc = out.as_mut().unwrap();
                            nc.push(XmlChild::Element(Arc::new(XmlNode {
                                name: e.name.clone(),
                                attrs: e
                                    .attrs
                                    .iter()
                                    .filter(|((q, _), _)| local_name(q) != "codeSpace")
                                    .cloned()
                                    .collect(),
                                children: vec![XmlChild::Text(label)],
                                source_url: e.source_url.clone(),
                            })));
                            nc.push(XmlChild::Element(Arc::new(XmlNode {
                                name: (format!("{}_code", e.name.0), EMPTY_NS_ID),
                                attrs: vec![],
                                children: vec![XmlChild::Text(trimmed.to_string())],
                                source_url: e.source_url.clone(),
                            })));
                            continue;
                        }
                    }
                    // lookup failed or empty text — fall through to normal recursion
                }

                let new_node = resolve_node(Arc::clone(e), resolver);
                match out {
                    None => {
                        if !Arc::ptr_eq(&new_node, e) {
                            let mut nc = children[..i].to_vec();
                            nc.push(XmlChild::Element(new_node));
                            out = Some(nc);
                        }
                    }
                    Some(ref mut nc) => {
                        nc.push(XmlChild::Element(new_node));
                    }
                }
            }
            XmlChild::Text(t) => {
                if let Some(ref mut nc) = out {
                    nc.push(XmlChild::Text(t.clone()));
                }
            }
        }
    }

    out
}

fn load_dictionary(url: &Url) -> Option<HashMap<String, String>> {
    let path = match url.to_file_path() {
        Ok(p) => p,
        Err(_) => {
            tracing::error!(
                url = url.as_str(),
                "citygml3: codeSpace URL is not a file path"
            );
            return None;
        }
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(path = %path.display(), "citygml3: failed to read codelist file: {e}");
            return None;
        }
    };
    let src = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(path = %path.display(), "citygml3: codelist file is not valid UTF-8: {e}");
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
                            "citygml3: failed to unescape codelist <name> text: {e}"
                        ),
                    }
                } else if in_desc {
                    match t.unescape() {
                        Ok(s) => desc_buf = s.trim().to_string(),
                        Err(e) => tracing::error!(
                            url = url.as_str(),
                            "citygml3: failed to unescape codelist <description> text: {e}"
                        ),
                    }
                }
            }
            Ok(Event::End(e)) => match local_name(&String::from_utf8_lossy(e.name().as_ref())) {
                "name" => in_name = false,
                "description" => in_desc = false,
                "Definition" => {
                    if !name_buf.is_empty() && !desc_buf.is_empty() {
                        map.insert(name_buf.clone(), desc_buf.clone());
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::error!(
                    url = url.as_str(),
                    "citygml3: error parsing codelist XML: {e}"
                );
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

    use tempfile::NamedTempFile;

    use super::*;

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

    fn make_code_element(
        elem_name: &str,
        code_space: &str,
        code: &str,
        source_url: Arc<Url>,
    ) -> Arc<XmlNode> {
        Arc::new(XmlNode {
            name: (elem_name.to_string(), EMPTY_NS_ID),
            attrs: vec![(
                ("codeSpace".to_string(), EMPTY_NS_ID),
                code_space.to_string(),
            )],
            children: vec![XmlChild::Text(code.to_string())],
            source_url,
        })
    }

    // On a successful lookup, the element is replaced by two siblings:
    //   1. same-named element without codeSpace, text = resolved label
    //   2. "{name}_code" element, text = original code
    #[test]
    fn resolve_replaces_element_with_label_and_code_sibling() {
        let dict_file = write_dict(&[("201", "Residential")]);
        let dict_filename = dict_file
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        // source_url sits in the same directory so that joining the filename finds the dict.
        let parent = dict_file.path().parent().unwrap();
        let source_url = Arc::new(Url::from_file_path(parent.join("source.gml")).unwrap());

        let code_elem = make_code_element("bldg:usage", &dict_filename, "201", source_url.clone());
        let root = Arc::new(XmlNode {
            name: ("bldg:Building".to_string(), EMPTY_NS_ID),
            attrs: vec![],
            children: vec![XmlChild::Element(code_elem)],
            source_url: source_url.clone(),
        });

        let mut resolver = CodelistResolver::new();
        let result = resolve(vec![root], &mut resolver);

        let children = &result[0].children;
        assert_eq!(children.len(), 2, "expected label node + _code node");

        let XmlChild::Element(label_node) = &children[0] else {
            panic!("first child must be an element");
        };
        assert_eq!(label_node.name.0, "bldg:usage");
        assert!(
            label_node.attrs.iter().all(|((k, _), _)| k != "codeSpace"),
            "codeSpace attr must be stripped from label node"
        );
        assert!(
            matches!(label_node.children.as_slice(), [XmlChild::Text(t)] if t == "Residential")
        );

        let XmlChild::Element(code_node) = &children[1] else {
            panic!("second child must be an element");
        };
        assert_eq!(code_node.name.0, "bldg:usage_code");
        assert!(matches!(code_node.children.as_slice(), [XmlChild::Text(t)] if t == "201"));
    }

    // When the code is absent from the dictionary the original element must survive intact.
    #[test]
    fn resolve_preserves_node_on_lookup_miss() {
        let dict_file = write_dict(&[("201", "Residential")]);
        let dict_filename = dict_file
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let parent = dict_file.path().parent().unwrap();
        let source_url = Arc::new(Url::from_file_path(parent.join("source.gml")).unwrap());

        let code_elem = make_code_element("bldg:usage", &dict_filename, "999", source_url.clone());
        let root = Arc::new(XmlNode {
            name: ("bldg:Building".to_string(), EMPTY_NS_ID),
            attrs: vec![],
            children: vec![XmlChild::Element(Arc::clone(&code_elem))],
            source_url: source_url.clone(),
        });

        let mut resolver = CodelistResolver::new();
        let result = resolve(vec![root], &mut resolver);

        let children = &result[0].children;
        assert_eq!(children.len(), 1);
        let XmlChild::Element(elem) = &children[0] else {
            panic!("child must be an element");
        };
        assert_eq!(elem.name.0, "bldg:usage");
        assert!(elem.attrs.iter().any(|((k, _), _)| k == "codeSpace"));
    }
}
