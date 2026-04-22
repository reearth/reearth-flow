use std::collections::HashMap;
use std::sync::Arc;

use quick_xml::events::Event;
use quick_xml::Reader;
use url::Url;

use super::parser::{RawChild, RawNode, RawRegistry};
use super::utils::{local_name, EMPTY_NS_ID};

pub fn resolve(pending: Vec<Arc<RawNode>>, registry: &RawRegistry) -> Vec<Arc<RawNode>> {
    let url_map: HashMap<*const RawNode, Url> = registry
        .iter()
        .filter_map(|((file_url, _), node)| {
            Url::parse(file_url).ok().map(|u| (Arc::as_ptr(node), u))
        })
        .collect();

    let mut cache: HashMap<String, HashMap<String, String>> = HashMap::new();

    pending
        .into_iter()
        .map(|node| match url_map.get(&Arc::as_ptr(&node)) {
            Some(url) => process_node(node, url, &mut cache),
            None => node,
        })
        .collect()
}

fn process_node(
    node: Arc<RawNode>,
    source_url: &Url,
    cache: &mut HashMap<String, HashMap<String, String>>,
) -> Arc<RawNode> {
    Arc::new(RawNode {
        name: node.name.clone(),
        attrs: node.attrs.clone(),
        children: process_children(&node.children, source_url, cache),
    })
}

fn process_children(
    children: &[RawChild],
    source_url: &Url,
    cache: &mut HashMap<String, HashMap<String, String>>,
) -> Vec<RawChild> {
    let mut out = Vec::with_capacity(children.len());
    for child in children {
        match child {
            RawChild::Element(e) => {
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
                            RawChild::Text(t) => Some(t.as_str()),
                            _ => None,
                        })
                        .collect();
                    let trimmed = text.trim();

                    if !trimmed.is_empty() {
                        if let Some(label) = lookup(source_url, &cs, trimmed, cache) {
                            out.push(RawChild::Element(Arc::new(RawNode {
                                name: e.name.clone(),
                                attrs: e
                                    .attrs
                                    .iter()
                                    .filter(|((q, _), _)| local_name(q) != "codeSpace")
                                    .cloned()
                                    .collect(),
                                children: vec![RawChild::Text(label)],
                            })));
                            out.push(RawChild::Element(Arc::new(RawNode {
                                name: (format!("{}_code", e.name.0), EMPTY_NS_ID),
                                attrs: vec![],
                                children: vec![RawChild::Text(trimmed.to_string())],
                            })));
                            continue;
                        }
                    }
                }
                out.push(RawChild::Element(process_node(Arc::clone(e), source_url, cache)));
            }
            RawChild::Text(t) => out.push(RawChild::Text(t.clone())),
            RawChild::Ref(k) => out.push(RawChild::Ref(k.clone())),
        }
    }
    out
}

fn lookup(
    source_url: &Url,
    code_space: &str,
    code: &str,
    cache: &mut HashMap<String, HashMap<String, String>>,
) -> Option<String> {
    let dict_url = match source_url.join(code_space) {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(source_url = source_url.as_str(), code_space, "citygml3: failed to join codeSpace URL: {e}");
            return None;
        }
    };
    let key = dict_url.to_string();
    if !cache.contains_key(&key) {
        let dict = load_dictionary(&dict_url).unwrap_or_default();
        cache.insert(key.clone(), dict);
    }
    let dict = cache.get(&key)?;
    if !dict.contains_key(code) {
        tracing::error!(dict_url = key, code, "citygml3: code not found in codelist");
    }
    dict.get(code).cloned()
}

fn load_dictionary(url: &Url) -> Option<HashMap<String, String>> {
    let path = match url.to_file_path() {
        Ok(p) => p,
        Err(_) => {
            tracing::error!(url = url.as_str(), "citygml3: codeSpace URL is not a file path");
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
            Ok(Event::Start(e)) => {
                match local_name(&String::from_utf8_lossy(e.name().as_ref())) {
                    "name" => in_name = true,
                    "description" => in_desc = true,
                    "Definition" => {
                        name_buf.clear();
                        desc_buf.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if in_name {
                    name_buf = t.unescape().unwrap_or_default().trim().to_string();
                } else if in_desc {
                    desc_buf = t.unescape().unwrap_or_default().trim().to_string();
                }
            }
            Ok(Event::End(e)) => {
                match local_name(&String::from_utf8_lossy(e.name().as_ref())) {
                    "name" => in_name = false,
                    "description" => in_desc = false,
                    "Definition" => {
                        if !name_buf.is_empty() && !desc_buf.is_empty() {
                            map.insert(name_buf.clone(), desc_buf.clone());
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::error!(url = url.as_str(), "citygml3: error parsing codelist XML: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Some(map)
}
