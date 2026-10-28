//! The tree of city objects one feature is written as.
//!
//! A feature read from CityGML carries its nested objects twice: their
//! geometry as members keyed by owner, and their data in the attribute tree,
//! nested as in the source. This module takes the nested objects out of the
//! attribute tree ([`split_nested_objects`]) and joins the two by key into one
//! tree of [`ObjectNode`]s ([`build_feature_objects`]). An object that cannot
//! be nested is returned as an orphan, to be written top-level. Across the
//! write, [`join`] then nests each feature under the object its parent key
//! names, which is another feature or an object nested in one.

use std::collections::{HashMap, HashSet};

use reearth_flow_citygml::pipeline::SYNTHETIC_OWNER_TREE_KEY;
use reearth_flow_citygml::schema::{QName, SchemaSet};
use reearth_flow_diagnostics::ErrorCode;
use reearth_flow_types::{AttributeValue, Attributes};

use super::classes::{class_of_element, CityClass, ClassResolver, Prefixes};
use super::model::{GeometryEntry, GmlElement, GmlSolid, GmlSurface, NestedObject, ObjectNode};
use super::placement::{self, is_city_object, Placed};
use super::properties::{Skip, XmlProperty};
use super::writer::is_ncname;

const CORE: &str = "http://www.opengis.net/citygml/2.0";
const GRP: &str = "http://www.opengis.net/citygml/cityobjectgroup/2.0";
/// An object's `gml:id` as the readers write it in the attribute tree.
const GML_ID_KEY: &str = "@gml:id";

/// Properties that refer to a city object rather than nest it, as
/// `(namespace, local name)`. Every city object can hold any other through
/// `core:generalizesTo`, so these never place a nested object.
pub const REFERENCE_PROPERTIES: [(&str, &str); 3] = [
    (CORE, "generalizesTo"),
    (GRP, "groupMember"),
    (GRP, "parent"),
];

/// A nested city object taken out of the attribute tree.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeObject {
    /// The property the source wrapped the object in, such as `bldg:boundedBy`.
    pub wrapper: QName,
    /// The object element, without the objects nested in it.
    pub element: XmlProperty,
    pub children: Vec<TreeObject>,
}

/// Takes the nested city objects out of `properties`. Returns the properties
/// left for placement and the objects, each with its own nested objects.
pub fn split_nested_objects(
    schema: &SchemaSet,
    prefixes: &Prefixes,
    properties: Vec<XmlProperty>,
) -> (Vec<XmlProperty>, Vec<TreeObject>) {
    let mut remaining = Vec::new();
    let mut tree = Vec::new();
    for property in properties {
        let wrapper = property.qname(prefixes);
        let holds_object = !is_reference(&wrapper)
            && property
                .children
                .iter()
                .any(|child| is_city_object(schema, &child.qname(prefixes)));
        if !holds_object {
            remaining.push(property);
            continue;
        }
        // The wrapper goes: the writer writes it around the object itself.
        for mut child in property.children {
            if !is_city_object(schema, &child.qname(prefixes)) {
                continue;
            }
            let (rest, children) =
                split_nested_objects(schema, prefixes, std::mem::take(&mut child.children));
            child.children = rest;
            tree.push(TreeObject {
                wrapper: wrapper.clone(),
                element: child,
                children,
            });
        }
    }
    (remaining, tree)
}

fn is_reference(property: &QName) -> bool {
    REFERENCE_PROPERTIES
        .iter()
        .any(|(namespace, local)| property.namespace == *namespace && property.local == *local)
}

/// The slot of `parent`'s class that nests a `child`, and the property
/// declared there. Reference properties never do.
pub fn containment_slot(
    schema: &SchemaSet,
    parent: &QName,
    child: &QName,
) -> Option<(usize, QName)> {
    containment_slot_via(schema, parent, child, None)
}

/// [`containment_slot`], taking the slot declared as `wrapper` when that one
/// can hold `child`.
///
/// The first slot that can hold a child is not always the one the source
/// used. In CityGML 2.0 each class nests a given child through one property,
/// but an ADE may declare a broader one ahead of it: i-UR's `urf:target`, on
/// every urban function and typed for any city object, precedes
/// `urf:districtDevelopmentPlan`. Both are valid, so the schema gate cannot
/// tell them apart; only the source's own wrapper keeps the meaning.
pub fn containment_slot_via(
    schema: &SchemaSet,
    parent: &QName,
    child: &QName,
    wrapper: Option<&QName>,
) -> Option<(usize, QName)> {
    let class = schema.class_for_element(parent)?;
    let holding: Vec<(usize, QName)> = schema
        .slots_holding(parent, child)
        .into_iter()
        .filter_map(|index| {
            let property = class.slots()[index].declared.clone()?;
            (!is_reference(&property)).then_some((index, property))
        })
        .collect();
    let preferred =
        wrapper.and_then(|wrapper| holding.iter().position(|(_, property)| property == wrapper));
    holding.into_iter().nth(preferred.unwrap_or(0))
}

/// Which properties nest a city object in which classes, each worked out
/// once per write.
#[derive(Debug, Default)]
pub struct ContainmentProperties {
    known: HashMap<(QName, QName), bool>,
}

impl ContainmentProperties {
    /// Whether `property`, in an object of class `parent`, is one that nests
    /// a city object, such as `bldg:opening` in a wall surface. Reference
    /// properties never are.
    pub fn holds_objects(&mut self, schema: &SchemaSet, parent: &QName, property: &QName) -> bool {
        let key = (parent.clone(), property.clone());
        if let Some(&holds) = self.known.get(&key) {
            return holds;
        }
        let holds = !is_reference(property)
            && schema.class_for_element(parent).is_some_and(|class| {
                let slots: Vec<usize> = class
                    .slots()
                    .iter()
                    .enumerate()
                    .filter(|(_, slot)| slot.declared.as_ref() == Some(property))
                    .map(|(index, _)| index)
                    .collect();
                !slots.is_empty()
                    && schema.global_elements().any(|element| {
                        is_city_object(schema, element)
                            && schema
                                .slots_holding(parent, element)
                                .iter()
                                .any(|slot| slots.contains(slot))
                    })
            });
        self.known.insert(key, holds);
        holds
    }
}

/// One feature, ready to be built into its object tree.
#[derive(Debug, Clone)]
pub struct FeatureParts {
    /// The feature's index in the write.
    pub source: usize,
    pub class: CityClass,
    pub gml_id: Option<String>,
    /// The key of the object the feature itself nests under.
    pub parent_key: Option<String>,
    /// The `gml:id` of the top-level object of the document the feature was
    /// extracted from, which scopes the synthetic keys its parent key may name.
    pub document_root: Option<String>,
    pub geometries: Vec<GeometryEntry>,
    /// The feature's own properties, placed in its class.
    pub properties: Vec<Placed>,
    /// The nested objects of its attribute tree.
    pub tree: Vec<TreeObject>,
    /// The keys of the objects its attribute tree holds through a reference
    /// property, such as a group's `grp:groupMember`, at any depth.
    pub referenced: HashSet<String>,
}

/// One feature's object tree.
#[derive(Debug, Clone)]
pub struct FeatureObjects {
    pub key: Option<String>,
    pub parent_key: Option<String>,
    pub document_root: Option<String>,
    pub root: ObjectNode,
    /// Objects that could not be nested, to be written top-level.
    pub orphans: Vec<ObjectNode>,
    /// The codes to report against the feature.
    pub reports: HashSet<ErrorCode>,
    /// The keys of its objects made from geometry members alone, with no
    /// data from the attribute tree.
    pub geometry_only: HashSet<String>,
}

/// The feature itself, always node 0.
const ROOT: usize = 0;

/// One object while the tree is built.
struct Node {
    /// `None` for an owner that names no city-object class; it is folded
    /// into the nearest object above it that has one.
    class: Option<CityClass>,
    key: Option<String>,
    synthetic: bool,
    /// The object's own data, for an object read from the attribute tree.
    data: Option<Vec<XmlProperty>>,
    /// The property the source wrapped the object in, for an object read
    /// from the attribute tree.
    wrapper: Option<QName>,
    /// Made from geometry members whose key the attribute tree lacks.
    group: bool,
    /// `None` for the root, and for a group whose parent key names no node.
    parent: Option<usize>,
    geometries: Vec<GeometryEntry>,
    /// Cleared when the node is dropped or folded.
    kept: bool,
}

impl Node {
    fn tree(class: Option<CityClass>, wrapper: QName, element: XmlProperty, parent: usize) -> Self {
        let XmlProperty {
            attrs,
            children,
            synthetic_key,
            ..
        } = element;
        // The writer writes the object element itself, so of its XML
        // attributes only the identity is kept, as the key.
        let id = attrs
            .into_iter()
            .find(|(name, _)| name == "gml:id")
            .map(|(_, id)| id);
        let synthetic = id.is_none() && synthetic_key.is_some();
        Self {
            class,
            key: id.or(synthetic_key),
            synthetic,
            data: Some(children),
            wrapper: Some(wrapper),
            group: false,
            parent: Some(parent),
            geometries: Vec::new(),
            kept: true,
        }
    }
}

/// Builds one feature's object tree from its geometry members and the
/// nested objects of its attribute tree.
///
/// Members are grouped by owner key onto the attribute tree's object with
/// that key, or onto a new object of the owner's type when the tree has
/// none. A member naming no owner goes to its owner parent, else to the
/// feature. An owner that is not a city object is folded into the nearest
/// object above it that is. Objects whose parent cannot be found, cannot hold
/// them, or leads back to them are returned as orphans, and reported.
pub fn build_feature_objects(
    parts: FeatureParts,
    schema: &SchemaSet,
    prefixes: &Prefixes,
    classes: &mut ClassResolver,
) -> FeatureObjects {
    let FeatureParts {
        source,
        class,
        gml_id,
        parent_key,
        document_root,
        geometries,
        mut properties,
        tree,
        referenced,
    } = parts;
    let mut reports = HashSet::new();

    // Nodes: the feature, then the attribute tree's objects in pre-order.
    let mut nodes = vec![Node {
        class: Some(class),
        key: gml_id.clone(),
        synthetic: false,
        data: None,
        wrapper: None,
        group: false,
        parent: None,
        geometries: Vec::new(),
        kept: true,
    }];
    let mut index: HashMap<String, usize> = HashMap::new();
    if let Some(id) = &gml_id {
        index.insert(id.clone(), ROOT);
    }
    add_tree_objects(&mut nodes, &mut index, tree, ROOT, schema, prefixes);

    // The geometry of an object the feature holds only through a reference,
    // such as a group's member, belongs to that object, which is written
    // where it lives. The reference itself is reported where it is placed.
    let geometries = without_referenced(geometries, &referenced, &index);

    // Entries, by owner key; a new key makes a group in first-seen order.
    let mut group_parents: Vec<(usize, Option<String>)> = Vec::new();
    let mut keyless = Vec::new();
    for entry in geometries {
        let Some(key) = entry.owner.key.clone() else {
            // A key-less member naming a city object other than the
            // feature's own class is an object of its own, which gets a
            // generated id. One naming the feature's class, as a reader
            // writes for an id-less feature, is the feature's.
            let class = entry
                .owner
                .feature_type
                .as_deref()
                .and_then(|value| classes.class_of(value))
                .filter(|class| {
                    nodes[ROOT].class.as_ref().map(|own| &own.element) != Some(&class.element)
                });
            let Some(class) = class else {
                keyless.push(entry);
                continue;
            };
            let node = nodes.len();
            group_parents.push((node, entry.owner.parent_key.clone()));
            nodes.push(Node {
                class: Some(class),
                key: None,
                synthetic: false,
                data: None,
                wrapper: None,
                group: true,
                parent: None,
                geometries: vec![entry],
                kept: true,
            });
            continue;
        };
        let target = match index.get(&key) {
            Some(&node) => node,
            None => {
                let class = match entry.owner.feature_type.as_deref() {
                    Some(value) => {
                        let class = classes.class_of(value);
                        if class.is_none() && names_no_element(value, schema, prefixes) {
                            reports.insert(ErrorCode::CitygmlUnknownFeatureType);
                        }
                        class
                    }
                    // A source object with an id always has a type, so its
                    // absence is lost data, not an id-less ADE owner.
                    None => {
                        if !entry.owner.synthetic {
                            reports.insert(ErrorCode::CitygmlUnknownFeatureType);
                        }
                        None
                    }
                };
                let node = nodes.len();
                group_parents.push((node, entry.owner.parent_key.clone()));
                nodes.push(Node {
                    class,
                    key: Some(key.clone()),
                    synthetic: entry.owner.synthetic,
                    data: None,
                    wrapper: None,
                    group: true,
                    parent: None,
                    geometries: Vec::new(),
                    kept: true,
                });
                index.insert(key, node);
                node
            }
        };
        nodes[target].geometries.push(entry);
    }
    for entry in keyless {
        let target = entry
            .owner
            .parent_key
            .as_ref()
            .and_then(|key| index.get(key).copied())
            .unwrap_or(ROOT);
        nodes[target].geometries.push(entry);
    }
    for (node, parent_key) in group_parents {
        nodes[node].parent = match parent_key {
            None => Some(ROOT),
            Some(key) => index.get(&key).copied(),
        };
    }

    // Unmatched data: a group of a city-object class means the attribute
    // tree lacks a key, so its key-less objects cannot be told apart and are
    // not written a second time as data-only objects.
    if nodes.iter().any(|node| node.group && node.class.is_some()) {
        let mut dropped = false;
        for node in nodes.iter_mut().skip(1) {
            if !node.group && node.key.is_none() {
                node.kept = false;
                dropped = true;
            }
        }
        if dropped {
            reports.insert(ErrorCode::CitygmlNestedDataNotMatched);
            for node in 1..nodes.len() {
                let mut parent = nodes[node].parent;
                while let Some(p) = parent.filter(|&p| !nodes[p].kept) {
                    parent = nodes[p].parent;
                }
                nodes[node].parent = parent;
            }
        }
    }

    // Fold every owner that is not a city object.
    for node in 1..nodes.len() {
        if !nodes[node].kept || nodes[node].class.is_some() {
            continue;
        }
        let target = nearest_classed_ancestor(&nodes, node);
        if nodes[node]
            .data
            .as_ref()
            .is_some_and(|data| !data.is_empty())
        {
            reports.insert(ErrorCode::CitygmlNestedObjectNotWritten);
        }
        // An owner the reader keyed may be an element in the data of the
        // object it is folded into, such as an ADE attribute declaring a
        // geometry property of its own: its geometry goes there instead.
        let mut entries = std::mem::take(&mut nodes[node].geometries);
        if let Some(key) = nodes[node].key.clone() {
            let mut kept = Vec::new();
            for entry in entries {
                let data: Vec<&mut XmlProperty> = if target == ROOT {
                    properties
                        .iter_mut()
                        .map(|placed| &mut placed.property)
                        .collect()
                } else {
                    nodes[target].data.iter_mut().flatten().collect()
                };
                if let Err(entry) =
                    placement::place_data_geometry(schema, prefixes, data, &key, Box::new(entry))
                {
                    kept.push(*entry);
                }
            }
            entries = kept;
        }
        nodes[target].geometries.extend(entries);
        for other in nodes.iter_mut() {
            if other.parent == Some(node) {
                other.parent = Some(target);
            }
        }
        nodes[node].kept = false;
    }

    // Link: a parent key found nowhere, then each cycle's lowest node.
    let mut orphans: Vec<usize> = Vec::new();
    for (node, entry) in nodes.iter().enumerate().skip(1) {
        if entry.kept && entry.parent.is_none() {
            orphans.push(node);
            reports.insert(ErrorCode::CitygmlParentNotFound);
        }
    }
    loop {
        let reached = reachable(&nodes, &orphans);
        let Some(start) = (1..nodes.len()).find(|&node| nodes[node].kept && !reached[node]) else {
            break;
        };
        let mut path = Vec::new();
        let mut current = start;
        while !path.contains(&current) {
            path.push(current);
            current = nodes[current].parent.unwrap_or(ROOT);
        }
        let from = path.iter().position(|&node| node == current).unwrap_or(0);
        let lowest = path[from..].iter().copied().min().unwrap_or(start);
        orphans.push(lowest);
        reports.insert(ErrorCode::CitygmlParentCycle);
    }

    // Place each object in its parent, in pre-order, then build the orphans.
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (node, entry) in nodes.iter().enumerate().skip(1) {
        if entry.kept && !orphans.contains(&node) {
            if let Some(parent) = entry.parent {
                children[parent].push(node);
            }
        }
    }
    let mut builder = Builder {
        schema,
        prefixes,
        source,
        nodes,
        children,
        root_properties: Some(properties),
        root_gml_id: gml_id.clone(),
        orphans,
        reports,
        geometry_only: HashSet::new(),
    };
    let root = builder.object(ROOT);
    let mut built: Vec<(usize, ObjectNode)> = Vec::new();
    while let Some(node) = builder.orphans.pop() {
        built.push((node, builder.object(node)));
    }
    built.sort_by_key(|(node, _)| *node);

    FeatureObjects {
        key: gml_id,
        parent_key,
        document_root,
        root,
        orphans: built.into_iter().map(|(_, object)| object).collect(),
        reports: builder.reports,
        geometry_only: builder.geometry_only,
    }
}

fn add_tree_objects(
    nodes: &mut Vec<Node>,
    index: &mut HashMap<String, usize>,
    tree: Vec<TreeObject>,
    parent: usize,
    schema: &SchemaSet,
    prefixes: &Prefixes,
) {
    for object in tree {
        let class = class_of_element(&object.element.qname(prefixes), prefixes, schema);
        let node = nodes.len();
        nodes.push(Node::tree(class, object.wrapper, object.element, parent));
        if let Some(key) = &nodes[node].key {
            index.entry(key.clone()).or_insert(node);
        }
        add_tree_objects(nodes, index, object.children, node, schema, prefixes);
    }
}

/// `geometries` without the members of objects held through a reference: a
/// member whose owner, or an owner its owner parent keys lead up to, is
/// `referenced`, before the chain reaches an object the feature holds itself.
fn without_referenced(
    geometries: Vec<GeometryEntry>,
    referenced: &HashSet<String>,
    held: &HashMap<String, usize>,
) -> Vec<GeometryEntry> {
    if referenced.is_empty() {
        return geometries;
    }
    let mut parents: HashMap<&str, Option<&str>> = HashMap::new();
    for entry in &geometries {
        if let Some(key) = entry.owner.key.as_deref() {
            parents
                .entry(key)
                .or_insert(entry.owner.parent_key.as_deref());
        }
    }
    let left_out: Vec<bool> = geometries
        .iter()
        .map(|entry| {
            let mut current = entry
                .owner
                .key
                .as_deref()
                .or(entry.owner.parent_key.as_deref());
            // Bounded, as parent keys may form a cycle.
            for _ in 0..=parents.len() {
                let Some(key) = current else {
                    return false;
                };
                if held.contains_key(key) {
                    return false;
                }
                if referenced.contains(key) {
                    return true;
                }
                current = parents.get(key).copied().flatten();
            }
            false
        })
        .collect();
    geometries
        .into_iter()
        .zip(left_out)
        .filter_map(|(entry, left_out)| (!left_out).then_some(entry))
        .collect()
}

/// The keys, `gml:id` or synthetic, of every object `attributes` holds
/// through a reference property ([`REFERENCE_PROPERTIES`]), at any depth.
///
/// Read from the attributes themselves rather than the properties built from
/// them, as a reference property may be in a namespace the writer does not
/// bind, such as the group module's, and so never become a property. Such a
/// prefix is matched by the name CityGML 2.0 gives its namespace.
pub fn referenced_keys(attributes: &Attributes, prefixes: &Prefixes) -> HashSet<String> {
    let mut keys = HashSet::new();
    for (name, value) in attributes {
        collect_referenced(
            value,
            is_reference_name(name.as_str(), prefixes),
            prefixes,
            &mut keys,
        );
    }
    keys
}

fn collect_referenced(
    value: &AttributeValue,
    referenced: bool,
    prefixes: &Prefixes,
    keys: &mut HashSet<String>,
) {
    match value {
        AttributeValue::Array(items) => {
            for item in items {
                collect_referenced(item, referenced, prefixes, keys);
            }
        }
        AttributeValue::Map(map) => {
            if referenced {
                for name in [GML_ID_KEY, SYNTHETIC_OWNER_TREE_KEY] {
                    if let Some(AttributeValue::String(key)) = map.get(name) {
                        if !key.is_empty() {
                            keys.insert(key.clone());
                        }
                    }
                }
            }
            for (name, child) in map {
                let referenced = referenced || is_reference_name(name.as_str(), prefixes);
                collect_referenced(child, referenced, prefixes, keys);
            }
        }
        _ => {}
    }
}

/// Whether an attribute named `prefix:local` is a reference property.
fn is_reference_name(name: &str, prefixes: &Prefixes) -> bool {
    let Some((prefix, local)) = name.split_once(':') else {
        return false;
    };
    let namespace = prefixes.namespace(prefix).or(match prefix {
        "core" => Some(CORE),
        "grp" => Some(GRP),
        _ => None,
    });
    namespace.is_some_and(|namespace| is_reference(&QName::new(namespace, local)))
}

/// Whether an owner type that resolved to no class names nothing at all: a
/// bare name, or a bound prefix with a local name the schema does not
/// declare, such as a misspelling. An unbound prefix, such as an ADE left out
/// of `adeSchemas`, and a declared element that is not a city object are
/// owners the writer folds by design.
fn names_no_element(value: &str, schema: &SchemaSet, prefixes: &Prefixes) -> bool {
    let Some((prefix, local)) = value.split_once(':') else {
        return true;
    };
    let Some(namespace) = prefixes.namespace(prefix) else {
        return false;
    };
    let element = QName::new(namespace, local);
    !schema.global_elements().any(|known| *known == element)
}

/// The nearest node above `node` with a class, or the root. A parent key
/// that names no node, or a chain that never reaches one, ends at the root.
fn nearest_classed_ancestor(nodes: &[Node], node: usize) -> usize {
    let mut current = nodes[node].parent;
    for _ in 0..nodes.len() {
        match current {
            Some(parent) if nodes[parent].kept && nodes[parent].class.is_some() => return parent,
            Some(parent) => current = nodes[parent].parent,
            None => break,
        }
    }
    ROOT
}

/// Which kept nodes hang, through parent links, from the root or an orphan.
fn reachable(nodes: &[Node], orphans: &[usize]) -> Vec<bool> {
    let mut reached = vec![false; nodes.len()];
    reached[ROOT] = true;
    for &orphan in orphans {
        reached[orphan] = true;
    }
    // Each pass extends every chain by at least one link.
    let mut changed = true;
    while changed {
        changed = false;
        for (node, entry) in nodes.iter().enumerate() {
            if !reached[node] && entry.kept && entry.parent.is_some_and(|p| reached[p]) {
                reached[node] = true;
                changed = true;
            }
        }
    }
    reached
}

struct Builder<'a> {
    schema: &'a SchemaSet,
    prefixes: &'a Prefixes,
    source: usize,
    nodes: Vec<Node>,
    children: Vec<Vec<usize>>,
    root_properties: Option<Vec<Placed>>,
    root_gml_id: Option<String>,
    orphans: Vec<usize>,
    reports: HashSet<ErrorCode>,
    geometry_only: HashSet<String>,
}

impl Builder<'_> {
    /// `node` and everything nested in it. A child its class cannot hold is
    /// queued as an orphan instead.
    fn object(&mut self, node: usize) -> ObjectNode {
        let class = self.nodes[node]
            .class
            .clone()
            .unwrap_or_else(CityClass::generic);
        let mut nested = Vec::new();
        for child in std::mem::take(&mut self.children[node]) {
            let child_class = self.nodes[child]
                .class
                .as_ref()
                .map(|class| class.element.clone());
            let wrapper = self.nodes[child].wrapper.as_ref();
            match child_class.and_then(|element| {
                containment_slot_via(self.schema, &class.element, &element, wrapper)
            }) {
                Some((slot, property)) => nested.push(NestedObject {
                    slot,
                    property,
                    object: self.object(child),
                }),
                None => {
                    self.reports.insert(ErrorCode::CitygmlParentCannotHold);
                    self.orphans.push(child);
                }
            }
        }

        let properties = if node == ROOT {
            self.root_properties.take().unwrap_or_default()
        } else {
            match self.nodes[node].data.take() {
                Some(data) => {
                    let (placed, skipped) =
                        placement::place(self.schema, self.prefixes, &class.element, data);
                    self.reports.extend(skipped.into_iter().map(Skip::code));
                    placed
                }
                None => Vec::new(),
            }
        };
        let entry = &mut self.nodes[node];
        if entry.group {
            if let Some(key) = &entry.key {
                self.geometry_only.insert(key.clone());
            }
        }
        let gml_id = if node == ROOT {
            self.root_gml_id.clone()
        } else {
            entry
                .key
                .clone()
                .filter(|key| !entry.synthetic && is_ncname(key))
        };
        ObjectNode {
            class,
            key: entry.key.clone(),
            synthetic: entry.synthetic,
            gml_id,
            geometries: std::mem::take(&mut entry.geometries),
            properties,
            children: nested,
            source: self.source,
        }
    }
}

/// A city object to write as a `core:cityObjectMember`.
#[derive(Debug, Clone)]
pub struct TopLevel {
    pub object: ObjectNode,
    /// An object of its feature that could not be nested, rather than a
    /// feature itself.
    pub orphan: bool,
    /// The features whose own object this is or holds, nested at any depth.
    pub roots: Vec<usize>,
}

/// Every feature's objects, joined into the trees the document is written as.
#[derive(Debug, Clone, Default)]
pub struct Joined {
    pub top: Vec<TopLevel>,
    /// Per feature, whether it was left out whole as a copy of another
    /// feature.
    pub copies: Vec<bool>,
    /// `(source, code)`, each pair once, in the order found.
    pub reports: Vec<(usize, ErrorCode)>,
}

/// Joins each feature into the object its parent key names: another
/// feature, or an object nested in one.
///
/// Copies are left out first and reported. Of the features that are one
/// feature, all but the best placed are left out whole ([`whole_copies`]).
/// Then, of the
/// nested objects and orphans that are one object written more than once,
/// such as a copy the reader inlined through `xlink:href`, one is kept
/// ([`drop_copies`]). A feature whose parent key names no object or
/// several, names an object inside the feature itself, or names one whose
/// class cannot hold it, stays top-level and is reported. A synthetic key is
/// matched only within the feature's own document where that is known, as
/// two documents can share one.
///
/// The result lists each top-level feature in arrival order, each followed
/// by its orphans; a nested feature's orphans stay where the feature was.
pub fn join(features: Vec<FeatureObjects>, schema: &SchemaSet) -> Joined {
    let mut features = features;
    let mut reports = Reports::default();

    let copies = whole_copies(&features, schema);
    for (source, _) in copies.iter().enumerate().filter(|(_, copy)| **copy) {
        report(
            &mut reports,
            source,
            ErrorCode::CitygmlDuplicateObjectNotWritten,
        );
    }
    drop_copies(&mut features, &copies, &mut reports);

    let mut index: HashMap<String, Vec<(Location, bool)>> = HashMap::new();
    for (feature, objects) in features.iter().enumerate() {
        if copies[feature] {
            continue;
        }
        let tops = std::iter::once((None, &objects.root)).chain(
            objects
                .orphans
                .iter()
                .enumerate()
                .map(|(i, o)| (Some(i), o)),
        );
        for (orphan, object) in tops {
            let location = Location {
                feature,
                orphan,
                path: Vec::new(),
            };
            index_objects(object, location, &mut index);
        }
    }

    let mut attached: Vec<Option<Attachment>> = (0..features.len()).map(|_| None).collect();
    for child in 0..features.len() {
        let Some(parent_key) = features[child].parent_key.as_deref() else {
            continue;
        };
        if copies[child] {
            continue;
        }
        let mut matches: Vec<&Location> = Vec::new();
        let mut synthetic = false;
        for (location, is_synthetic) in index.get(parent_key).into_iter().flatten() {
            matches.push(location);
            synthetic |= is_synthetic;
        }
        if let Some(document) = features[child]
            .document_root
            .as_deref()
            .filter(|_| synthetic)
        {
            matches.retain(|location| {
                features[location.feature].document_root.as_deref() == Some(document)
            });
        }
        let [target] = matches.as_slice() else {
            report(&mut reports, child, ErrorCode::CitygmlParentNotFound);
            continue;
        };
        // A target in the feature's own tree or orphans, or in a feature
        // already nested in it, would hold the feature inside itself.
        if target.feature == child
            || (target.orphan.is_none() && nests_in(&attached, target.feature, child))
        {
            report(&mut reports, child, ErrorCode::CitygmlParentCycle);
            continue;
        }
        let parent = object_at(&features[target.feature], target);
        let Some((slot, property)) = containment_slot(
            schema,
            &parent.class.element,
            &features[child].root.class.element,
        ) else {
            report(&mut reports, child, ErrorCode::CitygmlParentCannotHold);
            continue;
        };
        attached[child] = Some(Attachment {
            target: (*target).clone(),
            slot,
            property,
        });
    }

    let nested: Vec<bool> = attached.iter().map(Option::is_some).collect();
    let mut held_by: HashMap<(usize, Option<usize>), Vec<usize>> = HashMap::new();
    for (child, attachment) in attached.iter().enumerate() {
        if let Some(Attachment { target, .. }) = attachment {
            held_by
                .entry((target.feature, target.orphan))
                .or_default()
                .push(child);
        }
    }
    let mut orphans = Vec::with_capacity(features.len());
    let mut assembly = Assembly {
        roots: Vec::with_capacity(features.len()),
        attached,
        held_by,
    };
    for feature in features {
        assembly.roots.push(Some(feature.root));
        orphans.push(feature.orphans);
    }

    let mut top = Vec::new();
    for (feature, feature_orphans) in orphans.into_iter().enumerate() {
        if copies[feature] {
            continue;
        }
        if !nested[feature] {
            if let Some(root) = assembly.roots[feature].take() {
                let mut roots = vec![feature];
                let object = assembly.fill(root, feature, None, &mut roots);
                roots.sort_unstable();
                top.push(TopLevel {
                    object,
                    orphan: false,
                    roots,
                });
            }
        }
        for (i, orphan) in feature_orphans.into_iter().enumerate() {
            let mut roots = Vec::new();
            let object = assembly.fill(orphan, feature, Some(i), &mut roots);
            roots.sort_unstable();
            top.push(TopLevel {
                object,
                orphan: true,
                roots,
            });
        }
    }
    Joined {
        top,
        copies,
        reports: reports.list,
    }
}

/// Where an object sits before the join.
#[derive(Debug, Clone)]
struct Location {
    feature: usize,
    /// `None` in the feature's own tree, else the index of its orphan.
    orphan: Option<usize>,
    /// The child indices leading down to the object from the tree's top.
    path: Vec<usize>,
}

/// A feature's place in the object its parent key names.
struct Attachment {
    target: Location,
    slot: usize,
    property: QName,
}

/// `(source, code)` pairs, each kept once, in the order found.
#[derive(Default)]
struct Reports {
    list: Vec<(usize, ErrorCode)>,
    seen: HashSet<(usize, ErrorCode)>,
}

fn report(reports: &mut Reports, source: usize, code: ErrorCode) {
    if reports.seen.insert((source, code)) {
        reports.list.push((source, code));
    }
}

/// Whether two objects are one object written twice, as when the reader
/// inlines an `xlink:href` target or one document is read twice: the same
/// source key, class and content (properties and geometry coordinates, at
/// every depth). A synthetic key names no source object, so it never
/// repeats one. Objects that share only a key, as built by hand, are two
/// objects, and the writer gives the second a generated id.
fn same_object(a: &ObjectNode, b: &ObjectNode) -> bool {
    source_key(a).is_some() && source_key(a) == source_key(b) && same_content(a, b)
}

/// The object's source `gml:id`, if it has one.
fn source_key(object: &ObjectNode) -> Option<&str> {
    object.key.as_deref().filter(|_| !object.synthetic)
}

/// Whether two objects have the same class, properties and geometry
/// coordinates, and hold the same objects: equal by content and source
/// `gml:id`, at every depth. An id-less object nested in a copy is equal
/// whatever its synthetic key, which differs between two reads of one
/// document.
fn same_content(a: &ObjectNode, b: &ObjectNode) -> bool {
    a.class.element == b.class.element
        && a.properties == b.properties
        && a.geometries.len() == b.geometries.len()
        && a.geometries.iter().zip(&b.geometries).all(|(a, b)| {
            a.lod == b.lod && a.property == b.property && same_element(&a.element, &b.element)
        })
        && a.children.len() == b.children.len()
        && a.children.iter().zip(&b.children).all(|(a, b)| {
            a.property == b.property
                && source_key(&a.object) == source_key(&b.object)
                && same_content(&a.object, &b.object)
        })
}

/// Whether two geometries have the same coordinates, whatever their ids and
/// appearance, which index each source's own palette.
fn same_element(a: &GmlElement, b: &GmlElement) -> bool {
    fn surfaces(a: &[GmlSurface], b: &[GmlSurface]) -> bool {
        a.len() == b.len()
            && a.iter()
                .zip(b)
                .all(|(a, b)| a.exterior == b.exterior && a.interiors == b.interiors)
    }
    fn solid(a: &GmlSolid, b: &GmlSolid) -> bool {
        surfaces(&a.exterior, &b.exterior)
            && a.interiors.len() == b.interiors.len()
            && a.interiors
                .iter()
                .zip(&b.interiors)
                .all(|(a, b)| surfaces(a, b))
    }
    match (a, b) {
        (GmlElement::Solid(a), GmlElement::Solid(b)) => solid(a, b),
        (GmlElement::MultiSolid { solids: a, .. }, GmlElement::MultiSolid { solids: b, .. }) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| solid(a, b))
        }
        (
            GmlElement::MultiSurface { surfaces: a, .. },
            GmlElement::MultiSurface { surfaces: b, .. },
        ) => surfaces(a, b),
        (GmlElement::MultiCurve { curves: a, .. }, GmlElement::MultiCurve { curves: b, .. }) => {
            a == b
        }
        (
            GmlElement::TriangulatedSurface { triangles: a },
            GmlElement::TriangulatedSurface { triangles: b },
        ) => surfaces(a, b),
        _ => false,
    }
}

/// Where a copy of an object sits, best kept first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Place {
    /// A feature's own object; for a whole feature, one naming no parent.
    Own,
    /// Nested in an object that can hold it.
    Nested,
    /// Nested in an orphan.
    InOrphan,
    /// Top-level only because its parent is missing or cannot hold it.
    Orphan,
}

/// How a copy ranks against the others, lower kept first: where it sits,
/// then whether it is the weaker copy at that place: made from geometry
/// alone, or held by a parent reached only through a reference property.
type CopyRank = (Place, bool);

/// Orders `copies` of one object best first: by rank, then arrival order,
/// which only breaks ties.
fn best_first(copies: &mut [usize], rank: impl Fn(usize) -> CopyRank) {
    copies.sort_by_key(|&copy| (rank(copy), copy));
}

/// Per feature, whether it is a copy of another feature, to be left out
/// whole. Features are one feature when their key and document repeat, as
/// when one document is read twice with `extractTags`, or when they are the
/// same object, orphans included, as when one is read twice without.
/// Features that share only a key may be different objects, so they stay.
///
/// Of each set of copies, the one kept is the one placed best
/// ([`FeaturePlaces`]), so a copy the reader extracted through a group's
/// `grp:groupMember` or `grp:parent` loses to the real object however the
/// two arrive.
#[cfg(feature = "new-geometry")]
fn whole_copies(features: &[FeatureObjects], schema: &SchemaSet) -> Vec<bool> {
    let same_feature = |a: &FeatureObjects, b: &FeatureObjects| {
        (a.document_root.is_some() && a.document_root == b.document_root)
            || (same_object(&a.root, &b.root)
                && a.orphans.len() == b.orphans.len()
                && a.orphans
                    .iter()
                    .zip(&b.orphans)
                    .all(|(a, b)| same_object(a, b)))
    };
    let mut sets: Vec<Vec<usize>> = Vec::new();
    let mut sets_by_key: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, feature) in features.iter().enumerate() {
        let Some(key) = feature.key.as_deref() else {
            continue;
        };
        let keyed = sets_by_key.entry(key).or_default();
        let found = keyed.iter().copied().find(|&set| {
            sets[set]
                .iter()
                .any(|&other| same_feature(&features[other], feature))
        });
        match found {
            Some(set) => sets[set].push(index),
            None => {
                keyed.push(sets.len());
                sets.push(vec![index]);
            }
        }
    }

    let mut places = FeaturePlaces::new(features, schema);
    let mut copies = vec![false; features.len()];
    for mut set in sets.into_iter().filter(|set| set.len() > 1) {
        let ranks: HashMap<usize, CopyRank> = set
            .iter()
            .map(|&feature| (feature, places.rank(feature)))
            .collect();
        best_first(&mut set, |feature| ranks[&feature]);
        for &copy in &set[1..] {
            copies[copy] = true;
        }
    }
    copies
}

/// The legacy writer joins nothing, so it writes every feature, as before.
#[cfg(not(feature = "new-geometry"))]
fn whole_copies(features: &[FeatureObjects], _schema: &SchemaSet) -> Vec<bool> {
    vec![false; features.len()]
}

/// Where each feature would be placed by its parent key, before copies are
/// left out, to rank copies of one feature.
#[cfg(feature = "new-geometry")]
struct FeaturePlaces<'a> {
    features: &'a [FeatureObjects],
    schema: &'a SchemaSet,
    /// Every keyed object: its feature, whether it sits in one of the
    /// feature's orphans, and its class element.
    objects: HashMap<&'a str, Vec<(usize, bool, &'a QName)>>,
    /// Per feature, whether it is placed soundly: it names no parent, or a
    /// parent that can hold it and is itself placed soundly. `None` until
    /// worked out.
    sound: Vec<Option<bool>>,
}

#[cfg(feature = "new-geometry")]
impl<'a> FeaturePlaces<'a> {
    fn new(features: &'a [FeatureObjects], schema: &'a SchemaSet) -> Self {
        fn walk<'a>(
            object: &'a ObjectNode,
            feature: usize,
            orphan: bool,
            out: &mut HashMap<&'a str, Vec<(usize, bool, &'a QName)>>,
        ) {
            if let Some(key) = object.key.as_deref() {
                out.entry(key)
                    .or_default()
                    .push((feature, orphan, &object.class.element));
            }
            for child in &object.children {
                walk(&child.object, feature, orphan, out);
            }
        }
        let mut objects = HashMap::new();
        for (feature, parts) in features.iter().enumerate() {
            walk(&parts.root, feature, false, &mut objects);
            for orphan in &parts.orphans {
                walk(orphan, feature, true, &mut objects);
            }
        }
        Self {
            features,
            schema,
            objects,
            sound: vec![None; features.len()],
        }
    }

    /// The objects that `feature`'s parent key names and whose class can
    /// hold it, outside the feature itself, as `(feature, in an orphan)`.
    /// Those in the feature's own document are taken first, as that is where
    /// the reader found the parent.
    fn parents(&self, feature: usize) -> Vec<(usize, bool)> {
        let child = &self.features[feature];
        let Some(parent_key) = child.parent_key.as_deref() else {
            return Vec::new();
        };
        let mut parents: Vec<(usize, bool)> = self
            .objects
            .get(parent_key)
            .into_iter()
            .flatten()
            .filter(|(holder, _, class)| {
                *holder != feature
                    && containment_slot(self.schema, class, &child.root.class.element).is_some()
            })
            .map(|&(holder, orphan, _)| (holder, orphan))
            .collect();
        if let Some(document) = child.document_root.as_deref() {
            let in_document = |&(holder, _): &(usize, bool)| {
                self.features[holder].document_root.as_deref() == Some(document)
            };
            if parents.iter().any(in_document) {
                parents.retain(in_document);
            }
        }
        parents
    }

    /// Whether `feature` names no parent, or one that can hold it, nested in
    /// a feature's own tree, that is placed soundly in turn. A cycle is not.
    fn is_sound(&mut self, feature: usize) -> bool {
        if let Some(sound) = self.sound[feature] {
            return sound;
        }
        if self.features[feature].parent_key.is_none() {
            self.sound[feature] = Some(true);
            return true;
        }
        // Guards a cycle of parent keys.
        self.sound[feature] = Some(false);
        let sound = self
            .parents(feature)
            .into_iter()
            .any(|(holder, orphan)| !orphan && self.is_sound(holder));
        self.sound[feature] = Some(sound);
        sound
    }

    /// `feature`'s rank among its copies: naming no parent, then held by a
    /// parent that can hold it, ahead of one whose parent is missing or
    /// cannot hold it. Among those held, one whose parent is itself placed
    /// soundly beats one whose parent is reached only through a reference,
    /// such as the building the reader extracts from a group's `grp:parent`,
    /// which names the group as its own parent.
    fn rank(&mut self, feature: usize) -> CopyRank {
        if self.features[feature].parent_key.is_none() {
            return (Place::Own, false);
        }
        let parents = self.parents(feature);
        if parents.is_empty() {
            return (Place::Orphan, true);
        }
        let sound = parents
            .into_iter()
            .any(|(holder, orphan)| !orphan && self.is_sound(holder));
        (Place::Nested, !sound)
    }
}

/// An object with a source key, which another object may be a copy of.
struct Candidate {
    location: Location,
    /// Lower is kept first: where the object sits (the feature's own object,
    /// nested in its tree, nested in an orphan, an orphan), then whether it
    /// has no data from the attribute tree.
    rank: CopyRank,
    /// The candidates it is nested in, outermost first.
    ancestors: Vec<usize>,
}

/// Leaves out every nested object or orphan that is [`same_object`] as
/// another, with everything nested in it, and reports it.
///
/// Which copy is kept does not depend on arrival order. A feature's own
/// object is never left out here. Otherwise a copy nested in the feature's
/// tree beats one nested in an orphan, which beats an orphan, and a copy
/// with data from the attribute tree beats one made from geometry alone.
/// Arrival order only breaks ties. Copies are settled outermost first, as
/// leaving out an object leaves out the copies nested in it.
fn drop_copies(features: &mut [FeatureObjects], left_out: &[bool], reports: &mut Reports) {
    let mut candidates = Vec::new();
    for (feature, objects) in features.iter().enumerate() {
        if left_out[feature] {
            continue;
        }
        let tops = std::iter::once((None, &objects.root)).chain(
            objects
                .orphans
                .iter()
                .enumerate()
                .map(|(i, o)| (Some(i), o)),
        );
        for (orphan, object) in tops {
            let location = Location {
                feature,
                orphan,
                path: Vec::new(),
            };
            collect_candidates(
                object,
                location,
                &objects.geometry_only,
                &mut Vec::new(),
                &mut candidates,
            );
        }
    }

    // Sets of two or more candidates that are one object, each in arrival
    // order, ordered by their first.
    let classes: Vec<Vec<usize>> = {
        let at = |id: usize| {
            let location: &Location = &candidates[id].location;
            object_at(&features[location.feature], location)
        };
        let mut by_key: HashMap<&str, Vec<usize>> = HashMap::new();
        for id in 0..candidates.len() {
            if let Some(key) = at(id).key.as_deref() {
                by_key.entry(key).or_default().push(id);
            }
        }
        let mut classes: Vec<Vec<usize>> = Vec::new();
        for ids in by_key.into_values().filter(|ids| ids.len() > 1) {
            let mut sets: Vec<Vec<usize>> = Vec::new();
            for id in ids {
                match sets.iter_mut().find(|set| same_object(at(set[0]), at(id))) {
                    Some(set) => set.push(id),
                    None => sets.push(vec![id]),
                }
            }
            classes.extend(sets.into_iter().filter(|set| set.len() > 1));
        }
        classes.sort_unstable_by_key(|set| set[0]);
        classes
    };
    if classes.is_empty() {
        return;
    }

    let mut class_of: Vec<Option<usize>> = vec![None; candidates.len()];
    for (class, members) in classes.iter().enumerate() {
        for &member in members {
            class_of[member] = Some(class);
        }
    }
    let mut alive = vec![true; candidates.len()];
    let mut settled = vec![false; classes.len()];
    let mut losers: Vec<usize> = Vec::new();
    loop {
        let open = |class: usize, alive: &[bool], settled: &[bool]| {
            !settled[class] && classes[class].iter().filter(|&&m| alive[m]).count() > 1
        };
        let open_classes: Vec<usize> = (0..classes.len())
            .filter(|&class| open(class, &alive, &settled))
            .collect();
        if open_classes.is_empty() {
            break;
        }
        // Outermost first: a class none of whose live members sits inside a
        // live member of a class still open.
        let mut ready: Vec<usize> = open_classes
            .iter()
            .copied()
            .filter(|&class| {
                classes[class].iter().all(|&member| {
                    !alive[member]
                        || candidates[member].ancestors.iter().all(|&ancestor| {
                            !alive[ancestor]
                                || class_of[ancestor]
                                    .is_none_or(|other| !open(other, &alive, &settled))
                        })
                })
            })
            .collect();
        // Copies nest alike, so some class always comes first; this only
        // guards the loop.
        if ready.is_empty() {
            ready.push(open_classes[0]);
        }
        for class in ready {
            if !open(class, &alive, &settled) {
                continue;
            }
            settled[class] = true;
            let mut members: Vec<usize> = classes[class]
                .iter()
                .copied()
                .filter(|&member| alive[member])
                .collect();
            best_first(&mut members, |member| candidates[member].rank);
            for &loser in &members[1..] {
                // A feature's own object is left out only as a whole copy.
                if candidates[loser].rank.0 == Place::Own {
                    continue;
                }
                alive[loser] = false;
                for (other, candidate) in candidates.iter().enumerate() {
                    if candidate.ancestors.contains(&loser) {
                        alive[other] = false;
                    }
                }
                losers.push(loser);
            }
        }
    }

    losers.sort_unstable();
    for &loser in &losers {
        let location = &candidates[loser].location;
        let source = object_at(&features[location.feature], location).source;
        report(reports, source, ErrorCode::CitygmlDuplicateObjectNotWritten);
    }
    // Last first, so the indices of those still to go hold.
    let mut locations: Vec<Location> = losers
        .into_iter()
        .map(|loser| candidates[loser].location.clone())
        .collect();
    locations.sort_unstable_by(|a, b| {
        (b.feature, b.orphan, &b.path).cmp(&(a.feature, a.orphan, &a.path))
    });
    for location in locations {
        let objects = &mut features[location.feature];
        match location.path.split_last() {
            None => {
                let orphan = location
                    .orphan
                    .expect("a feature's own object is never left out here");
                objects.orphans.remove(orphan);
            }
            Some((&last, path)) => {
                let mut object = match location.orphan {
                    None => &mut objects.root,
                    Some(i) => &mut objects.orphans[i],
                };
                for &i in path {
                    object = &mut object.children[i].object;
                }
                object.children.remove(last);
            }
        }
    }
}

/// Adds `object` and the objects nested in it, in pre-order, to `out`.
fn collect_candidates(
    object: &ObjectNode,
    location: Location,
    geometry_only: &HashSet<String>,
    ancestors: &mut Vec<usize>,
    out: &mut Vec<Candidate>,
) {
    let id = match &object.key {
        Some(key) if !object.synthetic => {
            let place = match (location.orphan, location.path.is_empty()) {
                (None, true) => Place::Own,
                (None, false) => Place::Nested,
                (Some(_), false) => Place::InOrphan,
                (Some(_), true) => Place::Orphan,
            };
            out.push(Candidate {
                location: location.clone(),
                rank: (place, geometry_only.contains(key)),
                ancestors: ancestors.clone(),
            });
            Some(out.len() - 1)
        }
        _ => None,
    };
    if let Some(id) = id {
        ancestors.push(id);
    }
    for (i, child) in object.children.iter().enumerate() {
        let mut path = location.path.clone();
        path.push(i);
        let child_location = Location {
            path,
            ..location.clone()
        };
        collect_candidates(&child.object, child_location, geometry_only, ancestors, out);
    }
    if id.is_some() {
        ancestors.pop();
    }
}

fn index_objects(
    object: &ObjectNode,
    location: Location,
    index: &mut HashMap<String, Vec<(Location, bool)>>,
) {
    for (i, child) in object.children.iter().enumerate() {
        let mut path = location.path.clone();
        path.push(i);
        let child_location = Location {
            path,
            ..location.clone()
        };
        index_objects(&child.object, child_location, index);
    }
    if let Some(key) = &object.key {
        index
            .entry(key.clone())
            .or_default()
            .push((location, object.synthetic));
    }
}

fn object_at<'a>(feature: &'a FeatureObjects, location: &Location) -> &'a ObjectNode {
    let mut object = match location.orphan {
        None => &feature.root,
        Some(i) => &feature.orphans[i],
    };
    for &i in &location.path {
        object = &object.children[i].object;
    }
    object
}

/// Whether `feature`'s own tree is `ancestor`'s, or is nested in it through
/// the attachments made so far, which never form a cycle.
fn nests_in(attached: &[Option<Attachment>], mut feature: usize, ancestor: usize) -> bool {
    loop {
        if feature == ancestor {
            return true;
        }
        match &attached[feature] {
            Some(Attachment { target, .. }) if target.orphan.is_none() => feature = target.feature,
            _ => return false,
        }
    }
}

/// The features' objects, taken as each is placed.
struct Assembly {
    roots: Vec<Option<ObjectNode>>,
    attached: Vec<Option<Attachment>>,
    /// The features nested in each feature's own tree, keyed `(feature,
    /// None)`, or in one of its orphans, in arrival order.
    held_by: HashMap<(usize, Option<usize>), Vec<usize>>,
}

impl Assembly {
    /// `object`, the top of `feature`'s own tree or of its orphan, with every
    /// feature nested in it in place. The features placed are added to `roots`.
    fn fill(
        &mut self,
        mut object: ObjectNode,
        feature: usize,
        orphan: Option<usize>,
        roots: &mut Vec<usize>,
    ) -> ObjectNode {
        for child in self.held_by.remove(&(feature, orphan)).unwrap_or_default() {
            let (Some(root), Some(attachment)) =
                (self.roots[child].take(), self.attached[child].take())
            else {
                continue;
            };
            roots.push(child);
            let filled = self.fill(root, child, None, roots);
            // Placed objects are appended, so the path's indices still hold.
            let mut parent = &mut object;
            for &i in &attachment.target.path {
                parent = &mut parent.children[i].object;
            }
            parent.children.push(NestedObject {
                slot: attachment.slot,
                property: attachment.property,
                object: filled,
            });
        }
        object
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use reearth_flow_citygml::schema::SchemaSet;
    use reearth_flow_diagnostics::ErrorCode;
    use reearth_flow_types::{AttributeValue, Attributes};
    use serde_json::json;

    use super::super::classes::{self, ClassResolver, Prefixes};
    use super::super::model::{EntryOwner, GeometryEntry, GmlElement, GmlSurface, ObjectNode};
    use super::super::placement;
    use super::super::properties::{self, XmlProperty};
    use super::*;

    const URO: &str = "https://www.geospatial.jp/iur/uro/3.1";

    fn schema() -> &'static SchemaSet {
        SchemaSet::core().unwrap()
    }

    fn entry(key: Option<&str>, ty: Option<&str>, parent: Option<&str>, lod: u8) -> GeometryEntry {
        GeometryEntry {
            lod,
            property: None,
            element: GmlElement::MultiSurface {
                id: None,
                surfaces: Vec::new(),
            },
            owner: EntryOwner {
                key: key.map(str::to_owned),
                synthetic: false,
                feature_type: ty.map(str::to_owned),
                parent_key: parent.map(str::to_owned),
            },
        }
    }

    /// A member owned by `key`, of class `ty`, nested under `parent`, holding
    /// one triangle at height `z`.
    fn member_at(key: &str, ty: &str, parent: &str, z: f64) -> GeometryEntry {
        let mut entry = member(key, ty, parent, 2);
        entry.element = GmlElement::MultiSurface {
            id: None,
            surfaces: vec![GmlSurface {
                id: None,
                exterior: vec![[0.0, 0.0, z], [1.0, 0.0, z], [0.0, 1.0, z], [0.0, 0.0, z]],
                interiors: Vec::new(),
                material_idx: None,
                texture_idx: None,
                uv_exterior: Vec::new(),
                uv_interiors: Vec::new(),
            }],
        };
        entry
    }

    /// A member owned by `key`, of class `ty`, nested under `parent`.
    fn member(key: &str, ty: &str, parent: &str, lod: u8) -> GeometryEntry {
        entry(Some(key), Some(ty), Some(parent), lod)
    }

    fn synthetic_member(key: &str, ty: &str, parent: &str) -> GeometryEntry {
        let mut entry = member(key, ty, parent, 2);
        entry.owner.synthetic = true;
        entry
    }

    /// The feature's own geometry.
    fn own() -> GeometryEntry {
        entry(Some("b1"), Some("bldg:Building"), None, 1)
    }

    fn object(
        prefix: &str,
        local: &str,
        id: Option<&str>,
        children: Vec<XmlProperty>,
    ) -> XmlProperty {
        let mut element = XmlProperty::new(prefix, local);
        if let Some(id) = id {
            element.attrs.push(("gml:id".to_owned(), id.to_owned()));
        }
        element.children = children;
        element
    }

    fn wrap(prefix: &str, local: &str, child: XmlProperty) -> XmlProperty {
        let mut wrapper = XmlProperty::new(prefix, local);
        wrapper.children.push(child);
        wrapper
    }

    fn name(text: &str) -> XmlProperty {
        let mut name = XmlProperty::new("gml", "name");
        name.text = Some(text.to_owned());
        name
    }

    /// `bldg:boundedBy` around a `bldg:WallSurface`.
    fn wall(id: Option<&str>, children: Vec<XmlProperty>) -> XmlProperty {
        wrap(
            "bldg",
            "boundedBy",
            object("bldg", "WallSurface", id, children),
        )
    }

    /// `bldg:opening` around a `bldg:Window`.
    fn window(id: Option<&str>, children: Vec<XmlProperty>) -> XmlProperty {
        wrap("bldg", "opening", object("bldg", "Window", id, children))
    }

    fn parts_with(
        prefixes: &Prefixes,
        entries: Vec<GeometryEntry>,
        tree: Vec<XmlProperty>,
    ) -> FeatureParts {
        let (remaining, tree) = split_nested_objects(schema(), prefixes, tree);
        assert!(remaining.is_empty(), "{remaining:?}");
        FeatureParts {
            source: 0,
            class: classes::resolve("bldg:Building", prefixes, schema()).unwrap(),
            gml_id: Some("b1".to_owned()),
            parent_key: None,
            document_root: None,
            geometries: entries,
            properties: Vec::new(),
            tree,
            referenced: HashSet::new(),
        }
    }

    /// `bldg:Building` `b1` with these members and attribute tree.
    fn building(entries: Vec<GeometryEntry>, tree: Vec<XmlProperty>) -> FeatureParts {
        parts_with(&Prefixes::citygml2(), entries, tree)
    }

    fn build_with(prefixes: &Prefixes, parts: FeatureParts) -> FeatureObjects {
        let mut classes = ClassResolver::new(prefixes, schema());
        build_feature_objects(parts, schema(), prefixes, &mut classes)
    }

    fn build(parts: FeatureParts) -> FeatureObjects {
        build_with(&Prefixes::citygml2(), parts)
    }

    fn child_ids(node: &ObjectNode) -> Vec<Option<&str>> {
        node.children
            .iter()
            .map(|child| child.object.gml_id.as_deref())
            .collect()
    }

    fn property_names(node: &ObjectNode) -> Vec<String> {
        node.properties
            .iter()
            .map(|placed| format!("{}:{}", placed.property.prefix, placed.property.local))
            .collect()
    }

    fn codes(codes: &[ErrorCode]) -> HashSet<ErrorCode> {
        codes.iter().copied().collect()
    }

    fn bldg(local: &str) -> QName {
        QName::new("http://www.opengis.net/citygml/building/2.0", local)
    }

    #[test]
    fn a_boundary_surface_is_split_out_of_its_wrapper_with_its_data() {
        let (remaining, tree) = split_nested_objects(
            schema(),
            &Prefixes::citygml2(),
            vec![wall(Some("w1"), vec![name("N")])],
        );
        assert!(remaining.is_empty());
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].element.local, "WallSurface");
        assert_eq!(tree[0].element.attr("gml:id"), Some("w1"));
        assert_eq!(tree[0].element.children, vec![name("N")]);
        assert!(tree[0].children.is_empty());
    }

    #[test]
    fn an_opening_is_split_into_its_walls_children() {
        let (_, tree) = split_nested_objects(
            schema(),
            &Prefixes::citygml2(),
            vec![wall(
                Some("w1"),
                vec![name("N"), window(Some("win1"), vec![name("W")])],
            )],
        );
        let wall = &tree[0];
        assert_eq!(wall.element.children, vec![name("N")]);
        assert_eq!(wall.children.len(), 1);
        assert_eq!(wall.children[0].element.local, "Window");
        assert_eq!(wall.children[0].element.attr("gml:id"), Some("win1"));
        assert_eq!(wall.children[0].element.children, vec![name("W")]);
    }

    #[test]
    fn a_reference_property_and_a_non_city_object_stay_for_placement() {
        let generalizes = wrap(
            "core",
            "generalizesTo",
            object("bldg", "Building", Some("x"), Vec::new()),
        );
        let address = wrap(
            "bldg",
            "address",
            object("core", "Address", None, Vec::new()),
        );
        let (remaining, tree) = split_nested_objects(
            schema(),
            &Prefixes::citygml2(),
            vec![generalizes.clone(), address.clone()],
        );
        assert_eq!(remaining, vec![generalizes, address]);
        assert!(tree.is_empty());
    }

    #[test]
    fn containment_never_uses_a_reference_property() {
        assert_eq!(
            containment_slot(schema(), &bldg("Building"), &bldg("WallSurface"))
                .map(|(_, property)| property),
            Some(bldg("boundedBy"))
        );
        assert_eq!(
            containment_slot(schema(), &bldg("Building"), &bldg("Window")),
            None
        );
    }

    #[test]
    fn members_are_grouped_by_owner_into_nested_objects() {
        let objects = build(building(
            vec![
                own(),
                member("w1", "bldg:WallSurface", "b1", 2),
                member("w2", "bldg:WallSurface", "b1", 2),
                member("w1", "bldg:WallSurface", "b1", 2),
            ],
            Vec::new(),
        ));
        let root = &objects.root;
        assert_eq!(root.geometries.len(), 1);
        assert_eq!(child_ids(root), [Some("w1"), Some("w2")]);
        for child in &root.children {
            assert_eq!(child.property, bldg("boundedBy"));
            assert_eq!(child.object.class.element, bldg("WallSurface"));
            assert_eq!(child.object.source, 0);
        }
        assert_eq!(root.children[0].object.geometries.len(), 2);
        assert_eq!(root.children[0].object.key.as_deref(), Some("w1"));
        assert!(!root.children[0].object.synthetic);
        assert!(objects.orphans.is_empty());
        assert_eq!(objects.key.as_deref(), Some("b1"));
        assert_eq!(root.key.as_deref(), Some("b1"));
    }

    #[test]
    fn an_owner_parent_key_nests_a_window_in_its_wall() {
        let objects = build(building(
            vec![
                member("w1", "bldg:WallSurface", "b1", 2),
                member("win1", "bldg:Window", "w1", 3),
            ],
            Vec::new(),
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.children.len(), 1);
        let window = &wall.children[0];
        assert_eq!(window.property, bldg("opening"));
        assert_eq!(window.object.class.element, bldg("Window"));
        assert_eq!(window.object.gml_id.as_deref(), Some("win1"));
        assert!(objects.orphans.is_empty());
    }

    #[test]
    fn an_owner_key_equal_to_the_features_id_is_the_feature_itself() {
        let objects = build(building(
            vec![own(), member("b1", "bldg:Building", "x", 2)],
            Vec::new(),
        ));
        assert_eq!(objects.root.geometries.len(), 2);
        assert!(objects.root.children.is_empty());
        assert!(objects.orphans.is_empty());
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn a_synthetic_key_groups_but_never_becomes_the_id() {
        let objects = build(building(
            vec![
                synthetic_member("_synid1", "bldg:WallSurface", "b1"),
                synthetic_member("_synid1", "bldg:WallSurface", "b1"),
            ],
            Vec::new(),
        ));
        let root = &objects.root;
        assert_eq!(root.children.len(), 1);
        let wall = &root.children[0].object;
        assert_eq!(wall.geometries.len(), 2);
        assert_eq!(wall.gml_id, None);
        assert_eq!(wall.key.as_deref(), Some("_synid1"));
        assert!(wall.synthetic);
    }

    #[test]
    fn an_owner_key_that_is_not_a_valid_xml_name_is_not_kept() {
        let objects = build(building(
            vec![
                member("1abc", "bldg:WallSurface", "b1", 2),
                member("a b", "bldg:WallSurface", "b1", 2),
            ],
            Vec::new(),
        ));
        assert_eq!(child_ids(&objects.root), [None, None]);
        let keys: Vec<_> = objects
            .root
            .children
            .iter()
            .map(|child| child.object.key.as_deref())
            .collect();
        assert_eq!(keys, [Some("1abc"), Some("a b")]);
    }

    #[test]
    fn a_tree_object_without_geometry_is_kept_with_its_data() {
        let objects = build(building(
            vec![own()],
            vec![wall(Some("w1"), vec![name("N")])],
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.gml_id.as_deref(), Some("w1"));
        assert_eq!(property_names(wall), ["gml:name"]);
        assert!(wall.geometries.is_empty());
        assert!(!wall.has_geometry());
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn a_tree_object_loses_its_xml_attributes_to_the_written_element() {
        let objects = build(building(
            vec![member("w1", "bldg:WallSurface", "b1", 2)],
            vec![wall(Some("w1"), vec![name("N")])],
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.geometries.len(), 1);
        assert_eq!(wall.properties[0].property, name("N"));
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn a_group_missing_from_the_tree_keeps_type_geometry_and_id() {
        let objects = build(building(
            vec![member("w2", "bldg:WallSurface", "b1", 2)],
            vec![wall(Some("w1"), vec![name("N")])],
        ));
        let root = &objects.root;
        assert_eq!(child_ids(root), [Some("w1"), Some("w2")]);
        let w1 = &root.children[0].object;
        assert_eq!(property_names(w1), ["gml:name"]);
        let w2 = &root.children[1].object;
        assert_eq!(w2.class.element, bldg("WallSurface"));
        assert_eq!(w2.geometries.len(), 1);
        assert!(w2.properties.is_empty());
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn the_tree_supplies_a_parent_that_owns_no_geometry() {
        let objects = build(building(
            vec![member("win1", "bldg:Window", "w1", 3)],
            vec![wall(Some("w1"), Vec::new())],
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.gml_id.as_deref(), Some("w1"));
        assert!(wall.geometries.is_empty());
        assert_eq!(child_ids(wall), [Some("win1")]);
        assert_eq!(wall.children[0].property, bldg("opening"));
        assert!(objects.root.has_geometry());
    }

    #[test]
    fn a_non_city_object_owner_folds_into_its_nearest_city_object() {
        let objects = build(building(
            vec![member("dm1", "uro:DmGeometricAttribute", "w1", 2)],
            vec![wall(Some("w1"), Vec::new())],
        ));
        let root = &objects.root;
        assert_eq!(child_ids(root), [Some("w1")]);
        assert_eq!(root.children[0].object.geometries.len(), 1);
        assert!(root.children[0].object.children.is_empty());
        assert!(objects.orphans.is_empty());
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn a_folded_owner_hands_its_children_to_its_parent() {
        // An ADE owner whose ADE is not configured, holding a window, inside
        // wall w1.
        let objects = build(building(
            vec![
                member("x1", "uro:DmGeometricAttribute", "w1", 2),
                member("win1", "bldg:Window", "x1", 3),
            ],
            vec![wall(Some("w1"), Vec::new())],
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.geometries.len(), 1);
        assert_eq!(child_ids(wall), [Some("win1")]);
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn a_misspelt_owner_type_is_reported_and_its_geometry_kept_on_the_parent() {
        let objects = build(building(
            vec![member("s1", "bldg:WallSurfce", "w1", 2)],
            vec![wall(Some("w1"), Vec::new())],
        ));
        let root = &objects.root;
        assert_eq!(child_ids(root), [Some("w1")]);
        assert_eq!(root.children[0].object.geometries.len(), 1);
        assert!(root.children[0].object.children.is_empty());
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlUnknownFeatureType])
        );
    }

    #[test]
    fn an_owner_with_a_source_id_and_no_type_is_reported() {
        let objects = build(building(
            vec![entry(Some("x1"), None, Some("w1"), 2)],
            vec![wall(Some("w1"), Vec::new())],
        ));
        assert_eq!(objects.root.children[0].object.geometries.len(), 1);
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlUnknownFeatureType])
        );
    }

    #[test]
    fn owners_folded_by_design_are_not_reported() {
        let mut no_type = entry(Some("_synid1"), None, Some("w1"), 2);
        no_type.owner.synthetic = true;
        for owner in [
            no_type,
            member("a1", "core:Address", "w1", 2),
            member("dm1", "uro:DmGeometricAttribute", "w1", 2),
        ] {
            let objects = build(building(vec![owner], vec![wall(Some("w1"), Vec::new())]));
            assert_eq!(objects.root.children[0].object.geometries.len(), 1);
            assert!(objects.reports.is_empty(), "{:?}", objects.reports);
        }
    }

    #[test]
    fn a_tree_object_of_no_class_is_reported_when_its_data_is_lost() {
        let abstract_object = |children| {
            wrap(
                "bldg",
                "boundedBy",
                object("core", "_CityObject", Some("c1"), children),
            )
        };
        let objects = build(building(
            vec![member("c1", "bldg:WallSurface", "b1", 2)],
            vec![abstract_object(vec![name("N")])],
        ));
        assert!(objects.root.children.is_empty());
        assert_eq!(objects.root.geometries.len(), 1);
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedObjectNotWritten])
        );

        let objects = build(building(Vec::new(), vec![abstract_object(Vec::new())]));
        assert!(objects.root.children.is_empty());
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn a_member_with_only_a_parent_key_lands_on_that_parent() {
        let objects = build(building(
            vec![
                entry(None, None, Some("w1"), 2),
                entry(None, None, None, 1),
                entry(None, None, Some("nope"), 1),
            ],
            vec![wall(Some("w1"), Vec::new())],
        ));
        assert_eq!(objects.root.geometries.len(), 2);
        assert_eq!(objects.root.children[0].object.geometries.len(), 1);
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn an_unknown_parent_key_makes_a_reported_orphan() {
        let objects = build(building(
            vec![
                member("w1", "bldg:WallSurface", "nope", 2),
                member("win1", "bldg:Window", "w1", 3),
            ],
            Vec::new(),
        ));
        assert!(objects.root.children.is_empty());
        assert_eq!(objects.orphans.len(), 1);
        let orphan = &objects.orphans[0];
        assert_eq!(orphan.gml_id.as_deref(), Some("w1"));
        assert_eq!(child_ids(orphan), [Some("win1")]);
        assert_eq!(objects.reports, codes(&[ErrorCode::CitygmlParentNotFound]));
    }

    #[test]
    fn a_window_under_the_building_is_a_reported_orphan() {
        let objects = build(building(
            vec![own(), member("win1", "bldg:Window", "b1", 3)],
            Vec::new(),
        ));
        assert!(objects.root.children.is_empty());
        assert_eq!(objects.orphans.len(), 1);
        assert_eq!(objects.orphans[0].class.element, bldg("Window"));
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlParentCannotHold])
        );
    }

    #[test]
    fn an_owner_parent_cycle_is_broken_and_reported() {
        let objects = build(building(
            vec![
                member("p1", "bldg:BuildingPart", "p2", 2),
                member("p2", "bldg:BuildingPart", "p1", 2),
            ],
            Vec::new(),
        ));
        assert!(objects.root.children.is_empty());
        assert_eq!(objects.orphans.len(), 1);
        let orphan = &objects.orphans[0];
        assert_eq!(orphan.gml_id.as_deref(), Some("p1"));
        assert_eq!(child_ids(orphan), [Some("p2")]);
        assert_eq!(orphan.children[0].property, bldg("consistsOfBuildingPart"));
        assert_eq!(objects.reports, codes(&[ErrorCode::CitygmlParentCycle]));
    }

    #[test]
    fn without_tree_ids_nested_data_is_not_matched_or_duplicated() {
        let objects = build(building(
            vec![member("w1", "bldg:WallSurface", "b1", 2)],
            vec![wall(None, vec![name("N")])],
        ));
        let root = &objects.root;
        assert_eq!(child_ids(root), [Some("w1")]);
        assert!(root.children[0].object.properties.is_empty());
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
    }

    #[test]
    fn a_dropped_objects_keyed_descendant_moves_to_the_nearest_kept_ancestor() {
        // A key-less wall holding window win1, with keys missing elsewhere.
        let objects = build(building(
            vec![member("w2", "bldg:WallSurface", "b1", 2)],
            vec![wall(None, vec![window(Some("win1"), vec![name("W")])])],
        ));
        let root = &objects.root;
        assert_eq!(child_ids(root), [Some("w2")]);
        assert_eq!(objects.orphans.len(), 1);
        assert_eq!(objects.orphans[0].gml_id.as_deref(), Some("win1"));
        assert_eq!(
            objects.reports,
            codes(&[
                ErrorCode::CitygmlNestedDataNotMatched,
                ErrorCode::CitygmlParentCannotHold
            ])
        );
    }

    #[test]
    fn a_key_less_data_only_object_is_kept_when_every_group_matched() {
        let objects = build(building(
            vec![member("w1", "bldg:WallSurface", "b1", 2)],
            vec![wall(Some("w1"), Vec::new()), wall(None, vec![name("N")])],
        ));
        let root = &objects.root;
        assert_eq!(child_ids(root), [Some("w1"), None]);
        assert_eq!(root.children[0].object.geometries.len(), 1);
        assert_eq!(property_names(&root.children[1].object), ["gml:name"]);
        assert_eq!(root.children[1].object.key, None);
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn nested_properties_are_placed_in_the_nested_classs_model() {
        let prefixes = Prefixes::with_ade(&[("uro".to_owned(), URO.to_owned())]);
        let mut ade = XmlProperty::new("uro", "buildingIDAttribute");
        ade.text = Some("x".to_owned());
        let objects = build_with(
            &prefixes,
            parts_with(
                &prefixes,
                vec![member("w1", "bldg:WallSurface", "b1", 2)],
                vec![wall(Some("w1"), vec![ade, name("N")])],
            ),
        );
        let wall = &objects.root.children[0].object;
        assert_eq!(property_names(wall), ["gml:name"]);
        assert!(objects
            .reports
            .contains(&ErrorCode::CitygmlAttributeNotPlaced));
    }

    #[test]
    fn an_unambiguous_bare_owner_type_resolves() {
        let objects = build(building(vec![member("r1", "Room", "b1", 4)], Vec::new()));
        let room = &objects.root.children[0];
        assert_eq!(room.property, bldg("interiorRoom"));
        assert_eq!(room.object.class.element, bldg("Room"));
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn an_ambiguous_bare_owner_type_is_reported_and_folds() {
        let objects = build(building(
            vec![member("s1", "WallSurface", "b1", 2)],
            Vec::new(),
        ));
        assert_eq!(objects.root.geometries.len(), 1);
        assert!(objects.root.children.is_empty());
        assert!(objects.orphans.is_empty());
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlUnknownFeatureType])
        );
    }

    fn attrs(value: serde_json::Value) -> Attributes {
        match AttributeValue::from(value) {
            AttributeValue::Map(map) => map,
            other => panic!("expected a map, got {other:?}"),
        }
    }

    /// Reads the attribute tree as the writer does, splits it and builds.
    fn read_and_build(
        tree: serde_json::Value,
        entries: Vec<GeometryEntry>,
    ) -> (FeatureObjects, Vec<properties::Skip>) {
        let prefixes = Prefixes::citygml2();
        let converted =
            properties::from_attributes(&attrs(tree), &HashSet::new(), &prefixes, &|_, local| {
                properties::has_geometry_name(local)
            });
        let (remaining, tree) = split_nested_objects(schema(), &prefixes, converted.properties);
        let mut parts = building(entries, Vec::new());
        let (placed, skipped) = placement::place(schema(), &prefixes, &bldg("Building"), remaining);
        parts.properties = placed;
        parts.tree = tree;
        let mut skips = converted.skipped;
        skips.extend(skipped);
        (build(parts), skips)
    }

    #[test]
    fn a_data_less_object_read_with_its_id_matches_its_geometry() {
        let (objects, skipped) = read_and_build(
            json!({ "bldg:boundedBy": { "bldg:WallSurface": { "@gml:id": "w1" } } }),
            vec![member("w1", "bldg:WallSurface", "b1", 2)],
        );
        assert!(skipped.is_empty(), "{skipped:?}");
        assert_eq!(child_ids(&objects.root), [Some("w1")]);
        assert_eq!(objects.root.children[0].object.geometries.len(), 1);
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn an_id_less_object_read_with_only_its_synthetic_key_matches_its_geometry() {
        let (objects, skipped) = read_and_build(
            json!({ "bldg:boundedBy": { "bldg:WallSurface": {
                "__citygml_synthetic_owner_key": "a.gml#_synid1"
            } } }),
            vec![synthetic_member("a.gml#_synid1", "bldg:WallSurface", "b1")],
        );
        assert!(skipped.is_empty(), "{skipped:?}");
        let wall = &objects.root.children[0].object;
        assert_eq!(objects.root.children.len(), 1);
        assert_eq!(wall.geometries.len(), 1);
        assert_eq!(wall.gml_id, None);
        assert!(wall.synthetic);
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn the_synthetic_tree_key_is_read_never_reported_and_never_an_attribute() {
        let (objects, skipped) = read_and_build(
            json!({ "bldg:boundedBy": { "bldg:WallSurface": {
                "__citygml_synthetic_owner_key": "a.gml#_synid1",
                "gml:name": "N"
            } } }),
            vec![synthetic_member("a.gml#_synid1", "bldg:WallSurface", "b1")],
        );
        assert!(skipped.is_empty(), "{skipped:?}");
        let wall = &objects.root.children[0].object;
        assert_eq!(property_names(wall), ["gml:name"]);
        assert!(wall.properties[0].property.attrs.is_empty());
        assert_eq!(wall.geometries.len(), 1);
        assert!(objects.reports.is_empty());
    }

    /// Feature `source`: a `ty` keyed `id`, with geometry of its own and
    /// `entries` and `tree` besides, nesting under `parent`.
    fn feature_with(
        source: usize,
        ty: &str,
        id: &str,
        parent: Option<&str>,
        entries: Vec<GeometryEntry>,
        tree: Vec<XmlProperty>,
    ) -> FeatureObjects {
        let prefixes = Prefixes::citygml2();
        let (remaining, tree) = split_nested_objects(schema(), &prefixes, tree);
        assert!(remaining.is_empty(), "{remaining:?}");
        let mut geometries = vec![entry(Some(id), Some(ty), None, 2)];
        geometries.extend(entries);
        build(FeatureParts {
            source,
            class: classes::resolve(ty, &prefixes, schema()).unwrap(),
            gml_id: Some(id.to_owned()),
            parent_key: parent.map(str::to_owned),
            document_root: None,
            geometries,
            properties: Vec::new(),
            tree,
            referenced: HashSet::new(),
        })
    }

    fn feature(source: usize, ty: &str, id: &str, parent: Option<&str>) -> FeatureObjects {
        feature_with(source, ty, id, parent, Vec::new(), Vec::new())
    }

    /// `key[property:child,...]`, with the property's local name.
    fn shape(node: &ObjectNode) -> String {
        let children: Vec<String> = node
            .children
            .iter()
            .map(|child| format!("{}:{}", child.property.local, shape(&child.object)))
            .collect();
        format!(
            "{}[{}]",
            node.key.as_deref().unwrap_or("-"),
            children.join(",")
        )
    }

    fn top_shapes(joined: &Joined) -> Vec<String> {
        joined
            .top
            .iter()
            .map(|top| {
                let orphan = if top.orphan { "orphan " } else { "" };
                format!("{orphan}{}", shape(&top.object))
            })
            .collect()
    }

    fn join_all(features: Vec<FeatureObjects>) -> Joined {
        join(features, schema())
    }

    #[test]
    fn a_feature_naming_another_as_parent_is_nested_in_it() {
        let joined = join_all(vec![
            feature(0, "bldg:Building", "b1", None),
            feature(1, "bldg:WallSurface", "w1", Some("b1")),
            feature(2, "bldg:WallSurface", "w2", Some("b1")),
        ]);
        assert_eq!(joined.top.len(), 1);
        let b1 = &joined.top[0];
        assert!(!b1.orphan);
        assert_eq!(child_ids(&b1.object), [Some("w1"), Some("w2")]);
        assert_eq!(b1.object.children[0].property, bldg("boundedBy"));
        assert_eq!(b1.object.children[1].object.source, 2);
        assert_eq!(b1.roots, [0, 1, 2]);
        assert!(joined.reports.is_empty());
    }

    #[test]
    fn arrival_order_does_not_change_the_result() {
        let parent_first = join_all(vec![
            feature(0, "bldg:Building", "b1", None),
            feature(1, "bldg:WallSurface", "w1", Some("b1")),
            feature(2, "bldg:WallSurface", "w2", Some("b1")),
        ]);
        let children_first = join_all(vec![
            feature(0, "bldg:WallSurface", "w1", Some("b1")),
            feature(1, "bldg:WallSurface", "w2", Some("b1")),
            feature(2, "bldg:Building", "b1", None),
        ]);
        assert_eq!(
            top_shapes(&parent_first),
            ["b1[boundedBy:w1[],boundedBy:w2[]]"]
        );
        assert_eq!(top_shapes(&children_first), top_shapes(&parent_first));
        assert!(children_first.reports.is_empty());
    }

    #[test]
    fn a_feature_nests_under_an_object_inside_another_feature() {
        let joined = join_all(vec![
            feature_with(
                0,
                "bldg:Building",
                "b1",
                None,
                vec![member("WS1", "bldg:WallSurface", "b1", 2)],
                Vec::new(),
            ),
            feature(1, "bldg:Window", "win1", Some("WS1")),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[boundedBy:WS1[opening:win1[]]]"]);
        let wall = &joined.top[0].object.children[0].object;
        assert_eq!(wall.children[0].property, bldg("opening"));
        assert_eq!(wall.children[0].object.source, 1);
        assert!(joined.reports.is_empty());
    }

    #[test]
    fn a_chain_of_three_features_nests_at_every_depth() {
        let joined = join_all(vec![
            feature(0, "bldg:Window", "win1", Some("w1")),
            feature(1, "bldg:WallSurface", "w1", Some("b1")),
            feature(2, "bldg:Building", "b1", None),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[boundedBy:w1[opening:win1[]]]"]);
        assert_eq!(joined.top[0].roots, [0, 1, 2]);
        assert!(joined.reports.is_empty());
    }

    #[test]
    fn a_parent_key_matching_nothing_is_reported_and_left_top_level() {
        let joined = join_all(vec![
            feature(0, "bldg:Building", "b1", None),
            feature(1, "bldg:WallSurface", "w1", Some("b9")),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[]", "w1[]"]);
        assert_eq!(joined.reports, [(1, ErrorCode::CitygmlParentNotFound)]);
    }

    #[test]
    fn a_parent_key_matching_two_roots_is_ambiguous() {
        // Roots sharing a key but no document, as built by hand, or from two
        // different documents, may be two objects, as they are here: each
        // has geometry of its own.
        let own = |z| vec![member_at("b1", "bldg:Building", "b1", z)];
        for documents in [[None, None], [Some("d1"), Some("d2")]] {
            let mut features = vec![
                feature_with(0, "bldg:Building", "b1", None, own(0.0), Vec::new()),
                feature_with(1, "bldg:Building", "b1", None, own(1.0), Vec::new()),
                feature(2, "bldg:WallSurface", "w1", Some("b1")),
            ];
            for (feature, document) in features.iter_mut().zip(documents) {
                feature.document_root = document.map(str::to_owned);
            }
            let joined = join_all(features);
            assert_eq!(
                top_shapes(&joined),
                ["b1[]", "b1[]", "w1[]"],
                "{documents:?}"
            );
            assert_eq!(
                joined.reports,
                [(2, ErrorCode::CitygmlParentNotFound)],
                "{documents:?}"
            );
        }
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn a_second_copy_of_a_feature_from_its_document_is_left_out_whole() {
        let mut features = vec![
            feature(0, "bldg:Building", "b1", None),
            feature_with(
                1,
                "bldg:Building",
                "b1",
                None,
                vec![member("p9", "bldg:BuildingPart", "b1", 2)],
                Vec::new(),
            ),
            feature(2, "bldg:WallSurface", "w1", Some("b1")),
        ];
        for feature in &mut features {
            feature.document_root = Some("b1".to_owned());
        }
        let joined = join_all(features);
        assert_eq!(top_shapes(&joined), ["b1[boundedBy:w1[]]"]);
        assert_eq!(joined.top[0].roots, [0, 2]);
        assert_eq!(joined.copies, [false, true, false]);
        assert_eq!(
            joined.reports,
            [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[test]
    fn a_feature_naming_its_own_orphan_is_a_cycle() {
        let joined = join_all(vec![feature_with(
            0,
            "bldg:WallSurface",
            "w1",
            Some("win9"),
            vec![member("win9", "bldg:Window", "nowhere", 3)],
            Vec::new(),
        )]);
        assert_eq!(top_shapes(&joined), ["w1[]", "orphan win9[]"]);
        assert_eq!(joined.reports, [(0, ErrorCode::CitygmlParentCycle)]);
    }

    #[test]
    fn only_a_property_that_nests_a_city_object_holds_objects() {
        let mut containment = ContainmentProperties::default();
        let core = |local: &str| QName::new(CORE, local);
        for (parent, property, holds) in [
            (bldg("WallSurface"), bldg("opening"), true),
            (bldg("Building"), bldg("boundedBy"), true),
            (bldg("Building"), bldg("consistsOfBuildingPart"), true),
            (bldg("Building"), bldg("measuredHeight"), false),
            (bldg("Building"), bldg("opening"), false),
            (bldg("Building"), core("generalizesTo"), false),
        ] {
            assert_eq!(
                containment.holds_objects(schema(), &parent, &property),
                holds,
                "{parent:?} {property:?}"
            );
        }
    }

    #[test]
    fn a_parent_whose_class_cannot_hold_the_feature_is_reported() {
        let joined = join_all(vec![
            feature(0, "bldg:Building", "b1", None),
            feature(1, "bldg:Window", "win1", Some("b1")),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[]", "win1[]"]);
        assert_eq!(joined.reports, [(1, ErrorCode::CitygmlParentCannotHold)]);
    }

    #[test]
    fn a_cycle_between_features_is_broken_and_reported() {
        let joined = join_all(vec![
            feature(0, "bldg:BuildingPart", "p1", Some("p2")),
            feature(1, "bldg:BuildingPart", "p2", Some("p1")),
        ]);
        assert_eq!(top_shapes(&joined), ["p2[consistsOfBuildingPart:p1[]]"]);
        assert_eq!(joined.reports, [(1, ErrorCode::CitygmlParentCycle)]);
    }

    #[test]
    fn a_feature_naming_an_object_of_its_own_is_a_cycle() {
        let joined = join_all(vec![feature_with(
            0,
            "bldg:Building",
            "b1",
            Some("p1"),
            vec![member("p1", "bldg:BuildingPart", "b1", 2)],
            Vec::new(),
        )]);
        assert_eq!(top_shapes(&joined), ["b1[consistsOfBuildingPart:p1[]]"]);
        assert_eq!(joined.reports, [(0, ErrorCode::CitygmlParentCycle)]);
    }

    #[test]
    fn a_nested_copy_of_an_object_written_elsewhere_is_left_out() {
        let part = || vec![member("p1", "bldg:BuildingPart", "b1", 2)];
        let joined = join_all(vec![
            feature_with(0, "bldg:Building", "b1", None, part(), Vec::new()),
            feature_with(1, "bldg:Building", "b2", None, part(), Vec::new()),
        ]);
        assert_eq!(
            top_shapes(&joined),
            ["b1[consistsOfBuildingPart:p1[]]", "b2[]"]
        );
        assert_eq!(
            joined.reports,
            [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[test]
    fn a_nested_copy_of_a_feature_is_left_out_for_the_feature() {
        let joined = join_all(vec![
            feature_with(
                0,
                "bldg:Building",
                "b1",
                None,
                vec![member("w1", "bldg:WallSurface", "b1", 2)],
                Vec::new(),
            ),
            feature(1, "bldg:WallSurface", "w1", Some("b1")),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[boundedBy:w1[]]"]);
        assert_eq!(joined.top[0].object.children[0].object.source, 1);
        assert_eq!(
            joined.reports,
            [(0, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[test]
    fn a_synthetic_parent_key_matches_only_within_its_document() {
        let wall = |building: &str| {
            vec![synthetic_member(
                "a.gml#_synid1",
                "bldg:WallSurface",
                building,
            )]
        };
        let mut features = vec![
            feature_with(0, "bldg:Building", "b1", None, wall("b1"), Vec::new()),
            feature_with(1, "bldg:Building", "b2", None, wall("b2"), Vec::new()),
            feature(2, "bldg:Window", "win1", Some("a.gml#_synid1")),
            feature(3, "bldg:Window", "win2", Some("a.gml#_synid1")),
        ];
        for (feature, root) in features.iter_mut().zip(["b1", "b2", "b1", "b2"]) {
            feature.document_root = Some(root.to_owned());
        }
        let joined = join_all(features);
        assert_eq!(
            top_shapes(&joined),
            [
                "b1[boundedBy:a.gml#_synid1[opening:win1[]]]",
                "b2[boundedBy:a.gml#_synid1[opening:win2[]]]"
            ]
        );
        assert!(joined.reports.is_empty());
    }

    #[test]
    fn a_nested_features_orphans_stay_top_level_in_its_place() {
        let joined = join_all(vec![
            feature(0, "bldg:Building", "b1", None),
            feature_with(
                1,
                "bldg:WallSurface",
                "w1",
                Some("b1"),
                vec![member("win9", "bldg:Window", "nowhere", 3)],
                Vec::new(),
            ),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[boundedBy:w1[]]", "orphan win9[]"]);
        assert!(joined.top[1].roots.is_empty());
    }

    #[test]
    fn members_owned_by_an_object_held_through_a_reference_are_left_out() {
        // A group read with its members and its parent building inlined: their
        // geometry is the group's members, but they live elsewhere.
        let mut parts = building(
            vec![
                own(),
                member("r1", "bldg:Room", "b1", 4),
                member("b2", "bldg:Building", "b1", 2),
                member("r2", "bldg:Room", "b2", 4),
                member("i1", "bldg:IntBuildingInstallation", "r2", 4),
                entry(None, None, Some("r2"), 4),
            ],
            Vec::new(),
        );
        parts.referenced = ["r1", "b2", "r2"].map(str::to_owned).into();

        let objects = build(parts);

        assert_eq!(objects.root.geometries.len(), 1);
        assert!(objects.root.children.is_empty());
        assert!(objects.orphans.is_empty());
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);
    }

    #[test]
    fn a_referenced_key_the_feature_also_holds_keeps_its_members() {
        let mut parts = building(
            vec![own(), member("w1", "bldg:WallSurface", "b1", 2)],
            vec![wall(Some("w1"), vec![name("N")])],
        );
        parts.referenced = ["w1", "b1"].map(str::to_owned).into();

        let objects = build(parts);

        assert_eq!(objects.root.geometries.len(), 1);
        assert_eq!(child_ids(&objects.root), [Some("w1")]);
        assert_eq!(objects.root.children[0].object.geometries.len(), 1);
    }

    #[test]
    fn a_keyless_member_of_another_city_object_class_is_its_own_object() {
        let objects = build(building(
            vec![
                own(),
                entry(None, Some("bldg:WallSurface"), None, 2),
                entry(None, Some("bldg:WallSurface"), None, 2),
                entry(None, Some("bldg:Building"), None, 2),
            ],
            Vec::new(),
        ));
        let root = &objects.root;
        assert_eq!(root.geometries.len(), 2);
        assert_eq!(child_ids(root), [None, None]);
        for child in &root.children {
            assert_eq!(child.property, bldg("boundedBy"));
            assert_eq!(child.object.class.element, bldg("WallSurface"));
            assert_eq!(child.object.key, None);
            assert_eq!(child.object.geometries.len(), 1);
        }
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);
    }

    #[test]
    fn a_keyless_member_of_another_class_nests_under_its_parent_key() {
        let objects = build(building(
            vec![
                member("w1", "bldg:WallSurface", "b1", 2),
                entry(None, Some("bldg:Window"), Some("w1"), 3),
            ],
            Vec::new(),
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.geometries.len(), 1);
        assert_eq!(wall.children.len(), 1);
        assert_eq!(wall.children[0].property, bldg("opening"));
        assert_eq!(wall.children[0].object.class.element, bldg("Window"));
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);
    }

    #[test]
    fn a_nested_copy_beats_an_orphan_copy_in_either_order() {
        // b0 holds r1 under a parent it lacks, so r1 is its orphan; b1 holds
        // the same r1 as a room.
        let orphaned = |source| {
            feature_with(
                source,
                "bldg:Building",
                "b0",
                None,
                vec![member("r1", "bldg:Room", "nowhere", 4)],
                Vec::new(),
            )
        };
        let nested = |source| {
            feature_with(
                source,
                "bldg:Building",
                "b1",
                None,
                vec![member("r1", "bldg:Room", "b1", 4)],
                Vec::new(),
            )
        };

        let orphan_first = join_all(vec![orphaned(0), nested(1)]);
        let nested_first = join_all(vec![nested(0), orphaned(1)]);

        assert_eq!(top_shapes(&orphan_first), ["b0[]", "b1[interiorRoom:r1[]]"]);
        assert_eq!(
            orphan_first.reports,
            [(0, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
        assert_eq!(top_shapes(&nested_first), ["b1[interiorRoom:r1[]]", "b0[]"]);
        assert_eq!(
            nested_first.reports,
            [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[test]
    fn a_copy_matched_to_tree_data_beats_a_geometry_only_copy() {
        let joined = join_all(vec![
            feature_with(
                0,
                "bldg:Building",
                "b1",
                None,
                vec![member("p1", "bldg:BuildingPart", "b1", 2)],
                Vec::new(),
            ),
            feature_with(
                1,
                "bldg:Building",
                "b2",
                None,
                vec![member("p1", "bldg:BuildingPart", "b2", 2)],
                vec![wrap(
                    "bldg",
                    "consistsOfBuildingPart",
                    object("bldg", "BuildingPart", Some("p1"), Vec::new()),
                )],
            ),
        ]);
        assert_eq!(
            top_shapes(&joined),
            ["b1[]", "b2[consistsOfBuildingPart:p1[]]"]
        );
        assert_eq!(
            joined.reports,
            [(0, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[test]
    fn objects_sharing_a_key_but_not_their_content_are_both_kept() {
        let joined = join_all(vec![
            feature_with(
                0,
                "bldg:Building",
                "b1",
                None,
                vec![member_at("w1", "bldg:WallSurface", "b1", 0.0)],
                Vec::new(),
            ),
            feature_with(
                1,
                "bldg:Building",
                "b2",
                None,
                vec![member_at("w1", "bldg:WallSurface", "b2", 3.0)],
                Vec::new(),
            ),
            feature_with(
                2,
                "bldg:WallSurface",
                "w1",
                Some("b9"),
                vec![member_at("w1", "bldg:WallSurface", "b9", 5.0)],
                Vec::new(),
            ),
        ]);
        assert_eq!(
            top_shapes(&joined),
            ["b1[boundedBy:w1[]]", "b2[boundedBy:w1[]]", "w1[]"]
        );
        assert_eq!(joined.reports, [(2, ErrorCode::CitygmlParentNotFound)]);
        assert_eq!(joined.copies, [false, false, false]);
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn a_feature_equal_to_an_earlier_one_is_left_out_whole() {
        let building = |source| {
            feature_with(
                source,
                "bldg:Building",
                "b1",
                None,
                vec![member_at("w1", "bldg:WallSurface", "b1", 0.0)],
                Vec::new(),
            )
        };
        let joined = join_all(vec![building(0), building(1)]);
        assert_eq!(top_shapes(&joined), ["b1[boundedBy:w1[]]"]);
        assert_eq!(joined.top[0].roots, [0]);
        assert_eq!(joined.copies, [false, true]);
        assert_eq!(
            joined.reports,
            [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn the_legacy_writer_leaves_no_feature_out_as_a_copy() {
        let mut features = vec![
            feature(0, "bldg:Building", "b1", None),
            feature(1, "bldg:Building", "b1", None),
        ];
        for feature in &mut features {
            feature.document_root = Some("b1".to_owned());
        }
        let joined = join_all(features);
        assert_eq!(top_shapes(&joined), ["b1[]", "b1[]"]);
        assert_eq!(joined.copies, [false, false]);
        assert!(joined.reports.is_empty());
    }

    /// Feature `source`, a `ty` keyed `id` under `parent`, read from the
    /// document whose top-level object is `document`.
    #[cfg(feature = "new-geometry")]
    fn in_document(
        source: usize,
        ty: &str,
        id: &str,
        parent: Option<&str>,
        document: &str,
    ) -> FeatureObjects {
        let mut objects = feature(source, ty, id, parent);
        objects.document_root = Some(document.to_owned());
        objects
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn a_whole_copy_its_parent_holds_beats_an_earlier_one_whose_parent_is_missing() {
        // A group g1 that is not among the features names room r1 as a member:
        // the reader extracts r1 from it first, naming g1 as its parent.
        let joined = join_all(vec![
            in_document(0, "bldg:Room", "r1", Some("g1"), "g1"),
            in_document(1, "bldg:Building", "b1", None, "b1"),
            in_document(2, "bldg:Room", "r1", Some("b1"), "b1"),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[interiorRoom:r1[]]"]);
        assert_eq!(joined.top[0].roots, [1, 2]);
        assert_eq!(joined.copies, [true, false, false]);
        assert_eq!(
            joined.reports,
            [(0, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn a_whole_copy_whose_parent_is_reached_through_a_reference_loses() {
        // The group's `grp:parent` b1 is extracted from it with its room, both
        // ahead of the real ones; that b1 names the missing group as parent.
        let joined = join_all(vec![
            in_document(0, "bldg:Building", "b1", Some("g1"), "g1"),
            in_document(1, "bldg:Room", "r1", Some("b1"), "g1"),
            in_document(2, "bldg:Room", "r1", Some("b1"), "b1"),
            in_document(3, "bldg:Building", "b1", None, "b1"),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[interiorRoom:r1[]]"]);
        assert_eq!(joined.top[0].roots, [2, 3]);
        assert_eq!(joined.copies, [true, true, false, false]);
        assert_eq!(
            joined.reports,
            [
                (0, ErrorCode::CitygmlDuplicateObjectNotWritten),
                (1, ErrorCode::CitygmlDuplicateObjectNotWritten)
            ]
        );
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn equal_whole_copies_placed_alike_keep_the_first() {
        let joined = join_all(vec![
            in_document(0, "bldg:Building", "b1", None, "b1"),
            in_document(1, "bldg:Room", "r1", Some("b1"), "b1"),
            in_document(2, "bldg:Room", "r1", Some("b1"), "b1"),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[interiorRoom:r1[]]"]);
        assert_eq!(joined.copies, [false, false, true]);
    }

    /// An id-less wall of class `bldg:WallSurface` in `b1`, one triangle at
    /// height `z`, keyed as the reader keys it or not at all.
    fn id_less_wall(z: f64, synthetic_key: Option<&str>) -> GeometryEntry {
        let mut wall = member_at("-", "bldg:WallSurface", "b1", z);
        wall.owner.key = synthetic_key.map(str::to_owned);
        wall.owner.synthetic = synthetic_key.is_some();
        wall
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn a_feature_equal_to_an_earlier_one_but_for_id_less_objects_is_left_out_whole() {
        for keys in [[None, None], [Some("a.gml#_synid1"), Some("a.gml#_synid7")]] {
            let building = |source, key| {
                feature_with(
                    source,
                    "bldg:Building",
                    "b1",
                    None,
                    vec![id_less_wall(0.0, key)],
                    Vec::new(),
                )
            };
            let joined = join_all(vec![building(0, keys[0]), building(1, keys[1])]);
            assert_eq!(joined.copies, [false, true], "{keys:?}");
            assert_eq!(joined.top.len(), 1, "{keys:?}");
            assert_eq!(
                joined.reports,
                [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)],
                "{keys:?}"
            );
        }
    }

    #[test]
    fn buildings_sharing_a_key_with_different_id_less_walls_are_both_kept() {
        let building = |source, z| {
            feature_with(
                source,
                "bldg:Building",
                "b1",
                None,
                vec![id_less_wall(z, None)],
                Vec::new(),
            )
        };
        let joined = join_all(vec![building(0, 0.0), building(1, 3.0)]);
        assert_eq!(
            top_shapes(&joined),
            ["b1[boundedBy:-[]]", "b1[boundedBy:-[]]"]
        );
        assert_eq!(joined.copies, [false, false]);
        assert!(joined.reports.is_empty(), "{:?}", joined.reports);
    }

    #[test]
    fn a_nested_copy_holding_an_id_less_object_is_left_out() {
        // p1 with an id-less wall, in b1 and again in b2, as the reader
        // inlines a `consistsOfBuildingPart` xlink.
        let part = |building: &str, wall_key: &str| {
            let mut wall = member_at("-", "bldg:WallSurface", "p1", 0.0);
            wall.owner.key = Some(wall_key.to_owned());
            wall.owner.synthetic = true;
            vec![member("p1", "bldg:BuildingPart", building, 2), wall]
        };
        let joined = join_all(vec![
            feature_with(
                0,
                "bldg:Building",
                "b1",
                None,
                part("b1", "x#_synid1"),
                Vec::new(),
            ),
            feature_with(
                1,
                "bldg:Building",
                "b2",
                None,
                part("b2", "x#_synid2"),
                Vec::new(),
            ),
        ]);
        assert_eq!(
            top_shapes(&joined),
            [
                "b1[consistsOfBuildingPart:p1[boundedBy:x#_synid1[]]]",
                "b2[]"
            ]
        );
        assert_eq!(
            joined.reports,
            [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[test]
    fn a_feature_that_only_loses_a_nested_copy_is_not_a_whole_copy() {
        let part = || vec![member("p1", "bldg:BuildingPart", "b1", 2)];
        let joined = join_all(vec![
            feature_with(0, "bldg:Building", "b1", None, part(), Vec::new()),
            feature_with(1, "bldg:Building", "b2", None, part(), Vec::new()),
        ]);
        assert_eq!(joined.copies, [false, false]);
    }

    const URF: &str = "https://www.geospatial.jp/iur/urf/3.1";

    /// i-UR's Urban Function ADE 3.1, read from a PLATEAU dataset's own copy.
    fn urf() -> (SchemaSet, Prefixes) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../testing/data/fixtures/plateau-citymodel/43206_tamana-shi_city_2024_citygml_2_op/schemas/iur/urf/3.1/urbanFunction.xsd",
        );
        let schema = SchemaSet::with_ade(&[path]).unwrap();
        let prefixes = Prefixes::with_ade(&[("urf".to_owned(), URF.to_owned())]);
        (schema, prefixes)
    }

    /// `urf:DistrictPlan` can nest a `urf:DistrictDevelopmentPlan` through
    /// `urf:target`, declared on every urban function and typed for any city
    /// object, or through its own `urf:districtDevelopmentPlan`, declared
    /// after it. The first used to win whatever the source wrote.
    #[test]
    fn a_nested_ade_object_keeps_the_wrapper_the_source_used() {
        let (schema, prefixes) = urf();
        let plan = QName::new(URF, "DistrictPlan");
        let development = QName::new(URF, "DistrictDevelopmentPlan");
        let own = QName::new(URF, "districtDevelopmentPlan");

        let first = containment_slot(&schema, &plan, &development).unwrap();
        assert_eq!(first.1, QName::new(URF, "target"));
        let kept = containment_slot_via(&schema, &plan, &development, Some(&own)).unwrap();
        assert_eq!(kept.1, own);
        assert!(kept.0 > first.0);
        // A wrapper that cannot hold the child falls back to the first slot.
        let unrelated = QName::new(URF, "function");
        assert_eq!(
            containment_slot_via(&schema, &plan, &development, Some(&unrelated)),
            Some(first)
        );

        let tree = vec![wrap(
            "urf",
            "districtDevelopmentPlan",
            object(
                "urf",
                "DistrictDevelopmentPlan",
                Some("ddp1"),
                vec![name("N")],
            ),
        )];
        let (remaining, tree) = split_nested_objects(&schema, &prefixes, tree);
        assert!(remaining.is_empty(), "{remaining:?}");
        assert_eq!(tree[0].wrapper, own);
        let parts = FeatureParts {
            source: 0,
            class: classes::resolve("urf:DistrictPlan", &prefixes, &schema).unwrap(),
            gml_id: Some("dp1".to_owned()),
            parent_key: None,
            document_root: None,
            geometries: Vec::new(),
            properties: Vec::new(),
            tree,
            referenced: HashSet::new(),
        };
        let mut classes = ClassResolver::new(&prefixes, &schema);
        let objects = build_feature_objects(parts, &schema, &prefixes, &mut classes);
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);
        assert_eq!(child_ids(&objects.root), [Some("ddp1")]);
        assert_eq!(objects.root.children[0].property, own);
        assert_eq!(objects.root.children[0].slot, kept.0);
    }

    /// i-UR 3.1, read from a PLATEAU dataset's own copy.
    fn uro() -> (SchemaSet, Prefixes) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../testing/data/fixtures/plateau-citymodel/15202_nagaoka-shi_city_2024_citygml_1_op/schemas/iur/uro/3.1/urbanObject.xsd",
        );
        let schema = SchemaSet::with_ade(&[path]).unwrap();
        let prefixes = Prefixes::with_ade(&[("uro".to_owned(), URO.to_owned())]);
        (schema, prefixes)
    }

    /// A DM line read from an id-less `uro:DmGeometricAttribute` the reader
    /// keyed `key`, under the object keyed `parent`.
    fn dm_line(key: &str, parent: Option<&str>) -> GeometryEntry {
        let mut entry = entry(Some(key), Some("uro:DmGeometricAttribute"), parent, 0);
        entry.owner.synthetic = true;
        entry.property = Some("lod0Geometry".to_owned());
        entry.element = GmlElement::MultiCurve {
            id: None,
            curves: vec![vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]],
        };
        entry
    }

    /// `uro:bldgDmAttribute` around a `uro:DmGeometricAttribute` keyed `key`.
    fn dm_attribute(key: &str) -> XmlProperty {
        let mut code = XmlProperty::new("uro", "dmCode");
        code.text = Some("1".to_owned());
        let mut dm = XmlProperty::new("uro", "DmGeometricAttribute");
        dm.synthetic_key = Some(key.to_owned());
        dm.children.push(code);
        wrap("uro", "bldgDmAttribute", dm)
    }

    /// The `uro:lod0Geometry` children of each DM attribute in `properties`.
    fn dm_geometry(properties: &[Placed]) -> Vec<usize> {
        properties
            .iter()
            .flat_map(|placed| &placed.property.children)
            .map(|dm| {
                dm.children
                    .iter()
                    .filter(|child| child.local == "lod0Geometry" && child.geometry.is_some())
                    .count()
            })
            .collect()
    }

    /// The reader keys an id-less ADE element that owns geometry, and the
    /// geometry goes back inside that element, on the feature itself and on
    /// a nested object, rather than onto the object as a property it lacks.
    #[test]
    fn geometry_of_a_keyed_ade_element_goes_back_inside_it() {
        let (schema, prefixes) = uro();
        let building = bldg("Building");
        let (placed, skipped) =
            placement::place(&schema, &prefixes, &building, vec![dm_attribute("k1")]);
        assert!(skipped.is_empty(), "{skipped:?}");
        let part = wrap(
            "bldg",
            "consistsOfBuildingPart",
            object(
                "bldg",
                "BuildingPart",
                Some("bp1"),
                vec![dm_attribute("k2")],
            ),
        );
        let (remaining, tree) = split_nested_objects(&schema, &prefixes, vec![part]);
        assert!(remaining.is_empty(), "{remaining:?}");
        let parts = FeatureParts {
            source: 0,
            class: classes::resolve("bldg:Building", &prefixes, &schema).unwrap(),
            gml_id: Some("b1".to_owned()),
            parent_key: None,
            document_root: None,
            geometries: vec![
                dm_line("k1", None),
                dm_line("k2", Some("bp1")),
                // Keyed, but no element carries the key: it stays on the
                // feature, which has no such property, as before.
                dm_line("k3", None),
            ],
            properties: placed,
            tree,
            referenced: HashSet::new(),
        };
        let mut classes = ClassResolver::new(&prefixes, &schema);
        let objects = build_feature_objects(parts, &schema, &prefixes, &mut classes);
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);

        let root = &objects.root;
        assert_eq!(dm_geometry(&root.properties), [1]);
        assert_eq!(root.geometries.len(), 1);
        assert_eq!(root.geometries[0].owner.key.as_deref(), Some("k3"));
        let part = &root.children[0].object;
        assert_eq!(dm_geometry(&part.properties), [1]);
        assert!(part.geometries.is_empty());
    }
}
