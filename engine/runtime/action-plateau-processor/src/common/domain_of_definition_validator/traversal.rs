//! Collects, in one pass over a member, the inputs of the checks.

use fastxml::transform::EditableNodeRef;

use super::lod_geometry_type::parse_lod_geometry_level;

const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// The inputs of the checks found under a member, in document order.
#[derive(Default)]
pub(super) struct CollectedDescendants {
    pub(super) gml_id_entries: Vec<GmlIdEntry>,
    pub(super) code_space_entries: Vec<CodeSpaceEntry>,
    pub(super) pos_contents: Vec<String>,
    pub(super) xlink_entries: Vec<CollectedXlinkEntry>,
    pub(super) lod_entries: Vec<LodEntry>,
}

pub(super) struct GmlIdEntry {
    pub(super) gml_id: String,
    pub(super) tag: String,
    pub(super) xpath: String,
}

pub(super) struct CodeSpaceEntry {
    pub(super) code_space: String,
    pub(super) content: String,
    pub(super) tag: String,
    pub(super) xpath: String,
}

pub(super) struct CollectedXlinkEntry {
    pub(super) href: String,
    pub(super) tag: String,
    pub(super) xpath: String,
}

pub(super) struct LodEntry {
    pub(super) lod_level: u8,
    pub(super) parent_tag: String,
    pub(super) parent_gml_id: String,
    pub(super) gml_child_tag: String,
}

pub(super) fn collect_descendant_info(
    member: &EditableNodeRef,
    gml_ns: &str,
    member_tag: &str,
) -> CollectedDescendants {
    let mut result = CollectedDescendants::default();
    let path = vec![member_tag.to_string()];
    collect_recursive(member, gml_ns, XLINK_NS, member_tag, "", &path, &mut result);
    result
}

fn collect_recursive(
    node: &EditableNodeRef,
    gml_ns: &str,
    xlink_ns: &str,
    node_tag: &str,
    node_gml_id: &str,
    path: &[String],
    result: &mut CollectedDescendants,
) {
    for child in node.children() {
        let child_tag = child.qname();
        let child_name = child.name();
        let mut child_path = path.to_vec();
        child_path.push(child_tag.clone());
        let xpath_str = child_path.join("/");

        // gml:id attribute check
        let child_gml_id = child.get_attribute_ns(gml_ns, "id").unwrap_or_default();
        if !child_gml_id.is_empty() {
            result.gml_id_entries.push(GmlIdEntry {
                gml_id: child_gml_id.clone(),
                tag: child_tag.clone(),
                xpath: xpath_str.clone(),
            });
        }

        // codeSpace attribute check
        if let Some(cs) = child.get_attribute("codeSpace") {
            result.code_space_entries.push(CodeSpaceEntry {
                code_space: cs,
                content: child.get_content().unwrap_or_default(),
                tag: child_tag.clone(),
                xpath: xpath_str.clone(),
            });
        }

        // gml:pos / gml:posList check
        if child.namespace_uri().as_deref() == Some(gml_ns)
            && (child_name == "pos" || child_name == "posList")
            && let Some(content) = child.get_content()
        {
            result.pos_contents.push(content);
        }

        // xlink:href attribute check
        if let Some(href) = child.get_attribute_ns(xlink_ns, "href") {
            result.xlink_entries.push(CollectedXlinkEntry {
                href,
                tag: child_tag.clone(),
                xpath: xpath_str.clone(),
            });
        }

        // lod*Geometry check (parent = current node)
        if let Some(level) = parse_lod_geometry_level(&child_name) {
            let gml_child_tag = child
                .children()
                .iter()
                .find(|c| c.namespace_uri().as_deref() == Some(gml_ns))
                .map(|c| c.qname())
                .unwrap_or_default();
            result.lod_entries.push(LodEntry {
                lod_level: level,
                parent_tag: node_tag.to_string(),
                parent_gml_id: node_gml_id.to_string(),
                gml_child_tag,
            });
        }

        // Recurse
        collect_recursive(
            &child,
            gml_ns,
            xlink_ns,
            &child_tag,
            &child_gml_id,
            &child_path,
            result,
        );
    }
}
