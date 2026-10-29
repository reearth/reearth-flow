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
    /// extracted from, which tells copies of one feature apart from features
    /// that only share a key.
    pub document_root: Option<String>,
    pub geometries: Vec<GeometryEntry>,
    /// The feature's own properties, placed in its class.
    pub properties: Vec<Placed>,
    /// The nested objects of its attribute tree.
    pub tree: Vec<TreeObject>,
    /// The keys of the objects its attribute tree holds through a reference
    /// property, such as a group's `grp:groupMember`, at any depth.
    pub referenced: HashSet<String>,
    /// The city object holding each keyed object of its attribute tree that
    /// is not a city object ([`data_owners`]).
    pub data_owners: HashMap<String, Option<String>>,
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
    /// `None` for an object that names no city-object class; it is folded
    /// into the nearest object above it that has one.
    class: Option<CityClass>,
    key: Option<String>,
    /// The object's own data, for an object read from the attribute tree.
    data: Option<Vec<XmlProperty>>,
    /// The property the source wrapped the object in, for an object read
    /// from the attribute tree.
    wrapper: Option<QName>,
    /// `None` for the root.
    parent: Option<usize>,
    geometries: Vec<GeometryEntry>,
    /// Cleared when the node is dropped or folded.
    kept: bool,
}

impl Node {
    fn tree(class: Option<CityClass>, wrapper: QName, element: XmlProperty, parent: usize) -> Self {
        let XmlProperty {
            attrs, children, ..
        } = element;
        // The writer writes the object element itself, so of its XML
        // attributes only the identity is kept, as the key.
        let id = attrs
            .into_iter()
            .find(|(name, _)| name == "gml:id")
            .map(|(_, id)| id);
        Self {
            class,
            key: id,
            data: Some(children),
            wrapper: Some(wrapper),
            parent: Some(parent),
            geometries: Vec::new(),
            kept: true,
        }
    }
}

/// Builds one feature's object tree from the nested objects of its attribute
/// tree and its geometry members.
///
/// Objects are matched by `gml:id` alone. Each member goes onto the attribute
/// tree's object its owner's `gml:id` names, or onto the city object whose
/// data holds the owner, such as an ADE object owning geometry inside a wall.
/// A member whose owner matches nothing goes onto the feature, and is
/// reported when the owner is another object: an owner without a `gml:id`, or
/// one the attribute tree does not hold, as with `keepAttributes` off. An
/// object that is not a city object is folded into the nearest object above
/// it that is. Objects their parent cannot hold are returned as orphans, and
/// reported.
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
        referenced,
        data_owners,
    } = parts;
    let mut reports = HashSet::new();

    // Nodes: the feature, then the attribute tree's objects in pre-order.
    let mut nodes = vec![Node {
        class: Some(class),
        key: gml_id.clone(),
        data: None,
        wrapper: None,
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

    for entry in geometries {
        let key = entry.owner.key.as_deref();
        let target = match (
            key.and_then(|key| index.get(key)),
            key.and_then(|key| data_owners.get(key)),
        ) {
            (Some(&node), _) => node,
            (None, Some(holder)) => holder
                .as_deref()
                .and_then(|holder| index.get(holder).copied())
                .unwrap_or(ROOT),
            (None, None) => {
                reports.extend(report_unmatched_owner(
                    &entry,
                    &nodes[ROOT],
                    schema,
                    prefixes,
                    classes,
                ));
                ROOT
            }
        };
        nodes[target].geometries.push(entry);
    }

    // Fold every object that is not a city object.
    for node in 1..nodes.len() {
        if nodes[node].class.is_some() {
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

    // Place each object in its parent, in pre-order, then build the orphans.
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (node, entry) in nodes.iter().enumerate().skip(1) {
        if entry.kept {
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
        orphans: Vec::new(),
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

/// The codes to report for a member whose owner `gml:id` matches no object,
/// written on the feature: the owner is a nested object the attribute tree
/// carries no key for, unless the member names no key and the feature's own
/// class. An owner type that names nothing, such as a misspelling, is
/// reported as such. An owner that is not a city object, such as an ADE
/// object left out of `adeSchemas`, is folded onto the feature by design.
fn report_unmatched_owner(
    entry: &GeometryEntry,
    root: &Node,
    schema: &SchemaSet,
    prefixes: &Prefixes,
    classes: &mut ClassResolver,
) -> Vec<ErrorCode> {
    let mut codes = Vec::new();
    let class = match entry.owner.feature_type.as_deref() {
        Some(value) => {
            let class = classes.class_of(value);
            if class.is_none() && names_no_element(value, schema, prefixes) {
                codes.push(ErrorCode::CitygmlUnknownFeatureType);
            }
            class
        }
        None => None,
    };
    let another_object = entry.owner.key.is_some()
        || class.is_some_and(|class| {
            root.class.as_ref().map(|own| &own.element) != Some(&class.element)
        });
    if another_object {
        codes.push(ErrorCode::CitygmlNestedDataNotMatched);
    }
    codes
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
/// member whose owner is `referenced` and not an object the feature holds
/// itself.
fn without_referenced(
    geometries: Vec<GeometryEntry>,
    referenced: &HashSet<String>,
    held: &HashMap<String, usize>,
) -> Vec<GeometryEntry> {
    geometries
        .into_iter()
        .filter(|entry| {
            entry
                .owner
                .key
                .as_deref()
                .is_none_or(|key| held.contains_key(key) || !referenced.contains(key))
        })
        .collect()
}

/// The `gml:id`s of every object `attributes` holds
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
                if let Some(AttributeValue::String(key)) = map.get(GML_ID_KEY) {
                    if !key.is_empty() {
                        keys.insert(key.clone());
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

/// The city object holding each object of `attributes` that has a `gml:id`
/// and is not a city object, such as an ADE object, by the object's
/// `gml:id`: the `gml:id` of the nearest city object around it that has one,
/// or `None` for the feature itself.
/// Objects held through a reference property are left out.
///
/// Read from the attributes themselves, like [`referenced_keys`], so an
/// object whose namespace the writer does not bind, such as an ADE left out
/// of `adeSchemas`, still names its holder: the geometry it owns is written
/// on that city object, as the object itself is not.
pub fn data_owners(
    attributes: &Attributes,
    schema: &SchemaSet,
    prefixes: &Prefixes,
) -> HashMap<String, Option<String>> {
    let mut owners = HashMap::new();
    for (name, value) in attributes {
        collect_data_owners(name.as_str(), value, None, schema, prefixes, &mut owners);
    }
    owners
}

fn collect_data_owners(
    name: &str,
    value: &AttributeValue,
    holder: Option<&str>,
    schema: &SchemaSet,
    prefixes: &Prefixes,
    owners: &mut HashMap<String, Option<String>>,
) {
    if is_reference_name(name, prefixes) {
        return;
    }
    match value {
        AttributeValue::Array(items) => {
            for item in items {
                collect_data_owners(name, item, holder, schema, prefixes, owners);
            }
        }
        AttributeValue::Map(map) => {
            let key = match map.get(GML_ID_KEY) {
                Some(AttributeValue::String(key)) if !key.is_empty() => Some(key.as_str()),
                _ => None,
            };
            let city_object = name
                .split_once(':')
                .and_then(|(prefix, local)| {
                    prefixes
                        .namespace(prefix)
                        .map(|namespace| QName::new(namespace, local))
                })
                .is_some_and(|element| is_city_object(schema, &element));
            let holder = match (city_object, key) {
                (true, Some(key)) => Some(key),
                (true, None) => holder,
                (false, Some(key)) => {
                    owners
                        .entry(key.to_owned())
                        .or_insert(holder.map(str::to_owned));
                    holder
                }
                (false, None) => holder,
            };
            for (child_name, child) in map {
                collect_data_owners(child_name.as_str(), child, holder, schema, prefixes, owners);
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
        let gml_id = if node == ROOT {
            self.root_gml_id.clone()
        } else {
            entry.key.clone().filter(|key| is_ncname(key))
        };
        ObjectNode {
            class,
            key: entry.key.clone(),
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
/// class cannot hold it, stays top-level and is reported.
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

    let mut index: HashMap<String, Vec<Location>> = HashMap::new();
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
        let matches: Vec<&Location> = index.get(parent_key).into_iter().flatten().collect();
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
/// source `gml:id`, class and content (properties and geometry coordinates,
/// at every depth). Objects that share only a key, as built by hand, are two
/// objects, and the writer gives the second a generated id.
fn same_object(a: &ObjectNode, b: &ObjectNode) -> bool {
    a.key.is_some() && a.key == b.key && same_content(a, b)
}

/// Whether two objects have the same class, properties and geometry
/// coordinates, and hold the same objects: equal by content and source
/// `gml:id`, at every depth.
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
                && a.object.key == b.object.key
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

/// How a copy of a feature ranks against the others, lower kept first: where
/// it sits, then whether it is the weaker copy at that place, held by a
/// parent reached only through a reference property.
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

    /// Whether a feature naming no parent was extracted from inside another
    /// object of its document rather than being the document's top-level
    /// object, as the room and building the reader reaches through a group's
    /// `grp:groupMember` and `grp:parent` are: the copies it inlines there
    /// name no parent, since the group they sit in was not extracted.
    fn inside_another(&self, feature: usize) -> bool {
        let feature = &self.features[feature];
        feature
            .document_root
            .as_deref()
            .is_some_and(|root| feature.key.as_deref() != Some(root))
    }

    /// Whether `feature` is its document's top-level object, or names a
    /// parent that can hold it, nested in a feature's own tree, that is
    /// placed soundly in turn. A cycle is not.
    fn is_sound(&mut self, feature: usize) -> bool {
        if let Some(sound) = self.sound[feature] {
            return sound;
        }
        if self.features[feature].parent_key.is_none() {
            let sound = !self.inside_another(feature);
            self.sound[feature] = Some(sound);
            return sound;
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
    /// such as the room the reader extracts from the copy of its building it
    /// reaches through a group's `grp:parent`.
    ///
    /// A feature naming no parent [`Self::inside_another`] object ranks as
    /// one held through a reference, behind a copy held soundly.
    fn rank(&mut self, feature: usize) -> CopyRank {
        if self.features[feature].parent_key.is_none() {
            return if self.inside_another(feature) {
                (Place::Nested, true)
            } else {
                (Place::Own, false)
            };
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
    /// Lower is kept first: the feature's own object, nested in its tree,
    /// nested in an orphan, an orphan.
    place: Place,
    /// The candidates it is nested in, outermost first.
    ancestors: Vec<usize>,
}

/// Leaves out every nested object or orphan that is [`same_object`] as
/// another, with everything nested in it, and reports it.
///
/// Which copy is kept does not depend on arrival order. A feature's own
/// object is never left out here. Otherwise a copy nested in the feature's
/// tree beats one nested in an orphan, which beats an orphan. Arrival order
/// only breaks ties. Copies are settled outermost first, as
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
            collect_candidates(object, location, &mut Vec::new(), &mut candidates);
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
            best_first(&mut members, |member| (candidates[member].place, false));
            for &loser in &members[1..] {
                // A feature's own object is left out only as a whole copy.
                if candidates[loser].place == Place::Own {
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
    ancestors: &mut Vec<usize>,
    out: &mut Vec<Candidate>,
) {
    let id = match &object.key {
        Some(_) => {
            let place = match (location.orphan, location.path.is_empty()) {
                (None, true) => Place::Own,
                (None, false) => Place::Nested,
                (Some(_), false) => Place::InOrphan,
                (Some(_), true) => Place::Orphan,
            };
            out.push(Candidate {
                location: location.clone(),
                place,
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
        collect_candidates(&child.object, child_location, ancestors, out);
    }
    if id.is_some() {
        ancestors.pop();
    }
}

fn index_objects(
    object: &ObjectNode,
    location: Location,
    index: &mut HashMap<String, Vec<Location>>,
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
        index.entry(key.clone()).or_default().push(location);
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

    fn entry(key: Option<&str>, ty: Option<&str>, lod: u8) -> GeometryEntry {
        GeometryEntry {
            lod,
            property: None,
            element: GmlElement::MultiSurface {
                id: None,
                surfaces: Vec::new(),
            },
            owner: EntryOwner {
                key: key.map(str::to_owned),
                feature_type: ty.map(str::to_owned),
            },
        }
    }

    /// A member owned by `key`, of class `ty`, holding one triangle at
    /// height `z`.
    fn member_at(key: &str, ty: &str, z: f64) -> GeometryEntry {
        let mut entry = member(key, ty, 2);
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

    /// A member owned by `key`, of class `ty`.
    fn member(key: &str, ty: &str, lod: u8) -> GeometryEntry {
        entry(Some(key), Some(ty), lod)
    }

    /// The feature's own geometry.
    fn own() -> GeometryEntry {
        entry(Some("b1"), Some("bldg:Building"), 1)
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

    /// `bldg:consistsOfBuildingPart` around a `bldg:BuildingPart`.
    fn part(id: Option<&str>, children: Vec<XmlProperty>) -> XmlProperty {
        wrap(
            "bldg",
            "consistsOfBuildingPart",
            object("bldg", "BuildingPart", id, children),
        )
    }

    /// `bldg:interiorRoom` around a `bldg:Room`.
    fn room(id: &str) -> XmlProperty {
        wrap(
            "bldg",
            "interiorRoom",
            object("bldg", "Room", Some(id), Vec::new()),
        )
    }

    /// A room `id` in `bldg:opening`, where no class that has the property
    /// can hold it, so it is an orphan of its feature.
    fn misplaced_room(id: &str) -> XmlProperty {
        wrap(
            "bldg",
            "opening",
            object("bldg", "Room", Some(id), Vec::new()),
        )
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
            data_owners: HashMap::new(),
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
    fn members_join_the_tree_objects_their_owner_keys_name() {
        let objects = build(building(
            vec![
                own(),
                member("w1", "bldg:WallSurface", 2),
                member("w2", "bldg:WallSurface", 2),
                member("w1", "bldg:WallSurface", 2),
            ],
            vec![wall(Some("w1"), Vec::new()), wall(Some("w2"), Vec::new())],
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
        assert!(objects.orphans.is_empty());
        assert_eq!(objects.key.as_deref(), Some("b1"));
        assert_eq!(root.key.as_deref(), Some("b1"));
    }

    #[test]
    fn a_member_joins_a_tree_object_at_any_depth() {
        let objects = build(building(
            vec![
                member("w1", "bldg:WallSurface", 2),
                member("win1", "bldg:Window", 3),
            ],
            vec![wall(Some("w1"), vec![window(Some("win1"), Vec::new())])],
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.geometries.len(), 1);
        assert_eq!(wall.children.len(), 1);
        let window = &wall.children[0];
        assert_eq!(window.property, bldg("opening"));
        assert_eq!(window.object.class.element, bldg("Window"));
        assert_eq!(window.object.gml_id.as_deref(), Some("win1"));
        assert_eq!(window.object.geometries.len(), 1);
        assert!(objects.orphans.is_empty());
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn an_owner_key_equal_to_the_features_id_is_the_feature_itself() {
        let objects = build(building(
            vec![own(), member("b1", "bldg:Building", 2)],
            Vec::new(),
        ));
        assert_eq!(objects.root.geometries.len(), 2);
        assert!(objects.root.children.is_empty());
        assert!(objects.orphans.is_empty());
        assert!(objects.reports.is_empty());
    }

    /// Objects are matched by `gml:id` alone: the geometry of a nested object
    /// without one names only its class, so it goes on the feature and is
    /// reported, and the object stays in place, keyless, with no geometry.
    #[test]
    fn an_id_less_objects_geometry_goes_on_the_feature_and_is_reported() {
        let objects = build(building(
            vec![own(), entry(None, Some("bldg:WallSurface"), 2)],
            vec![wall(None, vec![name("w")])],
        ));
        let root = &objects.root;
        assert_eq!(root.geometries.len(), 2);
        assert_eq!(root.children.len(), 1);
        let wall = &root.children[0].object;
        assert!(wall.geometries.is_empty());
        assert_eq!(wall.key, None);
        assert_eq!(wall.gml_id, None);
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
    }

    #[test]
    fn an_owner_key_that_is_not_a_valid_xml_name_is_not_kept() {
        let objects = build(building(
            vec![
                member("1abc", "bldg:WallSurface", 2),
                member("a b", "bldg:WallSurface", 2),
            ],
            vec![
                wall(Some("1abc"), Vec::new()),
                wall(Some("a b"), Vec::new()),
            ],
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
            vec![member("w1", "bldg:WallSurface", 2)],
            vec![wall(Some("w1"), vec![name("N")])],
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.geometries.len(), 1);
        assert_eq!(wall.properties[0].property, name("N"));
        assert!(objects.reports.is_empty());
    }

    #[test]
    fn a_member_whose_owner_is_not_in_the_tree_goes_on_the_feature_and_is_reported() {
        let objects = build(building(
            vec![member("w2", "bldg:WallSurface", 2)],
            vec![wall(Some("w1"), vec![name("N")])],
        ));
        let root = &objects.root;
        assert_eq!(child_ids(root), [Some("w1")]);
        assert_eq!(root.geometries.len(), 1);
        let w1 = &root.children[0].object;
        assert_eq!(property_names(w1), ["gml:name"]);
        assert!(w1.geometries.is_empty());
        assert!(objects.orphans.is_empty());
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
    }

    #[test]
    fn the_tree_supplies_a_parent_that_owns_no_geometry() {
        let objects = build(building(
            vec![member("win1", "bldg:Window", 3)],
            vec![wall(Some("w1"), vec![window(Some("win1"), Vec::new())])],
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(wall.gml_id.as_deref(), Some("w1"));
        assert!(wall.geometries.is_empty());
        assert_eq!(child_ids(wall), [Some("win1")]);
        assert_eq!(wall.children[0].property, bldg("opening"));
        assert!(objects.root.has_geometry());
    }

    #[test]
    fn a_tree_object_of_no_class_hands_its_children_to_its_parent() {
        let abstract_object = wrap(
            "bldg",
            "opening",
            object(
                "core",
                "_CityObject",
                Some("c1"),
                vec![window(Some("win1"), Vec::new())],
            ),
        );
        let objects = build(building(
            vec![member("win1", "bldg:Window", 3)],
            vec![wall(Some("w1"), vec![abstract_object])],
        ));
        let wall = &objects.root.children[0].object;
        assert_eq!(child_ids(wall), [Some("win1")]);
        assert_eq!(wall.children[0].object.geometries.len(), 1);
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);
    }

    #[test]
    fn an_unmatched_misspelt_owner_type_is_reported_and_its_geometry_kept_on_the_feature() {
        let objects = build(building(
            vec![member("s1", "bldg:WallSurfce", 2)],
            vec![wall(Some("w1"), Vec::new())],
        ));
        let root = &objects.root;
        assert_eq!(child_ids(root), [Some("w1")]);
        assert_eq!(root.geometries.len(), 1);
        assert!(root.children[0].object.geometries.is_empty());
        assert_eq!(
            objects.reports,
            codes(&[
                ErrorCode::CitygmlUnknownFeatureType,
                ErrorCode::CitygmlNestedDataNotMatched
            ])
        );
    }

    #[test]
    fn an_unmatched_owner_key_without_a_type_is_reported() {
        let objects = build(building(
            vec![entry(Some("x1"), None, 2)],
            vec![wall(Some("w1"), Vec::new())],
        ));
        assert_eq!(objects.root.geometries.len(), 1);
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
    }

    #[test]
    fn a_key_less_owner_that_is_not_another_city_object_goes_quietly_on_the_feature() {
        for owner in [
            entry(None, None, 2),
            entry(None, Some("bldg:Building"), 2),
            entry(None, Some("core:Address"), 2),
            entry(None, Some("uro:DmGeometricAttribute"), 2),
        ] {
            let objects = build(building(vec![owner], vec![wall(Some("w1"), Vec::new())]));
            assert_eq!(objects.root.geometries.len(), 1);
            assert!(objects.root.children[0].object.geometries.is_empty());
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
            vec![member("c1", "bldg:WallSurface", 2)],
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
    fn a_window_under_the_building_is_a_reported_orphan() {
        let objects = build(building(
            vec![own(), member("win1", "bldg:Window", 3)],
            vec![wrap(
                "bldg",
                "boundedBy",
                object("bldg", "Window", Some("win1"), Vec::new()),
            )],
        ));
        assert!(objects.root.children.is_empty());
        assert_eq!(objects.orphans.len(), 1);
        assert_eq!(objects.orphans[0].class.element, bldg("Window"));
        assert_eq!(objects.orphans[0].geometries.len(), 1);
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlParentCannotHold])
        );
    }

    #[test]
    fn without_a_tree_key_the_geometry_goes_on_the_feature_and_the_data_stays() {
        let objects = build(building(
            vec![member("w1", "bldg:WallSurface", 2)],
            vec![wall(None, vec![name("N")])],
        ));
        let root = &objects.root;
        assert_eq!(root.geometries.len(), 1);
        assert_eq!(child_ids(root), [None]);
        assert_eq!(property_names(&root.children[0].object), ["gml:name"]);
        assert!(root.children[0].object.geometries.is_empty());
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
    }

    #[test]
    fn a_key_less_data_only_object_is_kept_beside_matched_ones() {
        let objects = build(building(
            vec![member("w1", "bldg:WallSurface", 2)],
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
                vec![member("w1", "bldg:WallSurface", 2)],
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
    fn an_unambiguous_bare_owner_type_names_another_object() {
        let objects = build(building(vec![entry(None, Some("Room"), 4)], Vec::new()));
        assert_eq!(objects.root.geometries.len(), 1);
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
    }

    #[test]
    fn an_ambiguous_bare_owner_type_is_reported() {
        let objects = build(building(
            vec![entry(None, Some("WallSurface"), 2)],
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
        let attributes = attrs(tree);
        let converted =
            properties::from_attributes(&attributes, &HashSet::new(), &prefixes, &|_, local| {
                properties::has_geometry_name(local)
            });
        let (remaining, tree) = split_nested_objects(schema(), &prefixes, converted.properties);
        let mut parts = building(entries, Vec::new());
        let (placed, skipped) = placement::place(schema(), &prefixes, &bldg("Building"), remaining);
        parts.properties = placed;
        parts.tree = tree;
        parts.data_owners = data_owners(&attributes, schema(), &prefixes);
        let mut skips = converted.skipped;
        skips.extend(skipped);
        (build(parts), skips)
    }

    #[test]
    fn a_data_less_object_read_with_its_id_matches_its_geometry() {
        let (objects, skipped) = read_and_build(
            json!({ "bldg:boundedBy": { "bldg:WallSurface": { "@gml:id": "w1" } } }),
            vec![member("w1", "bldg:WallSurface", 2)],
        );
        assert!(skipped.is_empty(), "{skipped:?}");
        assert_eq!(child_ids(&objects.root), [Some("w1")]);
        assert_eq!(objects.root.children[0].object.geometries.len(), 1);
        assert!(objects.reports.is_empty());
    }

    /// An id-less object read with data keeps its data in place; its
    /// geometry, which names no `gml:id`, goes on the feature and is reported.
    #[test]
    fn an_id_less_object_read_with_data_keeps_it_but_not_its_geometry() {
        let (objects, skipped) = read_and_build(
            json!({ "bldg:boundedBy": { "bldg:WallSurface": { "gml:name": "N" } } }),
            vec![entry(None, Some("bldg:WallSurface"), 2)],
        );
        assert!(skipped.is_empty(), "{skipped:?}");
        assert_eq!(objects.root.children.len(), 1);
        let wall = &objects.root.children[0].object;
        assert_eq!(property_names(wall), ["gml:name"]);
        assert!(wall.geometries.is_empty());
        assert_eq!(objects.root.geometries.len(), 1);
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
    }

    /// An object that is not a city object, such as an ADE object, stays in
    /// its city object's data, so its geometry goes on that city object,
    /// matched by the object's `gml:id`, even when the writer does not bind
    /// its namespace and leaves the object out.
    #[test]
    fn geometry_of_an_object_in_a_city_objects_data_goes_on_that_city_object() {
        let (objects, _) = read_and_build(
            json!({ "bldg:boundedBy": { "bldg:WallSurface": {
                "@gml:id": "w1",
                "uro:wallSurfaceDmAttribute": { "uro:DmGeometricAttribute": {
                    "@gml:id": "dm1",
                    "uro:dmCode": "1"
                } }
            } } }),
            vec![own(), member("dm1", "uro:DmGeometricAttribute", 0)],
        );
        let root = &objects.root;
        assert_eq!(root.geometries.len(), 1);
        assert_eq!(child_ids(root), [Some("w1")]);
        assert_eq!(root.children[0].object.geometries.len(), 1);
        assert!(
            !objects
                .reports
                .contains(&ErrorCode::CitygmlNestedDataNotMatched),
            "{:?}",
            objects.reports
        );
    }

    #[test]
    fn geometry_of_an_object_in_the_features_own_data_goes_on_the_feature() {
        let (objects, _) = read_and_build(
            json!({ "uro:buildingDmAttribute": { "uro:DmGeometricAttribute": {
                "@gml:id": "dm1"
            } } }),
            vec![member("dm1", "uro:DmGeometricAttribute", 0)],
        );
        assert_eq!(objects.root.geometries.len(), 1);
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);
    }

    #[test]
    fn a_key_held_through_a_reference_in_an_objects_data_is_not_that_objects() {
        let (objects, _) = read_and_build(
            json!({ "bldg:boundedBy": { "bldg:WallSurface": {
                "@gml:id": "w1",
                "core:generalizesTo": { "uro:Other": { "@gml:id": "g1" } }
            } } }),
            vec![member("g1", "uro:Other", 2)],
        );
        assert!(objects.root.children[0].object.geometries.is_empty());
        assert_eq!(objects.root.geometries.len(), 1);
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
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
        let mut geometries = vec![entry(Some(id), Some(ty), 2)];
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
            data_owners: HashMap::new(),
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
                vec![member("WS1", "bldg:WallSurface", 2)],
                vec![wall(Some("WS1"), Vec::new())],
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
        let own = |z| vec![member_at("b1", "bldg:Building", z)];
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
                vec![member("p9", "bldg:BuildingPart", 2)],
                vec![part(Some("p9"), Vec::new())],
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
            Some("r9"),
            vec![member("r9", "bldg:Room", 4)],
            vec![misplaced_room("r9")],
        )]);
        assert_eq!(top_shapes(&joined), ["w1[]", "orphan r9[]"]);
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
            vec![member("p1", "bldg:BuildingPart", 2)],
            vec![part(Some("p1"), Vec::new())],
        )]);
        assert_eq!(top_shapes(&joined), ["b1[consistsOfBuildingPart:p1[]]"]);
        assert_eq!(joined.reports, [(0, ErrorCode::CitygmlParentCycle)]);
    }

    #[test]
    fn a_nested_copy_of_an_object_written_elsewhere_is_left_out() {
        let members = || vec![member("p1", "bldg:BuildingPart", 2)];
        let tree = || vec![part(Some("p1"), Vec::new())];
        let joined = join_all(vec![
            feature_with(0, "bldg:Building", "b1", None, members(), tree()),
            feature_with(1, "bldg:Building", "b2", None, members(), tree()),
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
                vec![member("w1", "bldg:WallSurface", 2)],
                vec![wall(Some("w1"), Vec::new())],
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
    fn a_nested_features_orphans_stay_top_level_in_its_place() {
        let joined = join_all(vec![
            feature(0, "bldg:Building", "b1", None),
            feature_with(
                1,
                "bldg:WallSurface",
                "w1",
                Some("b1"),
                vec![member("r9", "bldg:Room", 4)],
                vec![misplaced_room("r9")],
            ),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[boundedBy:w1[]]", "orphan r9[]"]);
        assert!(joined.top[1].roots.is_empty());
    }

    #[test]
    fn members_owned_by_an_object_held_through_a_reference_are_left_out() {
        // A group read with its members and its parent building inlined: their
        // geometry is the group's members, but they live elsewhere.
        let mut parts = building(
            vec![
                own(),
                member("r1", "bldg:Room", 4),
                member("b2", "bldg:Building", 2),
                member("r2", "bldg:Room", 4),
                member("i1", "bldg:IntBuildingInstallation", 4),
            ],
            Vec::new(),
        );
        parts.referenced = ["r1", "b2", "r2", "i1"].map(str::to_owned).into();

        let objects = build(parts);

        assert_eq!(objects.root.geometries.len(), 1);
        assert!(objects.root.children.is_empty());
        assert!(objects.orphans.is_empty());
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);
    }

    #[test]
    fn a_referenced_key_the_feature_also_holds_keeps_its_members() {
        let mut parts = building(
            vec![own(), member("w1", "bldg:WallSurface", 2)],
            vec![wall(Some("w1"), vec![name("N")])],
        );
        parts.referenced = ["w1", "b1"].map(str::to_owned).into();

        let objects = build(parts);

        assert_eq!(objects.root.geometries.len(), 1);
        assert_eq!(child_ids(&objects.root), [Some("w1")]);
        assert_eq!(objects.root.children[0].object.geometries.len(), 1);
    }

    /// What the reader writes for id-less walls when it does not record the
    /// hierarchy: the walls' geometry goes on the feature, flattened as the
    /// legacy writer does, and is reported.
    #[test]
    fn a_key_less_member_of_another_city_object_class_goes_on_the_feature() {
        let objects = build(building(
            vec![
                own(),
                entry(None, Some("bldg:WallSurface"), 2),
                entry(None, Some("bldg:WallSurface"), 2),
                entry(None, Some("bldg:Building"), 2),
            ],
            vec![wall(None, Vec::new()), wall(None, Vec::new())],
        ));
        let root = &objects.root;
        assert_eq!(root.geometries.len(), 4);
        assert_eq!(child_ids(root), [None, None]);
        for child in &root.children {
            assert!(child.object.geometries.is_empty());
        }
        assert_eq!(
            objects.reports,
            codes(&[ErrorCode::CitygmlNestedDataNotMatched])
        );
    }

    #[test]
    fn a_nested_copy_beats_an_orphan_copy_in_either_order() {
        // b0 holds r1 under a wall, which cannot hold it, so r1 is its
        // orphan; b1 holds the same r1 as a room.
        let orphaned = |source| {
            feature_with(
                source,
                "bldg:Building",
                "b0",
                None,
                vec![member("r1", "bldg:Room", 4)],
                vec![wall(Some("w0"), vec![misplaced_room("r1")])],
            )
        };
        let nested = |source| {
            feature_with(
                source,
                "bldg:Building",
                "b1",
                None,
                vec![member("r1", "bldg:Room", 4)],
                vec![room("r1")],
            )
        };

        let orphan_first = join_all(vec![orphaned(0), nested(1)]);
        let nested_first = join_all(vec![nested(0), orphaned(1)]);

        assert_eq!(
            top_shapes(&orphan_first),
            ["b0[boundedBy:w0[]]", "b1[interiorRoom:r1[]]"]
        );
        assert_eq!(
            orphan_first.reports,
            [(0, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
        assert_eq!(
            top_shapes(&nested_first),
            ["b1[interiorRoom:r1[]]", "b0[boundedBy:w0[]]"]
        );
        assert_eq!(
            nested_first.reports,
            [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)]
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
                vec![member_at("w1", "bldg:WallSurface", 0.0)],
                vec![wall(Some("w1"), Vec::new())],
            ),
            feature_with(
                1,
                "bldg:Building",
                "b2",
                None,
                vec![member_at("w1", "bldg:WallSurface", 3.0)],
                vec![wall(Some("w1"), Vec::new())],
            ),
            feature_with(
                2,
                "bldg:WallSurface",
                "w1",
                Some("b9"),
                vec![member_at("w1", "bldg:WallSurface", 5.0)],
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
                vec![member_at("w1", "bldg:WallSurface", 0.0)],
                vec![wall(Some("w1"), Vec::new())],
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

    /// The CityGML reader's shape for a group listed ahead of its building,
    /// read with extractTags: the copies it extracts through the group's
    /// `grp:groupMember` and `grp:parent` name no parent, as the group they
    /// sit in is not extracted, but are not their document's top-level
    /// object. They lose to the real ones.
    #[cfg(feature = "new-geometry")]
    #[test]
    fn a_parentless_copy_from_inside_another_object_loses_to_a_nested_one() {
        let joined = join_all(vec![
            in_document(0, "bldg:Room", "r1", None, "g1"),
            in_document(1, "bldg:Room", "r1", Some("b1"), "g1"),
            in_document(2, "bldg:Building", "b1", None, "g1"),
            in_document(3, "bldg:Room", "r1", Some("b1"), "b1"),
            in_document(4, "bldg:Building", "b1", None, "b1"),
        ]);
        assert_eq!(top_shapes(&joined), ["b1[interiorRoom:r1[]]"]);
        assert_eq!(joined.top[0].roots, [3, 4]);
        assert_eq!(joined.copies, [true, true, true, false, false]);
        assert_eq!(
            joined.reports,
            [
                (0, ErrorCode::CitygmlDuplicateObjectNotWritten),
                (1, ErrorCode::CitygmlDuplicateObjectNotWritten),
                (2, ErrorCode::CitygmlDuplicateObjectNotWritten)
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

    /// An id-less wall of class `bldg:WallSurface`, one triangle at height
    /// `z`: its member, which names no owner `gml:id`, and its entry in the
    /// attribute tree.
    fn id_less_wall(z: f64) -> (GeometryEntry, XmlProperty) {
        let mut member = member_at("-", "bldg:WallSurface", z);
        member.owner.key = None;
        (member, wall(None, Vec::new()))
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn a_feature_equal_to_an_earlier_one_with_id_less_objects_is_left_out_whole() {
        let building = |source| {
            let (member, tree) = id_less_wall(0.0);
            feature_with(
                source,
                "bldg:Building",
                "b1",
                None,
                vec![member],
                vec![tree],
            )
        };
        let joined = join_all(vec![building(0), building(1)]);
        assert_eq!(joined.copies, [false, true]);
        assert_eq!(joined.top.len(), 1);
        assert_eq!(
            joined.reports,
            [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[test]
    fn buildings_sharing_a_key_with_different_id_less_walls_are_both_kept() {
        let building = |source, z| {
            let (member, tree) = id_less_wall(z);
            feature_with(
                source,
                "bldg:Building",
                "b1",
                None,
                vec![member],
                vec![tree],
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
        let building = |source, id| {
            let (wall_member, wall_tree) = id_less_wall(0.0);
            feature_with(
                source,
                "bldg:Building",
                id,
                None,
                vec![member("p1", "bldg:BuildingPart", 2), wall_member],
                vec![part(Some("p1"), vec![wall_tree])],
            )
        };
        let joined = join_all(vec![building(0, "b1"), building(1, "b2")]);
        assert_eq!(
            top_shapes(&joined),
            ["b1[consistsOfBuildingPart:p1[boundedBy:-[]]]", "b2[]"]
        );
        assert_eq!(
            joined.reports,
            [(1, ErrorCode::CitygmlDuplicateObjectNotWritten)]
        );
    }

    #[test]
    fn a_feature_that_only_loses_a_nested_copy_is_not_a_whole_copy() {
        let members = || vec![member("p1", "bldg:BuildingPart", 2)];
        let tree = || vec![part(Some("p1"), Vec::new())];
        let joined = join_all(vec![
            feature_with(0, "bldg:Building", "b1", None, members(), tree()),
            feature_with(1, "bldg:Building", "b2", None, members(), tree()),
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
            data_owners: HashMap::new(),
        };
        let mut classes = ClassResolver::new(&prefixes, &schema);
        let objects = build_feature_objects(parts, &schema, &prefixes, &mut classes);
        assert!(objects.reports.is_empty(), "{:?}", objects.reports);
        assert_eq!(child_ids(&objects.root), [Some("ddp1")]);
        assert_eq!(objects.root.children[0].property, own);
        assert_eq!(objects.root.children[0].slot, kept.0);
    }
}
