//! Reads fastxml's compiled particle trees into slots. fastxml's own content
//! automaton is no substitute: its positions are leaves, not the top-level
//! particles a writer orders children by, and their bounds are private.

use std::collections::{HashMap, HashSet};

use fastxml::schema::types::{
    CompiledSchema, ComplexType, DerivationMethod, ElementDef, NsName, Particle, TypeDef,
};
use fastxml::schema::xsd::XSD_NAMESPACE;

use super::model::{ClassModel, Slot, SlotAccepts};
use super::{MaxOccurs, QName, SchemaError, Wildcard};

/// What the model keeps of a global element.
#[derive(Debug, Clone)]
pub(super) struct ElementInfo {
    pub(super) type_name: Option<QName>,
    pub(super) substitution_group: Option<QName>,
    pub(super) is_abstract: bool,
}

/// How deep a derivation or substitution chain may run before it is taken
/// for a cycle.
pub(super) const MAX_DEPTH: u32 = 32;

/// The global elements, and the class model of every named complex type.
type Built = (HashMap<QName, ElementInfo>, HashMap<QName, ClassModel>);

pub(super) fn build(schema: &CompiledSchema) -> Result<Built, SchemaError> {
    let mut elements = HashMap::new();
    let mut substitutes: HashMap<QName, Vec<QName>> = HashMap::new();
    let mut element_types = HashMap::new();
    for (name, def) in &schema.elements_ns {
        let name = qname(name);
        if let Some(type_name) = &def.type_ns {
            element_types.insert(name.clone(), qname(type_name));
        }
        let head = def.substitution_ns.as_ref().map(qname);
        if let Some(head) = &head {
            substitutes
                .entry(head.clone())
                .or_default()
                .push(name.clone());
        }
        elements.insert(
            name,
            ElementInfo {
                type_name: def.type_ns.as_ref().map(qname),
                substitution_group: head,
                is_abstract: def.is_abstract,
            },
        );
    }
    let builder = Builder {
        substitutes,
        element_types,
    };
    let mut classes = HashMap::new();
    for (name, def) in &schema.types_ns {
        // The built-in types carry no element content worth a class.
        let TypeDef::Complex(ct) = def else { continue };
        if &*name.namespace_uri == XSD_NAMESPACE {
            continue;
        }
        let mut slots = Vec::new();
        for particle in effective(schema, ct, 0)? {
            builder.flatten(&particle, &mut slots);
        }
        classes.insert(qname(name), ClassModel { slots });
    }
    Ok((elements, classes))
}

/// A type's content in document order: an extension's base chain first, then
/// its own particle. A restriction restates its content, so it inherits none.
fn effective(
    schema: &CompiledSchema,
    ct: &ComplexType,
    depth: u32,
) -> Result<Vec<Particle>, SchemaError> {
    if depth > MAX_DEPTH {
        return Err(SchemaError::Cycle {
            kind: "type",
            name: ct.name.clone(),
        });
    }
    let mut out = Vec::new();
    if ct.derivation == Some(DerivationMethod::Extension) {
        if let Some(TypeDef::Complex(base)) = schema.complex_base_def(ct) {
            out.extend(effective(schema, base, depth + 1)?);
        }
    }
    if let Some(particle) = &ct.particle {
        out.push((**particle).clone());
    }
    Ok(out)
}

struct Builder {
    /// Each head and the elements naming it as their substitution group.
    substitutes: HashMap<QName, Vec<QName>>,
    /// The declared type of each global element that has a named one.
    element_types: HashMap<QName, QName>,
}

impl Builder {
    /// Append the slots of `particle`. A `(1,1)` sequence, the outermost one or
    /// an inlined group reference, is spliced into its parent, so its members
    /// are slots of their own. Any other sequence is one slot, except that a
    /// lone member without bounds of its own takes the sequence's: that is how
    /// fastxml writes a bounded reference to a one-element group.
    fn flatten(&self, particle: &Particle, out: &mut Vec<Slot>) {
        match particle {
            Particle::Sequence {
                min: 1,
                max: Some(1),
                items,
            } => {
                for item in items {
                    self.flatten(item, out);
                }
            }
            Particle::Sequence { min, max, items } => {
                let mut members = Vec::new();
                splice(items, &mut members);
                let slot = match members.as_slice() {
                    [only] if bounds(only) == (1, Some(1)) => self.slot(only, *min, *max),
                    _ => self.slot(particle, *min, *max),
                };
                out.push(slot);
            }
            other => {
                let (min, max) = bounds(other);
                out.push(self.slot(other, min, max));
            }
        }
    }

    fn slot(&self, particle: &Particle, min: u32, max: Option<u32>) -> Slot {
        let declared = match particle {
            Particle::Element(def) => Some(element_name(def)),
            _ => None,
        };
        let property_type = match particle {
            Particle::Element(def) => match &def.ref_ns {
                Some(reference) => self.element_types.get(&qname(reference)).cloned(),
                None => def.type_ns.as_ref().map(qname),
            },
            _ => None,
        };
        let mut accepts = SlotAccepts::default();
        self.collect(particle, &mut accepts);
        Slot {
            declared,
            property_type,
            accepts,
            min,
            max: max.map_or(MaxOccurs::Unbounded, MaxOccurs::Bounded),
        }
    }

    /// Everything `particle` accepts, at any depth of nesting.
    fn collect(&self, particle: &Particle, accepts: &mut SlotAccepts) {
        match particle {
            Particle::Element(def) => self.close(&element_name(def), &mut accepts.elements),
            Particle::Wildcard(constraint) => {
                let wildcard = Wildcard::from_fastxml(constraint);
                if !accepts.wildcards.contains(&wildcard) {
                    accepts.wildcards.push(wildcard);
                }
            }
            Particle::Sequence { items, .. } | Particle::Choice { items, .. } => {
                for item in items {
                    self.collect(item, accepts);
                }
            }
            Particle::All { elements, .. } => {
                for def in elements {
                    self.close(&element_name(def), &mut accepts.elements);
                }
            }
        }
    }

    /// `head` and every element whose substitution chain reaches it.
    fn close(&self, head: &QName, names: &mut HashSet<QName>) {
        if !names.insert(head.clone()) {
            return;
        }
        for substitute in self.substitutes.get(head).into_iter().flatten() {
            self.close(substitute, names);
        }
    }
}

/// The members of `items` with every `(1,1)` sequence among them opened up.
fn splice<'a>(items: &'a [Particle], out: &mut Vec<&'a Particle>) {
    for item in items {
        match item {
            Particle::Sequence {
                min: 1,
                max: Some(1),
                items,
            } => splice(items, out),
            other => out.push(other),
        }
    }
}

/// A particle's own occurrence bounds; `None` is unbounded.
fn bounds(particle: &Particle) -> (u32, Option<u32>) {
    match particle {
        Particle::Element(def) => (def.min_occurs, def.max_occurs),
        Particle::Wildcard(constraint) => (constraint.min_occurs, constraint.max_occurs),
        Particle::Sequence { min, max, .. } | Particle::Choice { min, max, .. } => (*min, *max),
        Particle::All { min, .. } => (*min, Some(1)),
    }
}

/// The expanded name an element particle stands for. A reference's `name`
/// holds the prefix it was written with, so only `ref_ns` is reliable there.
fn element_name(def: &ElementDef) -> QName {
    match &def.ref_ns {
        Some(name) => qname(name),
        None => QName::new(def.namespace.clone().unwrap_or_default(), def.name.as_str()),
    }
}

fn qname(name: &NsName) -> QName {
    QName::new(&*name.namespace_uri, &*name.local_name)
}

#[cfg(test)]
mod tests {
    use fastxml::schema::types::{WildcardConstraint, WildcardNamespace};

    use super::*;

    fn element(local: &str, min: u32, max: Option<u32>) -> Particle {
        let mut def = ElementDef::new(local);
        def.namespace = Some("urn:t".to_owned());
        def.min_occurs = min;
        def.max_occurs = max;
        Particle::Element(def)
    }

    fn sequence(min: u32, max: Option<u32>, items: Vec<Particle>) -> Particle {
        Particle::Sequence { min, max, items }
    }

    fn slots(particle: &Particle) -> Vec<Slot> {
        let builder = Builder {
            substitutes: HashMap::new(),
            element_types: HashMap::new(),
        };
        let mut out = Vec::new();
        builder.flatten(particle, &mut out);
        out
    }

    fn t(local: &str) -> QName {
        QName::new("urn:t", local)
    }

    #[test]
    fn a_one_one_sequence_nested_in_the_outermost_is_spliced() {
        let tree = sequence(
            1,
            Some(1),
            vec![
                element("a", 1, Some(1)),
                sequence(1, Some(1), vec![element("b", 0, None)]),
            ],
        );
        let slots = slots(&tree);
        let declared: Vec<_> = slots.iter().map(|s| s.declared.clone()).collect();
        assert_eq!(declared, [Some(t("a")), Some(t("b"))]);
        assert_eq!((slots[1].min, slots[1].max), (0, MaxOccurs::Unbounded));
    }

    #[test]
    fn a_lone_member_with_bounds_of_its_own_keeps_the_sequence_a_slot() {
        let slots = slots(&sequence(0, Some(1), vec![element("a", 0, None)]));
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].declared, None);
        assert_eq!((slots[0].min, slots[0].max), (0, MaxOccurs::Bounded(1)));
        assert!(slots[0].accepts(&t("a")));
    }

    #[test]
    fn a_bounded_sequence_of_one_spliced_sequence_gives_its_member_the_bounds() {
        let slots = slots(&sequence(
            0,
            None,
            vec![sequence(1, Some(1), vec![element("a", 1, Some(1))])],
        ));
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].declared, Some(t("a")));
        assert_eq!((slots[0].min, slots[0].max), (0, MaxOccurs::Unbounded));
    }

    #[test]
    fn a_wildcard_written_twice_in_one_slot_is_kept_once() {
        let any = Particle::Wildcard(WildcardConstraint {
            namespace: WildcardNamespace::Other,
            process_contents: fastxml::schema::types::ProcessContents::Lax,
            min_occurs: 0,
            max_occurs: None,
            target_namespace: Some("urn:t".to_owned()),
        });
        let choice = Particle::Choice {
            min: 1,
            max: Some(1),
            items: vec![any.clone(), any],
        };
        let slots = slots(&choice);
        assert_eq!(
            slots[0].accepts.wildcards,
            [Wildcard::Other("urn:t".into())]
        );
    }
}
