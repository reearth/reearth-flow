//! Collects, in one pass over a member, the inputs of the checks.

use std::collections::HashMap;

use fastxml::transform::EditableNodeRef;

use super::extents::Positions;
use super::lod_geometry_type::parse_lod_geometry_level;

const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// The inputs of the checks found under a member, in document order.
#[derive(Default)]
pub(super) struct CollectedDescendants {
    pub(super) gml_id_entries: Vec<GmlIdEntry>,
    pub(super) code_space_entries: Vec<CodeSpaceEntry>,
    /// The coordinates of the `gml:pos` / `gml:posList`; `None` when not
    /// collected.
    pub(super) positions: Option<Positions>,
    pub(super) xlinks: CollectedXlinks,
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

/// The xlinks under a member.
#[derive(Default)]
pub(super) struct CollectedXlinks {
    pub(super) entries: Vec<CollectedXlinkEntry>,
    /// The namespace URI of each prefix of the elements of the member, by its
    /// first element.
    pub(super) namespaces: HashMap<String, String>,
}

pub(super) struct CollectedXlinkEntry {
    pub(super) href: String,
    pub(super) tag: String,
    pub(super) xpath: String,
    /// The element holding `tag`.
    pub(super) holder_tag: String,
}

pub(super) struct LodEntry {
    pub(super) lod_level: u8,
    pub(super) parent_tag: String,
    pub(super) parent_gml_id: String,
    pub(super) gml_child_tag: String,
}

/// Collects the inputs of the checks under `member`, whose element is
/// `member_tag`. The coordinates are added to `positions` unless it is `None`.
pub(super) fn collect_descendant_info(
    member: &EditableNodeRef,
    gml_ns: &str,
    member_tag: &str,
    positions: Option<Positions>,
) -> CollectedDescendants {
    let mut result = CollectedDescendants {
        positions,
        ..CollectedDescendants::default()
    };
    add_namespace(member, &member.qname(), &mut result.xlinks.namespaces);
    let mut path = vec![member_tag.to_string()];
    collect_recursive(member, gml_ns, member_tag, "", &mut path, &mut result);
    result
}

/// Collects the inputs under `node`, whose element is `node_tag`, at the
/// element path `path`.
fn collect_recursive(
    node: &EditableNodeRef,
    gml_ns: &str,
    node_tag: &str,
    node_gml_id: &str,
    path: &mut Vec<String>,
    result: &mut CollectedDescendants,
) {
    for child in node.children() {
        let child_tag = child.qname();
        let local_name = child_tag
            .split_once(':')
            .map_or(child_tag.as_str(), |(_, local)| local);
        path.push(child_tag.clone());
        add_namespace(&child, &child_tag, &mut result.xlinks.namespaces);
        // Built only for an element that reports it.
        let mut joined_path = None;
        let mut xpath = || joined_path.get_or_insert_with(|| path.join("/")).clone();

        let child_gml_id = child.get_attribute_ns(gml_ns, "id").unwrap_or_default();
        if !child_gml_id.is_empty() {
            result.gml_id_entries.push(GmlIdEntry {
                gml_id: child_gml_id.clone(),
                tag: child_tag.clone(),
                xpath: xpath(),
            });
        }

        if let Some(code_space) = child.get_attribute("codeSpace") {
            result.code_space_entries.push(CodeSpaceEntry {
                code_space,
                content: child.get_content().unwrap_or_default(),
                tag: child_tag.clone(),
                xpath: xpath(),
            });
        }

        if let Some(positions) = &mut result.positions
            && (local_name == "pos" || local_name == "posList")
            && child.namespace_uri().as_deref() == Some(gml_ns)
            && let Some(content) = child.get_content()
        {
            positions.add_content(&content);
        }

        if let Some(href) = child.get_attribute_ns(XLINK_NS, "href") {
            result.xlinks.entries.push(CollectedXlinkEntry {
                href,
                tag: child_tag.clone(),
                xpath: xpath(),
                holder_tag: node_tag.to_string(),
            });
        }

        // The parent of a lod*Geometry is `node`.
        if let Some(level) = parse_lod_geometry_level(local_name) {
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

        collect_recursive(&child, gml_ns, &child_tag, &child_gml_id, path, result);
        path.pop();
    }
}

/// Adds the namespace URI of the prefix of `element`, whose name is `qname`,
/// unless the prefix has one.
fn add_namespace(element: &EditableNodeRef, qname: &str, namespaces: &mut HashMap<String, String>) {
    let prefix = qname.split_once(':').map_or("", |(prefix, _)| prefix);
    if namespaces.contains_key(prefix) {
        return;
    }
    if let Some(namespace) = element.namespace_uri() {
        namespaces.insert(prefix.to_string(), namespace);
    }
}
