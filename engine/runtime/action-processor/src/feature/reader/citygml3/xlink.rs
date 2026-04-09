use std::collections::HashMap;
use std::sync::Arc;

use super::parser::{RawChild, RawNode, RawRegistry};
use super::utils::{XmlChild, XmlNode};

/// Phase-2: convert a parsed feature root into a fully resolved `XmlNode`.
/// Each `RawChild::Ref` becomes `XmlChild::Element(Arc<XmlNode>)` — a direct node pointer.
/// Call once per feature after all files have been parsed and the registry is complete.
pub fn resolve(raw: Arc<RawNode>, registry: &RawRegistry) -> Arc<XmlNode> {
    let mut cache: HashMap<*const RawNode, Arc<XmlNode>> = HashMap::new();
    convert_node(&raw, registry, &mut cache)
}

fn convert_node(
    raw: &Arc<RawNode>,
    registry: &RawRegistry,
    cache: &mut HashMap<*const RawNode, Arc<XmlNode>>,
) -> Arc<XmlNode> {
    let ptr = Arc::as_ptr(raw);
    if let Some(cached) = cache.get(&ptr) {
        return Arc::clone(cached);
    }

    let children: Vec<XmlChild> = raw
        .children
        .iter()
        .filter_map(|c| match c {
            RawChild::Element(e) => Some(XmlChild::Element(convert_node(e, registry, cache))),
            RawChild::Text(t) => Some(XmlChild::Text(t.clone())),
            RawChild::Ref(key) => {
                if let Some(target) = registry.get(key) {
                    Some(XmlChild::Element(convert_node(target, registry, cache)))
                } else {
                    tracing::warn!(id = key.1, "citygml3: unresolved xlink:href, skipped");
                    None
                }
            }
        })
        .collect();

    let node = Arc::new(XmlNode {
        name: raw.name.clone(),
        attrs: raw.attrs.clone(),
        children,
    });
    cache.insert(ptr, Arc::clone(&node));
    node
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use url::Url;

    use super::*;
    use crate::feature::reader::citygml3::parser::{parse, RawRegistry};
    use crate::feature::reader::citygml3::utils::{local_name, XmlChild};

    fn dummy_url() -> Url {
        Url::parse("file:///test.gml").unwrap()
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

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();
        let tlf = resolve(raw.into_iter().next().unwrap(), &reg);

        let building = &tlf;
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

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();
        let tlf = resolve(raw.into_iter().next().unwrap(), &reg);

        let building = &tlf;
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

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();
        let tlf = resolve(raw.into_iter().next().unwrap(), &reg);

        let building = &tlf;
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

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();
        let tlf = resolve(raw.into_iter().next().unwrap(), &reg);

        let building = &tlf;
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

        let mut reg = RawRegistry::new();
        let raw_a = parse(xml_a, &url_a, &mut reg).unwrap();
        parse(xml_b, &url_b, &mut reg).unwrap();

        let tlf = resolve(raw_a.into_iter().next().unwrap(), &reg);
        let building = &tlf;
        let surface_member = match &building.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        match &surface_member.children[0] {
            XmlChild::Element(arc) => assert_eq!(local_name(&arc.name.0), "Polygon"),
            _ => panic!("expected Element child to cross-file polygon"),
        }
    }
}
