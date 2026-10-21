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

use reearth_flow_citygml::schema::{QName, SchemaSet};
use reearth_flow_diagnostics::ErrorCode;

use super::classes::{class_of_element, CityClass, ClassResolver, Prefixes};
use super::model::{GeometryEntry, NestedObject, ObjectNode};
use super::placement::{self, is_city_object, Placed};
use super::properties::{Skip, XmlProperty};
use super::writer::is_ncname;

const CORE: &str = "http://www.opengis.net/citygml/2.0";
const GRP: &str = "http://www.opengis.net/citygml/cityobjectgroup/2.0";

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
        let holds_object = !is_reference(&property.qname(prefixes))
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
    let class = schema.class_for_element(parent)?;
    schema
        .slots_holding(parent, child)
        .into_iter()
        .find_map(|index| {
            let property = class.slots()[index].declared.clone()?;
            (!is_reference(&property)).then_some((index, property))
        })
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
    /// Made from geometry members whose key the attribute tree lacks.
    group: bool,
    /// `None` for the root, and for a group whose parent key names no node.
    parent: Option<usize>,
    geometries: Vec<GeometryEntry>,
    /// Cleared when the node is dropped or folded.
    kept: bool,
}

impl Node {
    fn tree(class: Option<CityClass>, element: XmlProperty, parent: usize) -> Self {
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
        properties,
        tree,
    } = parts;
    let mut reports = HashSet::new();

    // Nodes: the feature, then the attribute tree's objects in pre-order.
    let mut nodes = vec![Node {
        class: Some(class),
        key: gml_id.clone(),
        synthetic: false,
        data: None,
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

    // Entries, by owner key; a new key makes a group in first-seen order.
    let mut group_parents: Vec<(usize, Option<String>)> = Vec::new();
    let mut keyless = Vec::new();
    for entry in geometries {
        let Some(key) = entry.owner.key.clone() else {
            keyless.push(entry);
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
        let entries = std::mem::take(&mut nodes[node].geometries);
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
        nodes.push(Node::tree(class, object.element, parent));
        if let Some(key) = &nodes[node].key {
            index.entry(key.clone()).or_insert(node);
        }
        add_tree_objects(nodes, index, object.children, node, schema, prefixes);
    }
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
            match child_class
                .and_then(|element| containment_slot(self.schema, &class.element, &element))
            {
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
    /// `(source, code)`, each pair once, in the order found.
    pub reports: Vec<(usize, ErrorCode)>,
}

/// Joins each feature into the object its parent key names: another
/// feature, or an object nested in one.
///
/// A nested object whose `gml:id` is a feature's own, or an earlier nested
/// object's, is left out first, so a copy the reader inlined through
/// `xlink:href` is written once. A feature whose parent key names no object
/// or several, names an object inside the feature itself, or names one whose
/// class cannot hold it, stays top-level and is reported. A synthetic key is
/// matched only within the feature's own document where that is known, as
/// two documents can share one. A feature repeating an earlier feature's key
/// from the same document is a second copy, left out whole and reported.
///
/// The result lists each top-level feature in arrival order, each followed
/// by its orphans; a nested feature's orphans stay where the feature was.
pub fn join(features: Vec<FeatureObjects>, schema: &SchemaSet) -> Joined {
    let mut features = features;
    let mut reports = Reports::default();

    // A later feature whose key and document both repeat an earlier
    // feature's is a second copy of it, as when one document is read twice,
    // and is left out whole. Features that share only a key, such as ones
    // built by hand, may be different objects, so they stay.
    let copies: Vec<bool> = {
        let mut firsts = HashSet::new();
        features
            .iter()
            .map(|feature| match (&feature.key, &feature.document_root) {
                (Some(key), Some(document)) => !firsts.insert((key.as_str(), document.as_str())),
                _ => false,
            })
            .collect()
    };
    for (source, _) in copies.iter().enumerate().filter(|(_, copy)| **copy) {
        report(
            &mut reports,
            source,
            ErrorCode::CitygmlDuplicateObjectNotWritten,
        );
    }

    let root_keys: HashSet<String> = features
        .iter()
        .filter_map(|feature| feature.key.clone())
        .collect();
    let mut seen = HashSet::new();
    for (feature, _) in features.iter_mut().zip(&copies).filter(|(_, copy)| !**copy) {
        drop_copies(
            &mut feature.root.children,
            &root_keys,
            &mut seen,
            &mut reports,
        );
        feature.orphans.retain_mut(|orphan| {
            if is_copy(orphan, &root_keys, &mut seen) {
                report(
                    &mut reports,
                    orphan.source,
                    ErrorCode::CitygmlDuplicateObjectNotWritten,
                );
                return false;
            }
            drop_copies(&mut orphan.children, &root_keys, &mut seen, &mut reports);
            true
        });
    }

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

/// Whether `object` repeats a feature's key or an earlier object's, which
/// marks the key as seen. A synthetic key names no source object, so it
/// never repeats one.
fn is_copy(object: &ObjectNode, root_keys: &HashSet<String>, seen: &mut HashSet<String>) -> bool {
    match &object.key {
        Some(key) if !object.synthetic => root_keys.contains(key) || !seen.insert(key.clone()),
        _ => false,
    }
}

/// Drops, in pre-order, each nested object that [`is_copy`], with
/// everything nested in it.
fn drop_copies(
    children: &mut Vec<NestedObject>,
    root_keys: &HashSet<String>,
    seen: &mut HashSet<String>,
    reports: &mut Reports,
) {
    children.retain_mut(|child| {
        if is_copy(&child.object, root_keys, seen) {
            report(
                reports,
                child.object.source,
                ErrorCode::CitygmlDuplicateObjectNotWritten,
            );
            return false;
        }
        drop_copies(&mut child.object.children, root_keys, seen, reports);
        true
    });
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
    use super::super::model::{EntryOwner, GeometryEntry, GmlElement, ObjectNode};
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
        let converted = properties::from_attributes(&attrs(tree), &HashSet::new(), &prefixes);
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
        // different documents, may be two objects.
        for documents in [[None, None], [Some("d1"), Some("d2")]] {
            let mut features = vec![
                feature(0, "bldg:Building", "b1", None),
                feature(1, "bldg:Building", "b1", None),
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
}
