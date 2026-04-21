use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::utils::{
    gml_id_attr, local_name, NamespaceRegistry, NsId, XmlChild, XmlNode, EMPTY_NS_ID,
};

/// Pre-processed form of the `included` tag set that avoids per-node allocation.
/// Clark-notation entries (`{ns}local`) are resolved to `(NsId, local)` pairs once at the
/// `extract()` boundary, so matching is a cheap integer + string-ref comparison.
struct MatchSets {
    raw: HashSet<String>,
    clark: HashMap<NsId, HashSet<String>>,
}

impl MatchSets {
    fn new(included: &HashSet<String>, ns_reg: &NamespaceRegistry) -> Self {
        let mut raw: HashSet<String> = HashSet::new();
        let mut clark: HashMap<NsId, HashSet<String>> = HashMap::new();
        for s in included {
            if let Some(rest) = s.strip_prefix('{') {
                if let Some(end) = rest.find('}') {
                    let uri = &rest[..end];
                    let local = rest[end + 1..].to_string();
                    if let Some(id) = ns_reg.get(uri) {
                        clark.entry(id).or_default().insert(local);
                    }
                }
            } else {
                raw.insert(s.clone());
            }
        }
        Self { raw, clark }
    }
}

fn tag_matches(node: &XmlNode, sets: &MatchSets) -> bool {
    let ln = local_name(&node.name.0);
    sets.raw.contains(node.name.0.as_str())
        || sets.raw.contains(ln)
        || (node.name.1 != EMPTY_NS_ID
            && sets
                .clark
                .get(&node.name.1)
                .is_some_and(|locals| locals.contains(ln)))
}

/// Extracts all nodes whose tag is in `included` from `node`'s subtree (including `node` itself),
/// deepest-first. Each extracted node has its own matching descendants stripped out.
/// Returns `(node, nearest_ancestor_gml_id)` pairs; the parent ID is correct even when the same
/// node is reached via multiple paths (e.g. shared xlink targets).
/// Note: if a parent and a descendant tag both appear in `included`, the descendant's geometry
/// and attributes are stripped from the parent and emitted separately.
pub(super) fn extract(
    node: &Arc<XmlNode>,
    included: &HashSet<String>,
    ns_registry: &NamespaceRegistry,
) -> Vec<(Arc<XmlNode>, Option<String>)> {
    if included.is_empty() {
        return Vec::new();
    }
    let sets = MatchSets::new(included, ns_registry);
    let mut out = Vec::new();
    extract_inner(node, &sets, &mut out, None);
    out
}

fn extract_inner(
    node: &Arc<XmlNode>,
    sets: &MatchSets,
    out: &mut Vec<(Arc<XmlNode>, Option<String>)>,
    parent_gml_id: Option<&str>,
) {
    if tag_matches(node, sets) {
        let stripped = extract_recursive(node, sets, out, parent_gml_id);
        out.push((stripped, parent_gml_id.map(str::to_string)));
    } else {
        let my_id = gml_id_attr(node);
        let next_parent = my_id.as_deref().or(parent_gml_id);
        for child in &node.children {
            if let XmlChild::Element(e) = child {
                extract_inner(e, sets, out, next_parent);
            }
        }
    }
}

fn extract_recursive(
    node: &Arc<XmlNode>,
    sets: &MatchSets,
    out: &mut Vec<(Arc<XmlNode>, Option<String>)>,
    parent_gml_id: Option<&str>,
) -> Arc<XmlNode> {
    let my_id = gml_id_attr(node);
    let child_parent = my_id.as_deref().or(parent_gml_id);

    let mut new_children: Option<Vec<XmlChild>> = None;

    for (i, child) in node.children.iter().enumerate() {
        match child {
            XmlChild::Element(e) => {
                if tag_matches(e, sets) {
                    let stripped_child = extract_recursive(e, sets, out, child_parent);
                    out.push((stripped_child, child_parent.map(str::to_string)));

                    if new_children.is_none() {
                        new_children = Some(node.children[..i].to_vec());
                    }
                    // deliberately not pushed into new_children — it is extracted
                } else {
                    let stripped_child = extract_recursive(e, sets, out, child_parent);
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
    use crate::feature::reader::citygml3::utils::{
        NamespaceRegistry, XmlChild, EMPTY_NS_ID, GML_NS_ID,
    };

    fn node(name: &str, children: Vec<XmlChild>) -> Arc<XmlNode> {
        Arc::new(XmlNode {
            name: (name.to_string(), EMPTY_NS_ID),
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

    fn gml_id(name: &str, id: &str, children: Vec<XmlChild>) -> Arc<XmlNode> {
        Arc::new(XmlNode {
            name: (name.to_string(), EMPTY_NS_ID),
            attrs: vec![(("gml:id".to_string(), GML_NS_ID), id.to_string())],
            children,
        })
    }

    #[test]
    fn matching_child_extracted_from_parent() {
        let ns_reg = NamespaceRegistry::new();
        let part = node("bldg:BuildingPart", vec![]);
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);
        let extracted = extract(&root, &included(&["bldg:BuildingPart"]), &ns_reg);
        assert_eq!(extracted.len(), 1);
        assert!(Arc::ptr_eq(&extracted[0].0, &part));
    }

    #[test]
    fn deep_match_extracted_before_shallow() {
        let ns_reg = NamespaceRegistry::new();
        let room = node("bldg:Room", vec![]);
        let part = node("bldg:BuildingPart", vec![elem(Arc::clone(&room))]);
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);

        let extracted = extract(
            &root,
            &included(&["bldg:BuildingPart", "bldg:Room"]),
            &ns_reg,
        );

        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].0.name.0, "bldg:Room");
        assert_eq!(extracted[1].0.name.0, "bldg:BuildingPart");
        assert!(extracted[1].0.children.is_empty());
    }

    #[test]
    fn local_name_match() {
        let ns_reg = NamespaceRegistry::new();
        let part = node("bldg:BuildingPart", vec![]);
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);
        let extracted = extract(&root, &included(&["BuildingPart"]), &ns_reg);
        assert_eq!(extracted.len(), 1);
    }

    #[test]
    fn clark_notation_match() {
        let mut ns_reg = NamespaceRegistry::new();
        let ns = "http://www.opengis.net/citygml/building/3.0";
        let ns_id = ns_reg.intern(ns);
        let part = Arc::new(XmlNode {
            name: ("bldg:BuildingPart".to_string(), ns_id),
            attrs: Vec::new(),
            children: Vec::new(),
        });
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);
        let clark = format!("{{{ns}}}BuildingPart");
        let extracted = extract(&root, &included(&[&clark]), &ns_reg);
        assert_eq!(extracted.len(), 1);
    }

    #[test]
    fn shared_node_gets_correct_parent_per_occurrence() {
        // C is referenced under both A and B (same Arc, simulating xlink resolution).
        // Each emission of C must carry the parent from its own traversal position.
        let ns_reg = NamespaceRegistry::new();
        let c = gml_id("bldg:Unit", "c", vec![]);
        let a = gml_id("bldg:Building", "a", vec![elem(Arc::clone(&c))]);
        let b = gml_id("bldg:Building", "b", vec![elem(Arc::clone(&c))]);
        let root = node("root", vec![elem(Arc::clone(&a)), elem(Arc::clone(&b))]);

        let extracted = extract(&root, &included(&["bldg:Unit"]), &ns_reg);

        assert_eq!(extracted.len(), 2);
        let parents: Vec<_> = extracted.iter().map(|(_, p)| p.as_deref()).collect();
        assert!(
            parents.contains(&Some("a")),
            "first emission should have parent a"
        );
        assert!(
            parents.contains(&Some("b")),
            "second emission should have parent b"
        );
    }
}
