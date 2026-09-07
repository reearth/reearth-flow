use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::utils::{
    gml_id_attr, local_name, GeomNode, NamespaceRegistry, NsId, XmlChild, XmlNode, EMPTY_NS_ID,
};

/// One feature discovered by [`extract`], ready to become an output `Feature`.
pub(super) struct Extracted {
    pub(super) node: Arc<XmlNode>,
    pub(super) parent_gml_id: Option<String>,
    /// Geometry found on this node or on its own non-extracted descendants, stopping at any
    /// deeper extracted feature (which collects its own). Unused under legacy, which attaches
    /// geometry via its own separate mechanism instead.
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    pub(super) geometry: Vec<FoundGeometry>,
}

/// A geometry found while walking, tagged with the LOD of the property it came from (`None` for
/// `tin`) and the node it was a direct child of, used to name the object a surface belongs to.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
pub(super) struct FoundGeometry {
    pub(super) lod: Option<u8>,
    pub(super) node: Arc<GeomNode>,
    pub(super) owner_gml_id: Option<String>,
    pub(super) owner_feature_type: String,
}

/// Collect every geometry in `node`'s subtree, with no stopping condition — used when
/// `extract_tags` is empty and the whole tree becomes a single feature.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
pub(super) fn collect_all_geometry(node: &Arc<XmlNode>) -> Vec<FoundGeometry> {
    let mut out = Vec::new();
    collect_all_geometry_inner(node, &mut out);
    out
}

#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
fn collect_all_geometry_inner(node: &Arc<XmlNode>, out: &mut Vec<FoundGeometry>) {
    for child in &node.children {
        match child {
            XmlChild::Element(e) => collect_all_geometry_inner(e, out),
            XmlChild::Geometry(lod, g) => out.push(FoundGeometry {
                lod: *lod,
                node: Arc::clone(g),
                owner_gml_id: gml_id_attr(&node.attrs),
                owner_feature_type: node.name.0.clone(),
            }),
            XmlChild::Text(_) => {}
        }
    }
}

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
/// Returns `(node, nearest_extracted_ancestor_gml_id)` pairs: the parent id is the nearest
/// ancestor that is *itself* extracted (i.e. also tag-matched), not merely the nearest ancestor
/// that happens to carry a `gml:id` — an unmatched wrapper element (e.g. a `Section` or
/// `TrafficSpace` not in `included`) never becomes a parent reference, even though it has its own
/// `gml:id`. The parent ID is correct even when the same node is reached via multiple paths (e.g.
/// shared xlink targets).
/// Note: if a parent and a descendant tag both appear in `included`, the descendant's geometry
/// and attributes are stripped from the parent and emitted separately.
pub(super) fn extract(
    node: &Arc<XmlNode>,
    included: &HashSet<String>,
    ns_registry: &NamespaceRegistry,
) -> Vec<Extracted> {
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
    out: &mut Vec<Extracted>,
    parent_gml_id: Option<&str>,
) {
    if tag_matches(node, sets) {
        let mut geometry = Vec::new();
        let stripped = extract_recursive(node, sets, out, parent_gml_id, true, &mut geometry);
        out.push(Extracted {
            node: stripped,
            parent_gml_id: parent_gml_id.map(str::to_string),
            geometry,
        });
    } else {
        // Not extracted: never adopt this node's own gml:id as a parent baseline, even if it
        // has one — only extracted ancestors may become parent references.
        for child in &node.children {
            if let XmlChild::Element(e) = child {
                extract_inner(e, sets, out, parent_gml_id);
            }
        }
    }
}

fn extract_recursive(
    node: &Arc<XmlNode>,
    sets: &MatchSets,
    out: &mut Vec<Extracted>,
    parent_gml_id: Option<&str>,
    matched: bool,
    geometry: &mut Vec<FoundGeometry>,
) -> Arc<XmlNode> {
    // Only a node that is itself extracted may hand its own gml:id down as the parent
    // baseline for its descendants; an unmatched wrapper passes `parent_gml_id` through as-is.
    let my_id = if matched {
        gml_id_attr(&node.attrs)
    } else {
        None
    };
    let child_parent = my_id.as_deref().or(parent_gml_id);

    let mut new_children: Option<Vec<XmlChild>> = None;

    for (i, child) in node.children.iter().enumerate() {
        match child {
            XmlChild::Element(e) => {
                if tag_matches(e, sets) {
                    let mut child_geometry = Vec::new();
                    let stripped_child =
                        extract_recursive(e, sets, out, child_parent, true, &mut child_geometry);
                    out.push(Extracted {
                        node: stripped_child,
                        parent_gml_id: child_parent.map(str::to_string),
                        geometry: child_geometry,
                    });

                    if new_children.is_none() {
                        new_children = Some(node.children[..i].to_vec());
                    }
                    // deliberately not pushed into new_children — it is extracted
                } else {
                    let stripped_child =
                        extract_recursive(e, sets, out, child_parent, false, geometry);
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
            XmlChild::Geometry(lod, g) => {
                geometry.push(FoundGeometry {
                    lod: *lod,
                    node: Arc::clone(g),
                    owner_gml_id: gml_id_attr(&node.attrs),
                    owner_feature_type: node.name.0.clone(),
                });
                if new_children.is_none() {
                    new_children = Some(node.children[..i].to_vec());
                }
            }
        }
    }

    match new_children {
        None => Arc::clone(node),
        Some(children) => node.with_children(children),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citygml_parser::utils::{
        test_url, NamespaceRegistry, XmlChild, EMPTY_NS_ID, GML_NS_ID,
    };

    fn node(name: &str, children: Vec<XmlChild>) -> Arc<XmlNode> {
        Arc::new(XmlNode {
            name: (name.to_string(), EMPTY_NS_ID),
            attrs: Vec::new(),
            children,
            source_url: test_url(),
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
            source_url: test_url(),
        })
    }

    #[test]
    fn matching_child_extracted_from_parent() {
        let ns_reg = NamespaceRegistry::new();
        let part = node("bldg:BuildingPart", vec![]);
        let root = node("bldg:Building", vec![elem(Arc::clone(&part))]);
        let extracted = extract(&root, &included(&["bldg:BuildingPart"]), &ns_reg);
        assert_eq!(extracted.len(), 1);
        assert!(Arc::ptr_eq(&extracted[0].node, &part));
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
        assert_eq!(extracted[0].node.name.0, "bldg:Room");
        assert_eq!(extracted[1].node.name.0, "bldg:BuildingPart");
        assert!(extracted[1].node.children.is_empty());
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
            source_url: test_url(),
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
        // A and B must themselves be extracted to legitimately serve as parents.
        let ns_reg = NamespaceRegistry::new();
        let c = gml_id("bldg:Unit", "c", vec![]);
        let a = gml_id("bldg:Building", "a", vec![elem(Arc::clone(&c))]);
        let b = gml_id("bldg:Building", "b", vec![elem(Arc::clone(&c))]);
        let root = node("root", vec![elem(Arc::clone(&a)), elem(Arc::clone(&b))]);

        let extracted = extract(&root, &included(&["bldg:Unit", "bldg:Building"]), &ns_reg);

        assert_eq!(extracted.len(), 4);
        let unit_parents: Vec<_> = extracted
            .iter()
            .filter(|e| e.node.name.0 == "bldg:Unit")
            .map(|e| e.parent_gml_id.as_deref())
            .collect();
        assert!(
            unit_parents.contains(&Some("a")),
            "first emission should have parent a"
        );
        assert!(
            unit_parents.contains(&Some("b")),
            "second emission should have parent b"
        );
    }

    #[test]
    fn unmatched_wrapper_with_gml_id_is_not_a_parent() {
        // `bldg:Section` carries a gml:id but is not itself in `included`; the extracted
        // descendant beneath it must skip past it and report the nearest *extracted*
        // ancestor instead of the unmatched wrapper's id.
        let ns_reg = NamespaceRegistry::new();
        let area = node("bldg:TrafficArea", vec![]);
        let wrapper = gml_id("bldg:Section", "wrapper", vec![elem(Arc::clone(&area))]);
        let root = gml_id("bldg:Track", "root", vec![elem(Arc::clone(&wrapper))]);

        let extracted = extract(
            &root,
            &included(&["bldg:Track", "bldg:TrafficArea"]),
            &ns_reg,
        );

        assert_eq!(extracted.len(), 2);
        let area_parent = extracted
            .iter()
            .find(|e| e.node.name.0 == "bldg:TrafficArea")
            .and_then(|e| e.parent_gml_id.as_deref());
        assert_eq!(area_parent, Some("root"));
    }

    #[cfg(feature = "new-geometry")]
    fn geom_ref(id: &str) -> Arc<GeomNode> {
        Arc::new(GeomNode::Ref(("file:///t.gml".to_string(), id.to_string())))
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn geometry_collected_from_own_node() {
        let ns_reg = NamespaceRegistry::new();
        let root = gml_id(
            "bldg:Building",
            "b1",
            vec![XmlChild::Geometry(Some(2), geom_ref("g1"))],
        );
        let extracted = extract(&root, &included(&["bldg:Building"]), &ns_reg);
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].geometry.len(), 1);
        assert_eq!(extracted[0].geometry[0].lod, Some(2));
        assert_eq!(extracted[0].geometry[0].owner_gml_id.as_deref(), Some("b1"));
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn geometry_rolls_up_past_unmatched_descendant() {
        let ns_reg = NamespaceRegistry::new();
        let wall = gml_id(
            "con:WallSurface",
            "wall1",
            vec![XmlChild::Geometry(Some(2), geom_ref("g1"))],
        );
        let root = gml_id("bldg:Building", "b1", vec![elem(wall)]);
        let extracted = extract(&root, &included(&["bldg:Building"]), &ns_reg);
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].geometry.len(), 1);
        assert_eq!(
            extracted[0].geometry[0].owner_gml_id.as_deref(),
            Some("wall1")
        );
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn geometry_stops_at_deeper_extracted_descendant() {
        let ns_reg = NamespaceRegistry::new();
        let part = gml_id(
            "bldg:BuildingPart",
            "part1",
            vec![XmlChild::Geometry(Some(2), geom_ref("g1"))],
        );
        let root = gml_id("bldg:Building", "b1", vec![elem(part)]);
        let extracted = extract(
            &root,
            &included(&["bldg:Building", "bldg:BuildingPart"]),
            &ns_reg,
        );
        assert_eq!(extracted.len(), 2);
        let building = extracted
            .iter()
            .find(|e| e.node.name.0 == "bldg:Building")
            .unwrap();
        let part_extracted = extracted
            .iter()
            .find(|e| e.node.name.0 == "bldg:BuildingPart")
            .unwrap();
        assert!(
            building.geometry.is_empty(),
            "geometry belongs to the extracted BuildingPart, not the Building"
        );
        assert_eq!(part_extracted.geometry.len(), 1);
    }
}
