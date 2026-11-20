use quick_xml::{
    NsReader,
    events::BytesStart,
    name::{Namespace, ResolveResult},
};

/// The gml:id of `element`, or `None` when it has none.
pub(crate) fn gml_id<R>(
    reader: &NsReader<R>,
    element: &BytesStart,
    gml_ns: &str,
) -> quick_xml::Result<Option<String>> {
    for attr in element.attributes().flatten() {
        let (ns, local_name) = reader.resolve_attribute(attr.key);
        if local_name.as_ref() == b"id"
            && matches!(ns, ResolveResult::Bound(Namespace(uri)) if uri == gml_ns.as_bytes())
        {
            return Ok(Some(attr.unescape_value()?.into_owned()));
        }
    }
    Ok(None)
}
