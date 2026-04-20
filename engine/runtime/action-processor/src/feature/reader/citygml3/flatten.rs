use std::collections::HashSet;
use std::sync::Arc;

use super::utils::{local_name, XmlChild, XmlNode};

/// Walks `node`'s subtree bottom-up, lifting out every descendant whose tag is in `included`
/// (matched by qualified name, local name, or Clark notation `{ns}local`).
/// Returns the stripped tree and the extracted nodes, deepest-first.
/// No-op when `included` is empty.
pub fn extract_by_types(
    node: &Arc<XmlNode>,
    included: &HashSet<String>,
) -> (Arc<XmlNode>, Vec<Arc<XmlNode>>) {
    if included.is_empty() {
        return (Arc::clone(node), Vec::new());
    }
    let mut extracted: Vec<Arc<XmlNode>> = Vec::new();
    let stripped = extract_recursive(node, included, &mut extracted);
    (stripped, extracted)
}

pub fn tag_matches(node: &XmlNode, included: &HashSet<String>) -> bool {
    let ln = local_name(&node.name.0);
    included.contains(node.name.0.as_str())
        || included.contains(ln)
        || (!node.name.1.is_empty()
            && included.contains(&format!("{{{}}}{ln}", node.name.1)))
}

fn extract_recursive(
    node: &Arc<XmlNode>,
    included: &HashSet<String>,
    out: &mut Vec<Arc<XmlNode>>,
) -> Arc<XmlNode> {
    let mut new_children: Option<Vec<XmlChild>> = None;

    for (i, child) in node.children.iter().enumerate() {
        match child {
            XmlChild::Element(e) => {
                if tag_matches(e, included) {
                    // Process the child's own subtree first (bottom-up), then lift the child out.
                    let stripped_child = extract_recursive(e, included, out);
                    out.push(stripped_child);

                    if new_children.is_none() {
                        new_children = Some(node.children[..i].to_vec());
                    }
                    // deliberately not pushed into new_children — it is extracted
                } else {
                    let stripped_child = extract_recursive(e, included, out);
                    match new_children {
                        None => {
                            if !Arc::ptr_eq(&stripped_child, e) {
                                let mut nc = node.children[..i].to_vec();
                                nc.push(XmlChild::Element(stripped_child));
                                new_children = Some(nc);
                            }
                        }
                        Some(ref mut nc) => {
                            nc.push(XmlChild::Element(stripped_child));
                        }
                    }
                }
            }
            XmlChild::Text(_) => {
                if let Some(ref mut nc) = new_children {
                    nc.push(child.clone());
                }
            }
        }
    }

    match new_children {
        None => Arc::clone(node),
        Some(children) => Arc::new(XmlNode {
            name: node.name.clone(),
            attrs: node.attrs.clone(),
            children,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::reader::citygml3::utils::XmlChild;

    fn node(name: &str, children: Vec<XmlChild>) -> Arc<XmlNode> {
        Arc::new(XmlNode {
            name: (name.to_string(), String::new()),
            attrs: Vec::new(),
            children,
        })
    }

    fn elem(n: Arc<XmlNode>) -> XmlChild {
        XmlChild::Element(n)
    }

    fn included(tags: &[&str]) -> HashSet<String> {
        tags.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn matching_child_extracted_from_parent() {
        let part = node("bldg:BuildingPart", vec![]);
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);
        let (stripped, extracted) = extract_by_types(&root, &included(&["bldg:BuildingPart"]));
        assert!(stripped.children.is_empty());
        assert_eq!(extracted.len(), 1);
        assert!(Arc::ptr_eq(&extracted[0], &part));
    }

    #[test]
    fn deep_match_extracted_before_shallow() {
        // Building > BuildingPart > Room — inclusion: BuildingPart, Room
        // Expect: Room first (deepest), BuildingPart second, BuildingPart no longer contains Room
        let room = node("bldg:Room", vec![]);
        let part = node("bldg:BuildingPart", vec![elem(Arc::clone(&room))]);
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);

        let (stripped, extracted) =
            extract_by_types(&root, &included(&["bldg:BuildingPart", "bldg:Room"]));

        assert!(stripped.children.is_empty());
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].name.0, "bldg:Room");
        assert_eq!(extracted[1].name.0, "bldg:BuildingPart");
        assert!(extracted[1].children.is_empty());
    }

    #[test]
    fn local_name_match() {
        let part = node("bldg:BuildingPart", vec![]);
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);
        let (stripped, extracted) = extract_by_types(&root, &included(&["BuildingPart"]));
        assert!(stripped.children.is_empty());
        assert_eq!(extracted.len(), 1);
    }

    #[test]
    fn clark_notation_match() {
        let ns = "http://www.opengis.net/citygml/building/3.0";
        let part = Arc::new(XmlNode {
            name: ("bldg:BuildingPart".to_string(), ns.to_string()),
            attrs: Vec::new(),
            children: Vec::new(),
        });
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);
        let clark = format!("{{{ns}}}BuildingPart");
        let (stripped, extracted) = extract_by_types(&root, &included(&[&clark]));
        assert!(stripped.children.is_empty());
        assert_eq!(extracted.len(), 1);
    }
}
