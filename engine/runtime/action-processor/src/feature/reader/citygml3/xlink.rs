//! XLink href resolution for CityGML 3 nodes.

use std::collections::HashSet;
use std::sync::Arc;

use url::Url;

use super::utils::{gml_id_attr, xlink_href_attr, IdRegistry, XmlChild, XmlNode, XLINK_NS};

/// Resolves `xlink:href` references in an [`XmlNode`] tree.
///
/// Both intra-file (`#id`) and cross-file (`other.gml#id`) hrefs are resolved.
/// Cross-file hrefs are resolved relative to `base_url`.
/// Unresolvable references are left in place. Circular references are detected
/// and left unresolved to prevent infinite recursion.
struct XlinkResolver<'a> {
    base_url: &'a Url,
    registry: &'a IdRegistry,
    visited: HashSet<(String, String)>,
}

impl<'a> XlinkResolver<'a> {
    fn new(base_url: &'a Url, registry: &'a IdRegistry) -> Self {
        Self {
            base_url,
            registry,
            visited: HashSet::new(),
        }
    }

    fn resolve(&mut self, node: &XmlNode) -> XmlNode {
        // Pre-mark this node's own gml:id so any back-reference to it is blocked
        // immediately, catching cycles one hop earlier.
        if let Some(id) = gml_id_attr(node) {
            self.visited
                .insert((self.base_url.as_str().to_string(), id));
        }

        if node.children.is_empty() {
            if let Some(href) = xlink_href_attr(&node.attrs) {
                let resolved_key = if let Some(frag) = href.strip_prefix('#') {
                    Some((self.base_url.as_str().to_string(), frag.to_string()))
                } else if let Some((file_part, frag)) = href.split_once('#') {
                    self.base_url
                        .join(file_part)
                        .ok()
                        .map(|u| (u.as_str().to_string(), frag.to_string()))
                } else {
                    None
                };

                if let Some(key) = resolved_key {
                    if self.visited.contains(&key) {
                        tracing::warn!(
                            "citygml3: circular xlink:href '{}' in element '{}', skipped",
                            href,
                            node.name
                        );
                    } else if let Some(target) = self.registry.get(&key) {
                        self.visited.insert(key);
                        let attrs = node
                            .attrs
                            .iter()
                            .filter(|(q, ns, _)| {
                                !(super::utils::local_name(q) == "href" && ns == XLINK_NS)
                            })
                            .cloned()
                            .collect();
                        let resolved_target = self.resolve(target);
                        return XmlNode {
                            name: node.name.clone(),
                            attrs,
                            children: vec![XmlChild::Element(Arc::new(resolved_target))],
                        };
                    } else {
                        tracing::warn!(
                            "citygml3: unresolved xlink:href '{}' in element '{}'",
                            href,
                            node.name
                        );
                    }
                }
            }
        }

        let children = node
            .children
            .iter()
            .map(|c| match c {
                XmlChild::Element(e) => XmlChild::Element(Arc::new(self.resolve(e))),
                XmlChild::Text(t) => XmlChild::Text(t.clone()),
            })
            .collect();

        XmlNode {
            name: node.name.clone(),
            attrs: node.attrs.clone(),
            children,
        }
    }
}

/// Recursively walk `node`, replacing elements that carry only an
/// `xlink:href` attribute and no children with the referenced node from
/// `registry`.
///
/// Both intra-file (`#id`) and cross-file (`other.gml#id`) hrefs are
/// resolved. Cross-file hrefs are resolved relative to `base_url`.
/// Unresolvable and circular references are left in place.
pub fn resolve_xlinks(node: &XmlNode, base_url: &Url, registry: &IdRegistry) -> XmlNode {
    XlinkResolver::new(base_url, registry).resolve(node)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use url::Url;

    use super::resolve_xlinks;
    use crate::feature::reader::citygml3::utils::{
        local_name, IdRegistry, XmlChild, XmlNode, GML_NS, XLINK_NS,
    };

    fn dummy_url() -> Url {
        Url::parse("file:///test.gml").unwrap()
    }

    fn make_node(name: &str, attrs: Vec<(&str, &str, &str)>, children: Vec<XmlChild>) -> XmlNode {
        XmlNode {
            name: name.to_string(),
            attrs: attrs
                .into_iter()
                .map(|(q, ns, v)| (q.to_string(), ns.to_string(), v.to_string()))
                .collect(),
            children,
        }
    }

    fn elem(node: XmlNode) -> XmlChild {
        XmlChild::Element(Arc::new(node))
    }

    #[test]
    fn resolve_xlinks_replaces_href_leaf() {
        let target = Arc::new(make_node(
            "gml:Polygon",
            vec![("gml:id", GML_NS, "poly001")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert((dummy_url().to_string(), "poly001".to_string()), target);

        let node = make_node(
            "bldg:lod1Solid",
            vec![("xlink:href", XLINK_NS, "#poly001")],
            vec![],
        );

        let resolved = resolve_xlinks(&node, &dummy_url(), &reg);

        assert_eq!(resolved.children.len(), 1);
        if let XmlChild::Element(e) = &resolved.children[0] {
            assert_eq!(e.name, "gml:Polygon");
        } else {
            panic!("expected Element child");
        }
        assert!(!resolved
            .attrs
            .iter()
            .any(|(q, ns, _)| local_name(q) == "href" && ns == XLINK_NS));
    }

    #[test]
    fn resolve_xlinks_non_standard_xlink_prefix() {
        // Uses 'xl:href' instead of 'xlink:href' — must still resolve.
        let target = Arc::new(make_node(
            "gml:Polygon",
            vec![("gml:id", GML_NS, "p1")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert((dummy_url().to_string(), "p1".to_string()), target);

        let node = make_node("ref", vec![("xl:href", XLINK_NS, "#p1")], vec![]);
        let resolved = resolve_xlinks(&node, &dummy_url(), &reg);

        assert_eq!(resolved.children.len(), 1);
    }

    #[test]
    fn resolve_xlinks_handles_cross_file_fragment() {
        // base_url is file:///test.gml; "other.gml" resolves to file:///other.gml
        let other_url = Url::parse("file:///other.gml").unwrap();
        let target = Arc::new(make_node(
            "gml:Polygon",
            vec![("gml:id", GML_NS, "p1")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert((other_url.to_string(), "p1".to_string()), target);

        let node = make_node(
            "ref",
            vec![("xlink:href", XLINK_NS, "other.gml#p1")],
            vec![],
        );
        let resolved = resolve_xlinks(&node, &dummy_url(), &reg);

        assert_eq!(resolved.children.len(), 1);
    }

    #[test]
    fn resolve_xlinks_leaves_unresolvable_in_place() {
        let reg = IdRegistry::new();
        let node = make_node("ref", vec![("xlink:href", XLINK_NS, "#missing")], vec![]);
        let resolved = resolve_xlinks(&node, &dummy_url(), &reg);

        assert!(resolved.children.is_empty());
        assert!(resolved
            .attrs
            .iter()
            .any(|(q, ns, _)| local_name(q) == "href" && ns == XLINK_NS));
    }

    #[test]
    fn resolve_xlinks_recurses_into_children() {
        let target = Arc::new(make_node(
            "gml:Point",
            vec![("gml:id", GML_NS, "pt1")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert((dummy_url().to_string(), "pt1".to_string()), target);

        let inner = make_node("bldg:pos", vec![("xlink:href", XLINK_NS, "#pt1")], vec![]);
        let node = make_node("bldg:Building", vec![], vec![elem(inner)]);

        let resolved = resolve_xlinks(&node, &dummy_url(), &reg);

        let child = match &resolved.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!("expected element"),
        };
        assert_eq!(child.name, "bldg:pos");
        assert_eq!(child.children.len(), 1);
    }

    #[test]
    fn resolve_xlinks_detects_cycle() {
        // A(gml:id=a, href=#b) → B(gml:id=b, href=#a): back-reference to A must be blocked.
        // Because A's id is pre-marked on entry, B's href=#a is caught immediately.
        let mut reg = IdRegistry::new();

        let node_b = Arc::new(make_node(
            "B",
            vec![("gml:id", GML_NS, "b"), ("xlink:href", XLINK_NS, "#a")],
            vec![],
        ));
        reg.insert(
            (dummy_url().to_string(), "b".to_string()),
            Arc::clone(&node_b),
        );

        let node_a = Arc::new(make_node(
            "A",
            vec![("gml:id", GML_NS, "a"), ("xlink:href", XLINK_NS, "#b")],
            vec![],
        ));
        reg.insert(
            (dummy_url().to_string(), "a".to_string()),
            Arc::clone(&node_a),
        );

        // Must terminate without stack overflow.
        let resolved = resolve_xlinks(&node_a, &dummy_url(), &reg);

        // A resolved to wrapper containing B.
        let b = match resolved.children.first() {
            Some(XmlChild::Element(e)) => e,
            _ => panic!("expected B as first child"),
        };
        assert_eq!(b.name, "B");
        // B's back-reference to A is blocked — no children injected.
        assert!(b.children.is_empty());
    }
}
