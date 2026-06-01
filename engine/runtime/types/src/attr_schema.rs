use indexmap::IndexMap;

use crate::attribute::Attribute;

/// Coarse attribute type, mirroring AttributeValue variants but value-free.
/// `Unknown` = key known, type not statically determinable (e.g. behind an expression).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrType {
    Bool,
    Number,
    String,
    DateTime,
    Array,
    Map,
    Bytes,
    Null,
    Unknown,
}

/// Whether a field is guaranteed present, or only conditionally produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Always,
    Maybe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrField {
    pub ty: AttrType,
    pub presence: Presence,
}

impl AttrField {
    pub fn always(ty: AttrType) -> Self {
        Self {
            ty,
            presence: Presence::Always,
        }
    }

    pub fn maybe(ty: AttrType) -> Self {
        Self {
            ty,
            presence: Presence::Maybe,
        }
    }
}

/// A node's attribute schema on one port.
/// `open == true` means the node may emit attributes whose names we can't
/// enumerate statically (sources, flatten, multi-expr) — disables "missing attr" errors downstream.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttrSchema {
    pub fields: IndexMap<Attribute, AttrField>,
    pub open: bool,
}

impl AttrSchema {
    /// A fully-unknown schema: any attribute may exist. Used to seed sources.
    pub fn open() -> Self {
        Self {
            fields: IndexMap::new(),
            open: true,
        }
    }

    /// An empty, closed schema: no attributes.
    pub fn empty() -> Self {
        Self {
            fields: IndexMap::new(),
            open: false,
        }
    }

    pub fn insert(&mut self, name: Attribute, field: AttrField) {
        self.fields.insert(name, field);
    }

    /// Join two schemas arriving on the SAME port (e.g. a router rejoin):
    /// - field in both, same type -> that type; presence Always only if both Always, else Maybe
    /// - field in both, different type -> AttrType::Unknown; presence rule as above
    /// - field in only one branch -> Presence::Maybe
    /// - result open if either input is open
    pub fn join(&self, other: &AttrSchema) -> AttrSchema {
        let mut out = AttrSchema {
            fields: IndexMap::new(),
            open: self.open || other.open,
        };
        for (name, a) in &self.fields {
            match other.fields.get(name) {
                Some(b) => {
                    let ty = if a.ty == b.ty {
                        a.ty
                    } else {
                        AttrType::Unknown
                    };
                    let presence =
                        if a.presence == Presence::Always && b.presence == Presence::Always {
                            Presence::Always
                        } else {
                            Presence::Maybe
                        };
                    out.fields.insert(name.clone(), AttrField { ty, presence });
                }
                None => {
                    out.fields
                        .insert(name.clone(), AttrField { ty: a.ty, presence: Presence::Maybe });
                }
            }
        }
        for (name, b) in &other.fields {
            if !self.fields.contains_key(name) {
                out.fields
                    .insert(name.clone(), AttrField { ty: b.ty, presence: Presence::Maybe });
            }
        }
        out
    }
}

/// An attribute a node reads from its input — checked against the inferred input schema later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrRef {
    pub name: Attribute,
    /// Input port name the reference applies to (use "default" when single-input).
    pub port: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::Attribute;

    fn attr(s: &str) -> Attribute {
        Attribute::new(s.to_string())
    }

    #[test]
    fn join_same_type_both_always_stays_always() {
        let mut a = AttrSchema::empty();
        a.insert(attr("x"), AttrField::always(AttrType::String));
        let mut b = AttrSchema::empty();
        b.insert(attr("x"), AttrField::always(AttrType::String));
        let j = a.join(&b);
        assert_eq!(j.fields.get(&attr("x")), Some(&AttrField::always(AttrType::String)));
        assert!(!j.open);
    }

    #[test]
    fn join_diff_type_becomes_unknown() {
        let mut a = AttrSchema::empty();
        a.insert(attr("x"), AttrField::always(AttrType::String));
        let mut b = AttrSchema::empty();
        b.insert(attr("x"), AttrField::always(AttrType::Number));
        let j = a.join(&b);
        assert_eq!(j.fields.get(&attr("x")).unwrap().ty, AttrType::Unknown);
    }

    #[test]
    fn join_one_branch_only_becomes_maybe() {
        let mut a = AttrSchema::empty();
        a.insert(attr("x"), AttrField::always(AttrType::String));
        let b = AttrSchema::empty();
        let j = a.join(&b);
        assert_eq!(j.fields.get(&attr("x")).unwrap().presence, Presence::Maybe);
    }

    #[test]
    fn join_open_propagates() {
        let a = AttrSchema::open();
        let b = AttrSchema::empty();
        assert!(a.join(&b).open);
    }
}
