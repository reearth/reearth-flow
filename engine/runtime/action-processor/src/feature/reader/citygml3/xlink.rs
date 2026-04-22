use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::parser::{RawChild, RawNode, RawRegistry};
use super::utils::{XmlChild, XmlNode};

/// Phase-2: convert a parsed feature root into a fully resolved `XmlNode`.
/// Each `RawChild::Ref` becomes `XmlChild::Element(Arc<XmlNode>)` — a direct node pointer.
pub fn resolve(
    raws: impl IntoIterator<Item = Arc<RawNode>>,
    registry: &RawRegistry,
) -> Vec<Arc<XmlNode>> {
    // cache of resolved nodes, key is pointer of RawNode safely held by registry
    let mut cache: HashMap<*const RawNode, Arc<XmlNode>> = HashMap::new();
    raws.into_iter()
        .filter_map(|raw| {
            let mut in_progress: HashSet<*const RawNode> = HashSet::new();
            convert_node(&raw, registry, &mut cache, &mut in_progress)
                .or_else(|| {
                    tracing::error!(
                        name = raw.name.0.as_str(),
                        "citygml3: failed to resolve top-level node due to cyclic xlink reference, skipped"
                    );
                    None
                })
        })
        .collect()
}

fn convert_node(
    raw: &Arc<RawNode>,
    registry: &RawRegistry,
    cache: &mut HashMap<*const RawNode, Arc<XmlNode>>,
    in_progress: &mut HashSet<*const RawNode>,
) -> Option<Arc<XmlNode>> {
    let ptr = Arc::as_ptr(raw);
    if let Some(cached) = cache.get(&ptr) {
        return Some(Arc::clone(cached));
    }
    if !in_progress.insert(ptr) {
        tracing::warn!(
            name = raw.name.0.as_str(),
            "citygml3: cyclic xlink reference detected, skipped at cycle boundary"
        );
        return None;
    }

    let children: Vec<XmlChild> = raw
        .children
        .iter()
        .filter_map(|c| match c {
            RawChild::Element(e) => {
                convert_node(e, registry, cache, in_progress).map(XmlChild::Element)
            }
            RawChild::Text(t) => Some(XmlChild::Text(t.clone())),
            RawChild::Ref(key) => {
                if let Some(target) = registry.get(key) {
                    convert_node(target, registry, cache, in_progress).map(XmlChild::Element)
                } else {
                    tracing::warn!(id = key.1, "citygml3: unresolved xlink:href, skipped");
                    None
                }
            }
        })
        .collect();

    let node = raw.with_children(children);
    in_progress.remove(&ptr);
    cache.insert(ptr, Arc::clone(&node));
    Some(node)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use url::Url;

    use super::*;
    use crate::feature::reader::citygml3::parser::{Parser, RawNode, RawRegistry};
    use crate::feature::reader::citygml3::utils::{local_name, XmlChild};

    fn dummy_url() -> Url {
        Url::parse("file:///test.gml").unwrap()
    }

    fn parse_test(xml: &[u8], url: &Url) -> (Vec<Arc<RawNode>>, RawRegistry) {
        let mut parser = Parser::new();
        parser.parse(xml, url).unwrap();
        let (pending, raw_reg, _) = parser.finish();
        (pending, raw_reg)
    }

    #[test]
    fn resolve_href_becomes_element_child() {
        let xml = br##"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <gml:surfaceMember xlink:href="#poly001"/>
      <gml:surfaceMember>
        <gml:Polygon gml:id="poly001"/>
      </gml:surfaceMember>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"##;

        let (raw, reg) = parse_test(xml, &dummy_url());
        let resolved = resolve(raw, &reg);
        assert_eq!(resolved.len(), 1);
        let tlf = &resolved[0];

        let building = tlf;
        let surface_member = match &building.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        match &surface_member.children[0] {
            XmlChild::Element(arc) => assert_eq!(local_name(&arc.name.0), "Polygon"),
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn resolve_shared_node_is_same_arc() {
        // Two surfaceMembers reference the same polygon — both child arcs must be pointer-equal.
        let xml = br##"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <gml:surfaceMember xlink:href="#poly001"/>
      <gml:surfaceMember xlink:href="#poly001"/>
      <gml:surfaceMember>
        <gml:Polygon gml:id="poly001"/>
      </gml:surfaceMember>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"##;

        let (raw, reg) = parse_test(xml, &dummy_url());
        let resolved = resolve(raw, &reg);
        assert_eq!(resolved.len(), 1);
        let tlf = &resolved[0];

        let building = tlf;
        let ref_arc = |i: usize| match &building.children[i] {
            XmlChild::Element(e) => match &e.children[0] {
                XmlChild::Element(arc) => Arc::clone(arc),
                _ => panic!("expected Element"),
            },
            _ => panic!("expected Element"),
        };

        let arc0 = ref_arc(0);
        let arc1 = ref_arc(1);
        assert!(Arc::ptr_eq(&arc0, &arc1), "same polygon must share one Arc");
    }

    #[test]
    fn resolve_unresolvable_ref_is_dropped() {
        let xml = br##"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <gml:surfaceMember xlink:href="#missing"/>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"##;

        let (raw, reg) = parse_test(xml, &dummy_url());
        let resolved = resolve(raw, &reg);
        assert_eq!(resolved.len(), 1);
        let tlf = &resolved[0];

        let building = tlf;
        let surface_member = match &building.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        assert!(surface_member.children.is_empty());
    }

    #[test]
    fn resolve_href_ignores_inline_content() {
        let xml = br##"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <gml:surfaceMember xlink:href="#poly001">
        <gml:Polygon gml:id="poly_inline"/>
      </gml:surfaceMember>
      <gml:surfaceMember>
        <gml:Polygon gml:id="poly001"/>
      </gml:surfaceMember>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"##;

        let (raw, reg) = parse_test(xml, &dummy_url());
        let resolved = resolve(raw, &reg);
        assert_eq!(resolved.len(), 1);
        let tlf = &resolved[0];

        let building = tlf;
        let surface_member = match &building.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        match &surface_member.children[0] {
            XmlChild::Element(arc) => assert_eq!(arc.attrs[0].1, "poly001"),
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn resolve_cross_file_ref() {
        let xml_a = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <gml:surfaceMember xlink:href="b.gml#poly001"/>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"#;

        let xml_b = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember>
    <gml:Polygon gml:id="poly001"/>
  </core:cityObjectMember>
</core:CityModel>"#;

        let url_a = Url::parse("file:///a.gml").unwrap();
        let url_b = Url::parse("file:///b.gml").unwrap();

        let (raw_a, mut reg) = parse_test(xml_a, &url_a);
        let (_, reg_b) = parse_test(xml_b, &url_b);
        reg.extend(reg_b);

        let resolved = resolve(raw_a, &reg);
        assert_eq!(resolved.len(), 1);
        let tlf = &resolved[0];
        let building = tlf;
        let surface_member = match &building.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        match &surface_member.children[0] {
            XmlChild::Element(arc) => assert_eq!(local_name(&arc.name.0), "Polygon"),
            _ => panic!("expected Element child to cross-file polygon"),
        }
    }

    #[test]
    fn resolve_cyclic_ref_terminates() {
        let xml = br##"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <core:cityObjectMember>
    <gml:CompositeSurface gml:id="a">
      <gml:surfaceMember xlink:href="#b"/>
    </gml:CompositeSurface>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <gml:CompositeSurface gml:id="b">
      <gml:surfaceMember xlink:href="#a"/>
    </gml:CompositeSurface>
  </core:cityObjectMember>
</core:CityModel>"##;

        let (raw, reg) = parse_test(xml, &dummy_url());
        let resolved = resolve(raw, &reg);
        assert_eq!(resolved.len(), 2);
        let _ = resolved;
    }

    #[test]
    fn resolve_many_shares_cache_across_entrypoints() {
        let xml = br##"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <gml:surfaceMember xlink:href="#poly001"/>
    </bldg:Building>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg002">
      <gml:surfaceMember xlink:href="#poly001"/>
    </bldg:Building>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <gml:Polygon gml:id="poly001"/>
  </core:cityObjectMember>
</core:CityModel>"##;

        let (raw, reg) = parse_test(xml, &dummy_url());
        let resolved = resolve(raw, &reg);

        let polygon_arc = |feature: &Arc<XmlNode>| match &feature.children[0] {
            XmlChild::Element(e) => match &e.children[0] {
                XmlChild::Element(arc) => Arc::clone(arc),
                _ => panic!("expected referenced polygon"),
            },
            _ => panic!("expected surfaceMember element"),
        };

        let arc0 = polygon_arc(&resolved[0]);
        let arc1 = polygon_arc(&resolved[1]);
        assert!(
            Arc::ptr_eq(&arc0, &arc1),
            "shared xlink target should be resolved once across feature roots"
        );
    }
}
