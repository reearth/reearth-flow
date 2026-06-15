use indexmap::IndexMap;
use serde::Serialize;

use crate::attribute::Attribute;

/// Coarse attribute type, mirroring AttributeValue variants but value-free.
/// `Unknown` = key known, type not statically determinable (e.g. behind an expression).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
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
                    out.fields.insert(
                        name.clone(),
                        AttrField {
                            ty: a.ty,
                            presence: Presence::Maybe,
                        },
                    );
                }
            }
        }
        for (name, b) in &other.fields {
            if !self.fields.contains_key(name) {
                out.fields.insert(
                    name.clone(),
                    AttrField {
                        ty: b.ty,
                        presence: Presence::Maybe,
                    },
                );
            }
        }
        out
    }
}

/// Top-level JSON contract returned by the `schema` command.
#[derive(Debug, Serialize)]
pub struct SchemaReport {
    pub version: u32,
    #[serde(rename = "sampleSize")]
    pub sample_size: usize,
    pub nodes: IndexMap<String, NodeReport>,
}

#[derive(Debug, Serialize)]
pub struct NodeReport {
    pub name: String,
    pub ports: IndexMap<String, PortReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PortReport {
    pub open: bool,
    pub fields: Vec<FieldReport>,
}

#[derive(Debug, Serialize)]
pub struct FieldReport {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: AttrType,
    pub presence: Presence,
}

impl PortReport {
    /// Build the ordered DTO from the in-memory schema (preserves IndexMap order).
    pub fn from_schema(schema: &AttrSchema) -> Self {
        PortReport {
            open: schema.open,
            fields: schema
                .fields
                .iter()
                .map(|(name, field)| FieldReport {
                    name: name.to_string(),
                    ty: field.ty,
                    presence: field.presence,
                })
                .collect(),
        }
    }
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
        assert_eq!(
            j.fields.get(&attr("x")),
            Some(&AttrField::always(AttrType::String))
        );
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

    #[test]
    fn schema_report_serializes_to_expected_json() {
        use indexmap::IndexMap;
        let mut fields = IndexMap::new();
        fields.insert(
            Attribute::new("myAttribute".to_string()),
            AttrField::always(AttrType::String),
        );
        fields.insert(
            Attribute::new("address".to_string()),
            AttrField::maybe(AttrType::String),
        );
        let schema = AttrSchema {
            fields,
            open: false,
        };

        let mut ports: IndexMap<String, PortReport> = IndexMap::new();
        ports.insert("default".to_string(), PortReport::from_schema(&schema));
        let mut nodes: IndexMap<String, NodeReport> = IndexMap::new();
        nodes.insert(
            "node-1".to_string(),
            NodeReport {
                name: "GeoJsonReader".to_string(),
                ports,
                note: None,
            },
        );
        let report = SchemaReport {
            version: 1,
            sample_size: 10,
            nodes,
        };

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["version"], 1);
        assert_eq!(json["sampleSize"], 10);
        let fields_json = &json["nodes"]["node-1"]["ports"]["default"]["fields"];
        assert_eq!(fields_json[0]["name"], "myAttribute");
        assert_eq!(fields_json[0]["type"], "String");
        assert_eq!(fields_json[0]["presence"], "always");
        assert_eq!(fields_json[1]["name"], "address");
        assert_eq!(fields_json[1]["presence"], "maybe");
        assert_eq!(json["nodes"]["node-1"]["ports"]["default"]["open"], false);
        // note: None must be omitted
        assert!(json["nodes"]["node-1"].get("note").is_none());
    }
}
