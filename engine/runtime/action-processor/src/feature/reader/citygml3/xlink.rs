use std::collections::HashSet;
use std::sync::Arc;

use url::Url;

use super::utils::{gml_id_attr, xlink_href_attr, IdRegistry, XmlChild, XmlNode, XLINK_NS};

struct XlinkResolver<'a> {
    registry: &'a IdRegistry,
    visited: HashSet<(String, String)>,
}

impl<'a> XlinkResolver<'a> {
    fn new(registry: &'a IdRegistry) -> Self {
        Self {
            registry,
            visited: HashSet::new(),
        }
    }

    fn resolve(&mut self, node: &Arc<XmlNode>, current_url: &Url) -> Arc<XmlNode> {
        if !node.has_xlinks {
            return Arc::clone(node);
        }

        // Pre-mark own gml:id so back-references are blocked before recursing.
        if let Some(id) = gml_id_attr(node) {
            self.visited.insert((current_url.as_str().to_string(), id));
        }

        if node.children.is_empty() {
            if let Some(href) = xlink_href_attr(&node.attrs) {
                let resolved_key: Option<(Url, String)> = if let Some(frag) = href.strip_prefix('#')
                {
                    Some((current_url.clone(), frag.to_string()))
                } else if let Some((file_part, frag)) = href.split_once('#') {
                    current_url
                        .join(file_part)
                        .ok()
                        .map(|u| (u, frag.to_string()))
                } else {
                    None
                };

                if let Some((ref target_url, ref frag)) = resolved_key {
                    let key = (target_url.as_str().to_string(), frag.clone());
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
                        let resolved_target = self.resolve(target, target_url);
                        return Arc::new(XmlNode {
                            name: node.name.clone(),
                            attrs,
                            children: vec![XmlChild::Element(resolved_target)],
                            has_xlinks: false,
                        });
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

        let children: Vec<XmlChild> = node
            .children
            .iter()
            .map(|c| match c {
                XmlChild::Element(e) => XmlChild::Element(self.resolve(e, current_url)),
                XmlChild::Text(t) => XmlChild::Text(t.clone()),
            })
            .collect();

        let has_xlinks = xlink_href_attr(&node.attrs).is_some()
            || children.iter().any(|c| match c {
                XmlChild::Element(e) => e.has_xlinks,
                XmlChild::Text(_) => false,
            });

        Arc::new(XmlNode {
            name: node.name.clone(),
            attrs: node.attrs.clone(),
            children,
            has_xlinks,
        })
    }
}

pub fn resolve_xlinks(node: &Arc<XmlNode>, base_url: &Url, registry: &IdRegistry) -> Arc<XmlNode> {
    XlinkResolver::new(registry).resolve(node, base_url)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use url::Url;

    use super::resolve_xlinks;
    use crate::feature::reader::citygml3::utils::{
        local_name, xlink_href_attr, IdRegistry, XmlChild, XmlNode, GML_NS, XLINK_NS,
    };

    fn dummy_url() -> Url {
        Url::parse("file:///test.gml").unwrap()
    }

    fn make_node(name: &str, attrs: Vec<(&str, &str, &str)>, children: Vec<XmlChild>) -> XmlNode {
        let owned_attrs: Vec<(String, String, String)> = attrs
            .into_iter()
            .map(|(q, ns, v)| (q.to_string(), ns.to_string(), v.to_string()))
            .collect();
        let has_xlinks = xlink_href_attr(&owned_attrs).is_some()
            || children.iter().any(|c| match c {
                XmlChild::Element(e) => e.has_xlinks,
                XmlChild::Text(_) => false,
            });
        XmlNode {
            name: name.to_string(),
            attrs: owned_attrs,
            children,
            has_xlinks,
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

        let resolved = resolve_xlinks(&Arc::new(node), &dummy_url(), &reg);

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
        let target = Arc::new(make_node(
            "gml:Polygon",
            vec![("gml:id", GML_NS, "p1")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert((dummy_url().to_string(), "p1".to_string()), target);

        let node = make_node("ref", vec![("xl:href", XLINK_NS, "#p1")], vec![]);
        let resolved = resolve_xlinks(&Arc::new(node), &dummy_url(), &reg);

        assert_eq!(resolved.children.len(), 1);
    }

    #[test]
    fn resolve_xlinks_handles_cross_file_fragment() {
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
        let resolved = resolve_xlinks(&Arc::new(node), &dummy_url(), &reg);

        assert_eq!(resolved.children.len(), 1);
    }

    #[test]
    fn resolve_xlinks_leaves_unresolvable_in_place() {
        let reg = IdRegistry::new();
        let node = make_node("ref", vec![("xlink:href", XLINK_NS, "#missing")], vec![]);
        let resolved = resolve_xlinks(&Arc::new(node), &dummy_url(), &reg);

        assert!(resolved.children.is_empty());
        assert!(resolved
            .attrs
            .iter()
            .any(|(q, ns, _)| local_name(q) == "href" && ns == XLINK_NS));
        // Unresolved xlink:href must keep has_xlinks true so callers know the
        // subtree still contains dangling references.
        assert!(resolved.has_xlinks);
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

        let resolved = resolve_xlinks(&Arc::new(node), &dummy_url(), &reg);

        let child = match &resolved.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!("expected element"),
        };
        assert_eq!(child.name, "bldg:pos");
        assert_eq!(child.children.len(), 1);
    }

    #[test]
    fn resolve_xlinks_cross_file_fragment_resolved_against_target_file() {
        // other.gml defines poly001.
        // other.gml also contains a node that has a fragment-only href (#poly001),
        // i.e. the xlink target itself contains an internal reference.
        // When the resolver follows the cross-file link it must resolve #poly001
        // against other.gml, not against the originating file (test.gml).
        let other_url = Url::parse("file:///other.gml").unwrap();

        let poly = Arc::new(make_node(
            "gml:Polygon",
            vec![("gml:id", GML_NS, "poly001")],
            vec![],
        ));

        // A wrapper in other.gml that carries a fragment-only href pointing at poly001
        // (same file: other.gml).
        let wrapper = Arc::new(make_node(
            "bldg:lod1Solid",
            vec![
                ("gml:id", GML_NS, "wrapper001"),
                ("xlink:href", XLINK_NS, "#poly001"),
            ],
            vec![],
        ));

        let mut reg = IdRegistry::new();
        reg.insert(
            (other_url.to_string(), "poly001".to_string()),
            Arc::clone(&poly),
        );
        reg.insert(
            (other_url.to_string(), "wrapper001".to_string()),
            Arc::clone(&wrapper),
        );

        // In test.gml we have a cross-file reference to other.gml#wrapper001.
        let node = make_node(
            "ref",
            vec![("xlink:href", XLINK_NS, "other.gml#wrapper001")],
            vec![],
        );

        let resolved = resolve_xlinks(&Arc::new(node), &dummy_url(), &reg);

        // ref → wrapper001 (from other.gml) → poly001 (from other.gml, via #poly001)
        assert_eq!(resolved.children.len(), 1);
        let wrapper_resolved = match &resolved.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!("expected wrapper element"),
        };
        assert_eq!(wrapper_resolved.name, "bldg:lod1Solid");
        assert_eq!(
            wrapper_resolved.children.len(),
            1,
            "wrapper must have resolved its own #poly001 fragment against other.gml"
        );
        let poly_resolved = match &wrapper_resolved.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!("expected polygon element"),
        };
        assert_eq!(poly_resolved.name, "gml:Polygon");
    }

    #[test]
    fn resolve_xlinks_detects_cycle() {
        // A(href=#b) → B(href=#a): back-reference must be blocked without stack overflow.
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

        let resolved = resolve_xlinks(&node_a, &dummy_url(), &reg);

        let b = match resolved.children.first() {
            Some(XmlChild::Element(e)) => e,
            _ => panic!("expected B as first child"),
        };
        assert_eq!(b.name, "B");
        assert!(b.children.is_empty());
    }
}
