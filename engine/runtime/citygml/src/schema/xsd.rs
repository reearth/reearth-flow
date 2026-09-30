//! Reads one XSD document into raw declarations. Resolves prefixes within the
//! document; resolves nothing across documents, which is `model`'s job.

use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::{MaxOccurs, QName, SchemaError};

/// The XML Schema namespace.
pub(crate) const XS: &str = "http://www.w3.org/2001/XMLSchema";

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawSchema {
    pub target_namespace: String,
    /// `xs:import` and `xs:include` locations, in document order.
    pub imports: Vec<String>,
    pub elements: Vec<RawElement>,
    pub complex_types: Vec<RawComplexType>,
    pub groups: Vec<RawGroup>,
}

/// A global `xs:element`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawElement {
    pub name: QName,
    pub type_name: Option<QName>,
    pub substitution_group: Option<QName>,
    pub is_abstract: bool,
}

/// A named `xs:complexType`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawComplexType {
    pub name: QName,
    /// The `complexContent` extension base. `None` for a restriction, whose
    /// particles restate the whole content model, for simple content, and for a
    /// type with no base.
    pub base: Option<QName>,
    /// The type's own particles; a top-level `xs:sequence` is spliced in.
    pub particles: Vec<Particle>,
}

/// A named `xs:group`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawGroup {
    pub name: QName,
    pub particles: Vec<Particle>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Particle {
    pub term: Term,
    pub min: u32,
    pub max: MaxOccurs,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Term {
    Element(QName),
    Sequence(Vec<Particle>),
    Choice(Vec<Particle>),
    GroupRef(QName),
    /// `xs:any`, with its `namespace` constraint as written.
    Any(String),
}

/// An element of the document in the XSD namespace. Elements from other
/// namespaces (inside annotations) keep a `#`-prefixed name so nothing below
/// matches them.
struct Node {
    local: String,
    attrs: HashMap<String, String>,
    children: Vec<Node>,
}

/// Parse `bytes`, an XSD document, reporting problems against `file`.
pub(crate) fn parse(bytes: &[u8], file: &str) -> Result<RawSchema, SchemaError> {
    let (root, prefixes) = read_tree(bytes, file)?;
    if root.local != "schema" {
        return Err(invalid(file, "the document element is not xs:schema"));
    }
    let tns = root
        .attrs
        .get("targetNamespace")
        .cloned()
        .unwrap_or_default();
    let ctx = Ctx {
        file,
        tns: &tns,
        prefixes: &prefixes,
        qualified: root.attrs.get("elementFormDefault").map(String::as_str) == Some("qualified"),
    };
    let mut schema = RawSchema {
        target_namespace: tns.clone(),
        imports: Vec::new(),
        elements: Vec::new(),
        complex_types: Vec::new(),
        groups: Vec::new(),
    };
    for child in &root.children {
        let name = child.attrs.get("name");
        match (child.local.as_str(), name) {
            ("import" | "include", _) => {
                if let Some(location) = child.attrs.get("schemaLocation") {
                    schema.imports.push(location.clone());
                }
            }
            ("element", Some(name)) => schema.elements.push(RawElement {
                name: QName::new(tns.as_str(), name.as_str()),
                type_name: ctx.resolve_attr(child, "type")?,
                substitution_group: ctx.resolve_attr(child, "substitutionGroup")?,
                is_abstract: child.attrs.get("abstract").map(String::as_str) == Some("true"),
            }),
            ("complexType", Some(name)) => schema
                .complex_types
                .push(ctx.complex_type(QName::new(tns.as_str(), name.as_str()), child)?),
            ("group", Some(name)) => schema.groups.push(RawGroup {
                name: QName::new(tns.as_str(), name.as_str()),
                particles: ctx.body(child)?,
            }),
            _ => {}
        }
    }
    Ok(schema)
}

struct Ctx<'a> {
    file: &'a str,
    tns: &'a str,
    prefixes: &'a HashMap<String, String>,
    qualified: bool,
}

impl Ctx<'_> {
    /// Resolve a QName-valued attribute. An unprefixed value takes the default
    /// namespace, as XSD specifies, or no namespace when none is declared.
    fn resolve(&self, value: &str) -> Result<QName, SchemaError> {
        match value.split_once(':') {
            Some((prefix, local)) => self
                .prefixes
                .get(prefix)
                .map(|namespace| QName::new(namespace.as_str(), local))
                .ok_or_else(|| SchemaError::UnboundPrefix {
                    file: self.file.to_owned(),
                    prefix: prefix.to_owned(),
                    value: value.to_owned(),
                }),
            None => Ok(QName::new(
                self.prefixes.get("").cloned().unwrap_or_default(),
                value,
            )),
        }
    }

    fn resolve_attr(&self, node: &Node, attr: &str) -> Result<Option<QName>, SchemaError> {
        node.attrs
            .get(attr)
            .map(|value| self.resolve(value))
            .transpose()
    }

    fn complex_type(&self, name: QName, node: &Node) -> Result<RawComplexType, SchemaError> {
        for content in &node.children {
            match content.local.as_str() {
                // Simple content carries text, never child elements.
                "simpleContent" => {
                    return Ok(RawComplexType {
                        name,
                        base: None,
                        particles: Vec::new(),
                    })
                }
                "complexContent" => {
                    for derivation in &content.children {
                        match derivation.local.as_str() {
                            "extension" => {
                                return Ok(RawComplexType {
                                    name,
                                    base: self.resolve_attr(derivation, "base")?,
                                    particles: self.body(derivation)?,
                                })
                            }
                            "restriction" => {
                                return Ok(RawComplexType {
                                    name,
                                    base: None,
                                    particles: self.body(derivation)?,
                                })
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(RawComplexType {
            name,
            base: None,
            particles: self.body(node)?,
        })
    }

    /// The particles of the compositor under `node`, with a top-level sequence
    /// spliced in so its members are the slots.
    fn body(&self, node: &Node) -> Result<Vec<Particle>, SchemaError> {
        let mut out = Vec::new();
        for child in &node.children {
            match child.local.as_str() {
                "sequence" => out.extend(self.particles(child)?),
                "choice" | "group" | "all" => out.push(self.particle(child)?),
                _ => {}
            }
        }
        Ok(out)
    }

    fn particles(&self, node: &Node) -> Result<Vec<Particle>, SchemaError> {
        node.children
            .iter()
            .filter(|child| {
                matches!(
                    child.local.as_str(),
                    "element" | "sequence" | "choice" | "group" | "any" | "all"
                )
            })
            .map(|child| self.particle(child))
            .collect()
    }

    fn particle(&self, node: &Node) -> Result<Particle, SchemaError> {
        let min = match node.attrs.get("minOccurs") {
            None => 1,
            Some(value) => value
                .parse()
                .map_err(|_| invalid(self.file, &format!("minOccurs `{value}` is not a count")))?,
        };
        let max =
            match node.attrs.get("maxOccurs").map(String::as_str) {
                None => MaxOccurs::Bounded(1),
                Some("unbounded") => MaxOccurs::Unbounded,
                Some(value) => MaxOccurs::Bounded(value.parse().map_err(|_| {
                    invalid(self.file, &format!("maxOccurs `{value}` is not a count"))
                })?),
            };
        let term = match node.local.as_str() {
            "element" => match (node.attrs.get("ref"), node.attrs.get("name")) {
                (Some(reference), _) => Term::Element(self.resolve(reference)?),
                (None, Some(name)) => {
                    let qualified = match node.attrs.get("form").map(String::as_str) {
                        Some("qualified") => true,
                        Some("unqualified") => false,
                        _ => self.qualified,
                    };
                    Term::Element(QName::new(
                        if qualified { self.tns } else { "" },
                        name.as_str(),
                    ))
                }
                (None, None) => {
                    return Err(invalid(self.file, "an xs:element has neither name nor ref"))
                }
            },
            "sequence" => Term::Sequence(self.particles(node)?),
            "choice" => Term::Choice(self.particles(node)?),
            "group" => match node.attrs.get("ref") {
                Some(reference) => Term::GroupRef(self.resolve(reference)?),
                None => return Err(invalid(self.file, "an xs:group in content has no ref")),
            },
            "any" => Term::Any(
                node.attrs
                    .get("namespace")
                    .cloned()
                    .unwrap_or_else(|| "##any".to_owned()),
            ),
            "all" => {
                return Err(SchemaError::Unsupported {
                    file: self.file.to_owned(),
                    construct: "xs:all",
                })
            }
            other => {
                return Err(invalid(
                    self.file,
                    &format!("unexpected xs:{other} in content"),
                ))
            }
        };
        Ok(Particle { term, min, max })
    }
}

/// Build the element tree, recording namespace declarations file-wide (the
/// first binding of a prefix wins; XSD documents declare them on the root).
fn read_tree(bytes: &[u8], file: &str) -> Result<(Node, HashMap<String, String>), SchemaError> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut prefixes = HashMap::new();
    let mut stack: Vec<Node> = Vec::new();
    let mut root = None;
    let mut buf = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| SchemaError::Xml {
                file: file.to_owned(),
                line: line_at(bytes, reader.buffer_position() as usize),
                message: e.to_string(),
            })?;
        match event {
            Event::Start(e) => stack.push(open(&e, &mut prefixes, file)?),
            Event::Empty(e) => {
                let node = open(&e, &mut prefixes, file)?;
                close(node, &mut stack, &mut root);
            }
            Event::End(_) => {
                let node = stack.pop().ok_or_else(|| SchemaError::Xml {
                    file: file.to_owned(),
                    line: line_at(bytes, reader.buffer_position() as usize),
                    message: "an end tag has no matching start tag".to_owned(),
                })?;
                close(node, &mut stack, &mut root);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    if !stack.is_empty() {
        return Err(SchemaError::Xml {
            file: file.to_owned(),
            line: line_at(bytes, bytes.len()),
            message: "the document ends before its elements are closed".to_owned(),
        });
    }
    let root = root.ok_or_else(|| invalid(file, "the document is empty"))?;
    Ok((root, prefixes))
}

fn open(
    e: &BytesStart<'_>,
    prefixes: &mut HashMap<String, String>,
    file: &str,
) -> Result<Node, SchemaError> {
    let mut attrs = HashMap::new();
    for attr in e.attributes() {
        let attr = attr.map_err(|err| invalid(file, &err.to_string()))?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
        let value = attr
            .unescape_value()
            .map_err(|err| invalid(file, &err.to_string()))?
            .into_owned();
        if key == "xmlns" {
            prefixes
                .entry(String::new())
                .or_insert_with(|| value.clone());
        } else if let Some(prefix) = key.strip_prefix("xmlns:") {
            prefixes
                .entry(prefix.to_owned())
                .or_insert_with(|| value.clone());
        }
        attrs.insert(key, value);
    }
    let raw = String::from_utf8_lossy(e.name().as_ref()).into_owned();
    let (prefix, local) = raw.split_once(':').unwrap_or(("", raw.as_str()));
    let is_xs = prefixes.get(prefix).map(String::as_str) == Some(XS);
    Ok(Node {
        local: if is_xs {
            local.to_owned()
        } else {
            format!("#{local}")
        },
        attrs,
        children: Vec::new(),
    })
}

fn close(node: Node, stack: &mut [Node], root: &mut Option<Node>) {
    match stack.last_mut() {
        Some(parent) => parent.children.push(node),
        None => *root = Some(node),
    }
}

fn line_at(bytes: &[u8], offset: usize) -> usize {
    1 + bytes[..offset.min(bytes.len())]
        .iter()
        .filter(|b| **b == b'\n')
        .count()
}

fn invalid(file: &str, message: &str) -> SchemaError {
    SchemaError::Invalid {
        file: file.to_owned(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{MaxOccurs, QName, SchemaError};

    fn schema(form: &str, body: &str) -> Vec<u8> {
        format!(
            r#"<xs:schema xmlns:xs="{XS}" xmlns:t="urn:t" xmlns:o="urn:o"
                 targetNamespace="urn:t" elementFormDefault="{form}">{body}</xs:schema>"#
        )
        .into_bytes()
    }

    fn t(local: &str) -> QName {
        QName::new("urn:t", local)
    }

    #[test]
    fn reads_a_global_element_with_its_type_and_substitution_group() {
        let raw = parse(
            &schema("qualified", r#"<xs:element name="Wall" type="t:WallType" substitutionGroup="o:_Surface" abstract="true"/>"#),
            "a.xsd",
        )
        .unwrap();
        assert_eq!(raw.target_namespace, "urn:t");
        assert_eq!(
            raw.elements,
            vec![RawElement {
                name: t("Wall"),
                type_name: Some(t("WallType")),
                substitution_group: Some(QName::new("urn:o", "_Surface")),
                is_abstract: true,
            }]
        );
    }

    #[test]
    fn reads_an_extension_base_and_its_sequence_with_occurrence_bounds() {
        let raw = parse(
            &schema(
                "qualified",
                r#"<xs:complexType name="B"><xs:complexContent><xs:extension base="t:A"><xs:sequence>
                     <xs:element name="own" type="xs:string" minOccurs="0"/>
                     <xs:element ref="o:hook" minOccurs="0" maxOccurs="unbounded"/>
                   </xs:sequence></xs:extension></xs:complexContent></xs:complexType>"#,
            ),
            "a.xsd",
        )
        .unwrap();
        let ty = &raw.complex_types[0];
        assert_eq!(ty.name, t("B"));
        assert_eq!(ty.base, Some(t("A")));
        assert_eq!(
            ty.particles,
            vec![
                Particle {
                    term: Term::Element(t("own")),
                    min: 0,
                    max: MaxOccurs::Bounded(1)
                },
                Particle {
                    term: Term::Element(QName::new("urn:o", "hook")),
                    min: 0,
                    max: MaxOccurs::Unbounded
                },
            ]
        );
    }

    #[test]
    fn a_restriction_restates_its_content_and_has_no_base() {
        let raw = parse(
            &schema(
                "qualified",
                r#"<xs:complexType name="R"><xs:complexContent><xs:restriction base="t:A"><xs:sequence>
                     <xs:element name="only" type="xs:string"/>
                   </xs:sequence></xs:restriction></xs:complexContent></xs:complexType>"#,
            ),
            "a.xsd",
        )
        .unwrap();
        assert_eq!(raw.complex_types[0].base, None);
        assert_eq!(raw.complex_types[0].particles.len(), 1);
    }

    #[test]
    fn simple_content_has_no_child_elements() {
        let raw = parse(
            &schema(
                "qualified",
                r#"<xs:complexType name="Code"><xs:simpleContent><xs:extension base="xs:string">
                     <xs:attribute name="codeSpace" type="xs:anyURI"/>
                   </xs:extension></xs:simpleContent></xs:complexType>"#,
            ),
            "a.xsd",
        )
        .unwrap();
        assert_eq!(raw.complex_types[0].base, None);
        assert!(raw.complex_types[0].particles.is_empty());
    }

    #[test]
    fn reads_choices_named_groups_and_group_references() {
        let raw = parse(
            &schema(
                "qualified",
                r#"<xs:group name="G"><xs:sequence><xs:element name="g1" type="xs:string"/></xs:sequence></xs:group>
                   <xs:complexType name="C"><xs:sequence>
                     <xs:group ref="t:G"/>
                     <xs:choice><xs:element name="x" type="xs:string"/><xs:element name="y" type="xs:string"/></xs:choice>
                   </xs:sequence></xs:complexType>"#,
            ),
            "a.xsd",
        )
        .unwrap();
        assert_eq!(raw.groups[0].name, t("G"));
        assert_eq!(raw.groups[0].particles[0].term, Term::Element(t("g1")));
        let particles = &raw.complex_types[0].particles;
        assert_eq!(particles[0].term, Term::GroupRef(t("G")));
        assert!(
            matches!(&particles[1].term, Term::Choice(alternatives) if alternatives.len() == 2)
        );
    }

    #[test]
    fn a_local_element_takes_the_target_namespace_only_when_qualified() {
        let body = r#"<xs:complexType name="C"><xs:sequence><xs:element name="p" type="xs:string"/></xs:sequence></xs:complexType>"#;
        let qualified = parse(&schema("qualified", body), "q.xsd").unwrap();
        let unqualified = parse(&schema("unqualified", body), "u.xsd").unwrap();
        assert_eq!(
            qualified.complex_types[0].particles[0].term,
            Term::Element(t("p"))
        );
        assert_eq!(
            unqualified.complex_types[0].particles[0].term,
            Term::Element(QName::new("", "p"))
        );
    }

    #[test]
    fn records_import_and_include_locations_in_order() {
        let raw = parse(
            &schema(
                "qualified",
                r#"<xs:import namespace="urn:o" schemaLocation="http://example.org/o.xsd"/>
                   <xs:include schemaLocation="part.xsd"/>"#,
            ),
            "a.xsd",
        )
        .unwrap();
        assert_eq!(raw.imports, vec!["http://example.org/o.xsd", "part.xsd"]);
    }

    #[test]
    fn ignores_annotation_content_from_other_namespaces() {
        let raw = parse(
            &schema(
                "qualified",
                r#"<xs:annotation><xs:appinfo><sch:rule xmlns:sch="urn:sch"><sch:element name="x"/></sch:rule></xs:appinfo></xs:annotation>
                   <xs:element name="Real" type="t:RealType"/>"#,
            ),
            "a.xsd",
        )
        .unwrap();
        assert_eq!(raw.elements.len(), 1);
    }

    #[test]
    fn an_undeclared_prefix_is_an_error_naming_the_file() {
        let err = parse(
            &schema("qualified", r#"<xs:element name="E" type="nope:T"/>"#),
            "a.xsd",
        )
        .unwrap_err();
        assert!(
            matches!(&err, SchemaError::UnboundPrefix { file, prefix, .. } if file == "a.xsd" && prefix == "nope"),
            "{err}"
        );
    }

    #[test]
    fn xs_all_is_reported_as_unsupported() {
        let err = parse(
            &schema("qualified", r#"<xs:complexType name="C"><xs:all><xs:element name="p" type="xs:string"/></xs:all></xs:complexType>"#),
            "a.xsd",
        )
        .unwrap_err();
        assert!(
            matches!(
                err,
                SchemaError::Unsupported {
                    construct: "xs:all",
                    ..
                }
            ),
            "{err}"
        );
    }

    #[test]
    fn malformed_xml_reports_the_line() {
        let bytes = format!("<xs:schema xmlns:xs=\"{XS}\">\n<xs:element name=\"E\">\n</xs:schema>")
            .into_bytes();
        let err = parse(&bytes, "bad.xsd").unwrap_err();
        assert!(
            matches!(&err, SchemaError::Xml { file, line, .. } if file == "bad.xsd" && *line >= 2),
            "{err}"
        );
    }
}
