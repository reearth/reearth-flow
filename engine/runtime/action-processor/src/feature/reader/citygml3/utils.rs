use std::sync::Arc;

pub(super) const GML_NS: &str = "http://www.opengis.net/gml/3.2";
pub(super) const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// `(qname, ns-uri)`
pub type QName = (String, String);

#[derive(Debug, Clone)]
pub struct XmlNode {
    pub name: QName,
    pub attrs: Vec<(QName, String)>,
    pub children: Vec<XmlChild>,
}

#[derive(Debug, Clone)]
pub enum XmlChild {
    Element(Arc<XmlNode>),
    Text(String),
}

pub(super) fn local_name(qname: &str) -> &str {
    qname.rfind(':').map(|i| &qname[i + 1..]).unwrap_or(qname)
}

pub(super) fn gml_id_attr(node: &XmlNode) -> Option<String> {
    node.attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "id" && ns == GML_NS)
        .map(|(_, v)| v.clone())
}

pub(super) fn xlink_href_attr(attrs: &[(QName, String)]) -> Option<&str> {
    attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "href" && ns == XLINK_NS)
        .map(|(_, v)| v.as_str())
}
