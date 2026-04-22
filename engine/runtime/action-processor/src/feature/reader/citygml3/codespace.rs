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
            let dict = load_dictionary(&dict_url).unwrap_or_default();
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
                    name_buf = t.unescape().unwrap_or_default().trim().to_string();
                } else if in_desc {
                    desc_buf = t.unescape().unwrap_or_default().trim().to_string();
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
