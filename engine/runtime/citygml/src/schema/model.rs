//! Loads a set of XSD documents and compiles each complex type into an ordered
//! list of slots.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use super::bundle;
use super::xsd::{self, Particle, RawComplexType, RawElement, Term, XS};
use super::{MaxOccurs, QName, SchemaError, Wildcard};

/// Where a schema document comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Location {
    /// A key into the embedded bundle.
    Embedded(String),
    File(PathBuf),
}

impl Location {
    fn describe(&self) -> String {
        match self {
            Self::Embedded(key) => key.clone(),
            Self::File(path) => path.display().to_string(),
        }
    }

    fn read(&self) -> Result<Cow<'static, [u8]>, SchemaError> {
        match self {
            Self::Embedded(key) => {
                bundle::get(key)
                    .map(Cow::Borrowed)
                    .ok_or_else(|| SchemaError::Read {
                        file: key.clone(),
                        message: "not in the embedded bundle".to_owned(),
                    })
            }
            Self::File(path) => {
                std::fs::read(path)
                    .map(Cow::Owned)
                    .map_err(|e| SchemaError::Read {
                        file: path.display().to_string(),
                        message: e.to_string(),
                    })
            }
        }
    }

    /// Where `location`, written in this document, points. A URL resolves only
    /// to the bundle; a relative location resolves against this document.
    fn resolve(&self, location: &str) -> Option<Location> {
        if let Some(key) = bundle::key_for_url(location) {
            return bundle::get(&key).map(|_| Location::Embedded(key));
        }
        match self {
            Self::Embedded(key) => {
                let joined = normalize(
                    &Path::new(key)
                        .parent()
                        .unwrap_or(Path::new(""))
                        .join(location),
                );
                let key = joined.to_string_lossy().replace('\\', "/");
                bundle::get(&key).map(|_| Location::Embedded(key))
            }
            Self::File(path) => {
                let joined = normalize(&path.parent().unwrap_or(Path::new("")).join(location));
                joined.exists().then_some(Location::File(joined))
            }
        }
    }
}

/// Resolve `.` and `..` without touching the file system. A `..` that runs
/// past the start of a relative path is kept; past the root it stays at the root.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => match out.components().next_back() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                Some(Component::RootDir | Component::Prefix(_)) => {}
                _ => out.push(".."),
            },
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The CityGML 2.0 modules: every bundled document under `citygml/*/2.0/`.
fn core_entry_points() -> Vec<Location> {
    bundle::keys()
        .filter(|key| key.starts_with("schemas.opengis.net/citygml/") && key.contains("/2.0/"))
        .map(|key| Location::Embedded(key.to_owned()))
        .collect()
}

static CORE: LazyLock<Result<SchemaSet, SchemaError>> =
    LazyLock::new(|| SchemaSet::load(core_entry_points()));

/// A compiled set of schemas.
#[derive(Debug)]
pub struct SchemaSet {
    elements: HashMap<QName, RawElement>,
    classes: HashMap<QName, ClassModel>,
}

impl SchemaSet {
    /// The embedded CityGML 2.0 set, compiled once per process.
    pub fn core() -> Result<&'static SchemaSet, &'static SchemaError> {
        CORE.as_ref()
    }

    /// The embedded CityGML 2.0 set plus ADE schema documents from disk.
    pub fn with_ade(paths: &[PathBuf]) -> Result<SchemaSet, SchemaError> {
        let mut entries = core_entry_points();
        entries.extend(paths.iter().map(|path| Location::File(normalize(path))));
        Self::load(entries)
    }

    /// The content model of a global element's type.
    pub fn class_for_element(&self, element: &QName) -> Option<&ClassModel> {
        let type_name = self.elements.get(element)?.type_name.as_ref()?;
        self.classes.get(type_name)
    }

    /// The head a global element substitutes for, if any.
    pub fn substitution_group(&self, element: &QName) -> Option<&QName> {
        self.elements.get(element)?.substitution_group.as_ref()
    }

    /// Every global element in the set.
    pub fn global_elements(&self) -> impl Iterator<Item = &QName> {
        self.elements.keys()
    }

    fn load(entries: Vec<Location>) -> Result<SchemaSet, SchemaError> {
        let mut seen = HashSet::new();
        let mut queue: VecDeque<Location> = entries.into();
        let mut schemas = Vec::new();
        while let Some(location) = queue.pop_front() {
            if !seen.insert(location.clone()) {
                continue;
            }
            let raw = xsd::parse(&location.read()?, &location.describe())?;
            for import in &raw.imports {
                let next = location
                    .resolve(import)
                    .ok_or_else(|| SchemaError::MissingImport {
                        from: location.describe(),
                        location: import.clone(),
                    })?;
                queue.push_back(next);
            }
            schemas.push((location.describe(), raw));
        }
        Compiler::new(schemas).compile()
    }
}

/// A complex type's content: an ordered list of slots.
#[derive(Debug, Clone)]
pub struct ClassModel {
    slots: Vec<Slot>,
}

impl ClassModel {
    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    /// The first slot accepting `child`, for ordering.
    pub fn slot_index(&self, child: &QName) -> Option<usize> {
        self.slots.iter().position(|slot| slot.accepts(child))
    }

    /// Where `child` goes after a sibling placed at `after`: the first slot at
    /// or after it that accepts `child`. A child only an earlier slot accepts
    /// is out of order; one no slot accepts is unknown.
    pub fn place(&self, child: &QName, after: Option<usize>) -> Placement {
        let start = after.unwrap_or(0);
        if let Some(offset) = self.slots[start.min(self.slots.len())..]
            .iter()
            .position(|slot| slot.accepts(child))
        {
            return Placement::Slot(start + offset);
        }
        match self.slot_index(child) {
            Some(index) => Placement::OutOfOrder(index),
            None => Placement::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Slot(usize),
    OutOfOrder(usize),
    Unknown,
}

/// One position in a content model.
#[derive(Debug, Clone)]
pub struct Slot {
    /// The element the schema declares here; `None` for a choice, a nested
    /// sequence or `xs:any`.
    pub declared: Option<QName>,
    pub accepts: SlotAccepts,
    pub min: u32,
    pub max: MaxOccurs,
}

/// What a slot accepts: some elements by name, whatever its wildcards permit,
/// or both when a choice or nested sequence mixes the two.
#[derive(Debug, Clone, Default)]
pub struct SlotAccepts {
    /// The elements the slot names and everything that substitutes for them.
    pub elements: HashSet<QName>,
    /// The `xs:any` wildcards in the slot, each resolved against the schema
    /// that declared it.
    pub wildcards: Vec<Wildcard>,
}

impl SlotAccepts {
    pub fn contains(&self, child: &QName) -> bool {
        self.elements.contains(child) || self.wildcards.iter().any(|w| w.accepts(child))
    }
}

impl Slot {
    pub fn accepts(&self, child: &QName) -> bool {
        self.accepts.contains(child)
    }
}

struct Compiler {
    elements: HashMap<QName, RawElement>,
    types: HashMap<QName, (RawComplexType, String)>,
    groups: HashMap<QName, (Vec<Particle>, String)>,
    substitutes: HashMap<QName, Vec<QName>>,
}

impl Compiler {
    fn new(schemas: Vec<(String, xsd::RawSchema)>) -> Self {
        let mut compiler = Compiler {
            elements: HashMap::new(),
            types: HashMap::new(),
            groups: HashMap::new(),
            substitutes: HashMap::new(),
        };
        for (file, raw) in schemas {
            for element in raw.elements {
                if let Some(head) = &element.substitution_group {
                    compiler
                        .substitutes
                        .entry(head.clone())
                        .or_default()
                        .push(element.name.clone());
                }
                compiler.elements.insert(element.name.clone(), element);
            }
            for ty in raw.complex_types {
                compiler.types.insert(ty.name.clone(), (ty, file.clone()));
            }
            for group in raw.groups {
                compiler
                    .groups
                    .insert(group.name, (group.particles, file.clone()));
            }
        }
        compiler
    }

    fn compile(self) -> Result<SchemaSet, SchemaError> {
        let mut classes = HashMap::new();
        let mut names: Vec<&QName> = self.types.keys().collect();
        names.sort();
        for name in names {
            let content = self.content(name, &mut Vec::new())?;
            let slots = content.iter().map(|particle| self.slot(particle)).collect();
            classes.insert(name.clone(), ClassModel { slots });
        }
        Ok(SchemaSet {
            elements: self.elements,
            classes,
        })
    }

    /// A type's particles in document order: inherited ones first, groups inlined.
    fn content(&self, name: &QName, stack: &mut Vec<QName>) -> Result<Vec<Particle>, SchemaError> {
        let Some((ty, file)) = self.types.get(name) else {
            return Ok(Vec::new());
        };
        if stack.contains(name) {
            return Err(SchemaError::Cycle {
                kind: "type",
                name: name.to_string(),
            });
        }
        stack.push(name.clone());
        let mut out = match &ty.base {
            Some(base) if self.types.contains_key(base) => self.content(base, stack)?,
            // `xs:anyType` and the built-in types add no child elements.
            Some(base) if base.namespace == XS => Vec::new(),
            Some(base) => {
                return Err(SchemaError::Unresolved {
                    from: file.clone(),
                    kind: "base type",
                    name: base.to_string(),
                })
            }
            None => Vec::new(),
        };
        out.extend(self.expand(&ty.particles, file, &mut Vec::new())?);
        stack.pop();
        Ok(out)
    }

    fn expand(
        &self,
        particles: &[Particle],
        file: &str,
        groups: &mut Vec<QName>,
    ) -> Result<Vec<Particle>, SchemaError> {
        let mut out = Vec::new();
        for particle in particles {
            match &particle.term {
                Term::GroupRef(name) => {
                    let (body, group_file) =
                        self.groups
                            .get(name)
                            .ok_or_else(|| SchemaError::Unresolved {
                                from: file.to_owned(),
                                kind: "group",
                                name: name.to_string(),
                            })?;
                    if groups.contains(name) {
                        return Err(SchemaError::Cycle {
                            kind: "group",
                            name: name.to_string(),
                        });
                    }
                    groups.push(name.clone());
                    let mut body = self.expand(body, group_file, groups)?;
                    groups.pop();
                    // The reference's own occurrence bounds must survive the
                    // inlining; same rule as `xsd.rs` applies to a sequence.
                    if (particle.min, particle.max) == (1, MaxOccurs::Bounded(1)) {
                        out.extend(body);
                    } else if body.len() == 1
                        && (body[0].min, body[0].max) == (1, MaxOccurs::Bounded(1))
                    {
                        let mut only = body.remove(0);
                        only.min = particle.min;
                        only.max = particle.max;
                        out.push(only);
                    } else {
                        out.push(Particle {
                            term: Term::Sequence(body),
                            min: particle.min,
                            max: particle.max,
                        });
                    }
                }
                Term::Sequence(inner) => out.push(Particle {
                    term: Term::Sequence(self.expand(inner, file, groups)?),
                    ..particle.clone()
                }),
                Term::Choice(inner) => out.push(Particle {
                    term: Term::Choice(self.expand(inner, file, groups)?),
                    ..particle.clone()
                }),
                Term::Element(_) | Term::Any(_) => out.push(particle.clone()),
            }
        }
        Ok(out)
    }

    fn slot(&self, particle: &Particle) -> Slot {
        let declared = match &particle.term {
            Term::Element(name) => Some(name.clone()),
            _ => None,
        };
        let mut accepts = SlotAccepts::default();
        self.collect(&particle.term, &mut accepts);
        Slot {
            declared,
            accepts,
            min: particle.min,
            max: particle.max,
        }
    }

    /// Everything `term` accepts, at any depth of nesting.
    fn collect(&self, term: &Term, accepts: &mut SlotAccepts) {
        match term {
            Term::Element(name) => self.close(name, &mut accepts.elements),
            Term::Any(wildcard) => {
                if !accepts.wildcards.contains(wildcard) {
                    accepts.wildcards.push(wildcard.clone());
                }
            }
            Term::Sequence(inner) | Term::Choice(inner) => {
                for particle in inner {
                    self.collect(&particle.term, accepts);
                }
            }
            // Groups are inlined by `expand`.
            Term::GroupRef(_) => {}
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::schema::xsd::XS;

    #[test]
    fn normalize_keeps_a_parent_step_that_leaves_a_relative_path() {
        let n = |s: &str| normalize(Path::new(s));
        assert_eq!(n("a/../../b.xsd"), Path::new("../b.xsd"));
        assert_eq!(n("../iur/urf.xsd"), Path::new("../iur/urf.xsd"));
        assert_eq!(n("../../x.xsd"), Path::new("../../x.xsd"));
        assert_eq!(n("a/./b/../c.xsd"), Path::new("a/c.xsd"));
    }

    #[test]
    fn normalize_stops_a_parent_step_at_the_root() {
        assert_eq!(normalize(Path::new("/x/../../y.xsd")), Path::new("/y.xsd"));
    }

    /// Write `files` into a temporary directory and load the first one, which
    /// may import the rest by relative location.
    fn load(files: &[(&str, &str)]) -> Result<SchemaSet, SchemaError> {
        let dir = tempfile::tempdir().unwrap();
        for (name, body) in files {
            fs::write(
                dir.path().join(name),
                format!(
                    r#"<xs:schema xmlns:xs="{XS}" xmlns:t="urn:t" targetNamespace="urn:t" elementFormDefault="qualified">{body}</xs:schema>"#
                ),
            )
            .unwrap();
        }
        SchemaSet::load(vec![Location::File(dir.path().join(files[0].0))])
    }

    fn t(local: &str) -> QName {
        QName::new("urn:t", local)
    }

    fn declared(class: &ClassModel) -> Vec<String> {
        class
            .slots()
            .iter()
            .map(|s| s.declared.as_ref().map_or("?".into(), |q| q.local.clone()))
            .collect()
    }

    const BASE: &str = r#"
        <xs:element name="hook" abstract="true"/>
        <xs:complexType name="AType"><xs:sequence>
          <xs:element name="a1" type="xs:string"/>
          <xs:element ref="t:hook" minOccurs="0" maxOccurs="unbounded"/>
        </xs:sequence></xs:complexType>
        <xs:complexType name="BType"><xs:complexContent><xs:extension base="t:AType"><xs:sequence>
          <xs:element name="b1" type="xs:string"/>
        </xs:sequence></xs:extension></xs:complexContent></xs:complexType>
        <xs:element name="B" type="t:BType"/>"#;

    #[test]
    fn inherited_slots_come_before_the_types_own() {
        let set = load(&[("a.xsd", BASE)]).unwrap();
        assert_eq!(
            declared(set.class_for_element(&t("B")).unwrap()),
            ["a1", "hook", "b1"]
        );
    }

    #[test]
    fn a_substitution_chain_is_accepted_at_the_heads_slot() {
        let set = load(&[(
            "a.xsd",
            &format!(
                r#"{BASE}<xs:element name="mid" substitutionGroup="t:hook" abstract="true"/>
                        <xs:element name="leaf" type="xs:string" substitutionGroup="t:mid"/>"#
            ),
        )])
        .unwrap();
        let class = set.class_for_element(&t("B")).unwrap();
        assert_eq!(class.slot_index(&t("leaf")), Some(1));
        assert_eq!(class.slot_index(&t("mid")), Some(1));
    }

    #[test]
    fn named_groups_are_inlined_in_order() {
        let set = load(&[(
            "a.xsd",
            r#"<xs:group name="G"><xs:sequence><xs:element name="g1" type="xs:string"/><xs:element name="g2" type="xs:string"/></xs:sequence></xs:group>
               <xs:complexType name="CType"><xs:sequence><xs:element name="c0" type="xs:string"/><xs:group ref="t:G"/><xs:element name="c3" type="xs:string"/></xs:sequence></xs:complexType>
               <xs:element name="C" type="t:CType"/>"#,
        )])
        .unwrap();
        assert_eq!(
            declared(set.class_for_element(&t("C")).unwrap()),
            ["c0", "g1", "g2", "c3"]
        );
    }

    #[test]
    fn a_group_reference_keeps_its_own_occurrence_bounds() {
        let set = load(&[(
            "a.xsd",
            r#"<xs:group name="One"><xs:sequence><xs:element name="o1" type="xs:string"/></xs:sequence></xs:group>
               <xs:group name="Two"><xs:sequence><xs:element name="w1" type="xs:string"/><xs:element name="w2" type="xs:string"/></xs:sequence></xs:group>
               <xs:complexType name="CType"><xs:sequence>
                 <xs:group ref="t:One" minOccurs="0"/>
                 <xs:group ref="t:Two" minOccurs="0" maxOccurs="unbounded"/>
               </xs:sequence></xs:complexType>
               <xs:element name="C" type="t:CType"/>"#,
        )])
        .unwrap();
        let class = set.class_for_element(&t("C")).unwrap();
        assert_eq!(class.slots().len(), 2);
        assert_eq!(class.slots()[0].min, 0);
        assert_eq!(class.slots()[0].max, MaxOccurs::Bounded(1));
        assert_eq!(class.slot_index(&t("o1")), Some(0));
        let many = &class.slots()[1];
        assert_eq!((many.min, many.max), (0, MaxOccurs::Unbounded));
        assert_eq!(class.slot_index(&t("w1")), Some(1));
        assert_eq!(class.slot_index(&t("w2")), Some(1));
    }

    #[test]
    fn a_restriction_does_not_inherit_its_bases_slots() {
        let set = load(&[(
            "a.xsd",
            &format!(r#"{BASE}<xs:complexType name="RType"><xs:complexContent><xs:restriction base="t:AType"><xs:sequence>
                          <xs:element name="a1" type="xs:string"/></xs:sequence></xs:restriction></xs:complexContent></xs:complexType>
                        <xs:element name="R" type="t:RType"/>"#),
        )])
        .unwrap();
        assert_eq!(declared(set.class_for_element(&t("R")).unwrap()), ["a1"]);
    }

    #[test]
    fn place_takes_the_first_acceptable_slot_after_the_previous_child() {
        let set = load(&[(
            "a.xsd",
            r#"<xs:complexType name="DType"><xs:sequence>
                 <xs:element name="x" type="xs:string" minOccurs="0"/>
                 <xs:element name="y" type="xs:string"/>
                 <xs:element name="x" type="xs:string" minOccurs="0"/>
               </xs:sequence></xs:complexType>
               <xs:element name="D" type="t:DType"/>"#,
        )])
        .unwrap();
        let class = set.class_for_element(&t("D")).unwrap();
        assert_eq!(class.place(&t("x"), None), Placement::Slot(0));
        assert_eq!(class.place(&t("x"), Some(1)), Placement::Slot(2));
        assert_eq!(class.place(&t("y"), Some(2)), Placement::OutOfOrder(1));
        assert_eq!(class.place(&t("zzz"), None), Placement::Unknown);
    }

    #[test]
    fn any_accepts_by_its_namespace_constraint() {
        let set = load(&[(
            "a.xsd",
            r###"<xs:complexType name="EType"><xs:sequence><xs:any namespace="##other" minOccurs="0"/></xs:sequence></xs:complexType>
               <xs:element name="E" type="t:EType"/>"###,
        )])
        .unwrap();
        let class = set.class_for_element(&t("E")).unwrap();
        assert_eq!(class.slot_index(&QName::new("urn:elsewhere", "x")), Some(0));
        assert_eq!(class.slot_index(&t("x")), None);
    }

    #[test]
    fn relative_imports_resolve_from_the_importing_file() {
        let set = load(&[
            ("main.xsd", r#"<xs:include schemaLocation="part.xsd"/><xs:element name="P" type="t:PType"/>"#),
            ("part.xsd", r#"<xs:complexType name="PType"><xs:sequence><xs:element name="p1" type="xs:string"/></xs:sequence></xs:complexType>"#),
        ])
        .unwrap();
        assert_eq!(declared(set.class_for_element(&t("P")).unwrap()), ["p1"]);
    }

    #[test]
    fn a_missing_import_is_an_error_naming_the_location() {
        let err = load(&[("a.xsd", r#"<xs:include schemaLocation="gone.xsd"/>"#)]).unwrap_err();
        assert!(
            matches!(&err, SchemaError::MissingImport { location, .. } if location == "gone.xsd"),
            "{err}"
        );
    }

    #[test]
    fn an_unresolved_group_is_an_error_naming_the_group() {
        let err = load(&[(
            "a.xsd",
            r#"<xs:complexType name="CType"><xs:sequence><xs:group ref="t:Nope"/></xs:sequence></xs:complexType>"#,
        )])
        .unwrap_err();
        assert!(
            matches!(&err, SchemaError::Unresolved { kind: "group", name, .. } if name.ends_with("Nope")),
            "{err}"
        );
    }

    #[test]
    fn a_derivation_cycle_is_an_error() {
        let err = load(&[(
            "a.xsd",
            r#"<xs:complexType name="X"><xs:complexContent><xs:extension base="t:Y"/></xs:complexContent></xs:complexType>
               <xs:complexType name="Y"><xs:complexContent><xs:extension base="t:X"/></xs:complexContent></xs:complexType>"#,
        )])
        .unwrap_err();
        assert!(
            matches!(err, SchemaError::Cycle { kind: "type", .. }),
            "{err}"
        );
    }

    #[test]
    fn an_element_whose_type_contains_itself_is_fine() {
        let set = load(&[(
            "a.xsd",
            r#"<xs:complexType name="PartType"><xs:sequence><xs:element ref="t:Part" minOccurs="0" maxOccurs="unbounded"/></xs:sequence></xs:complexType>
               <xs:element name="Part" type="t:PartType"/>"#,
        )])
        .unwrap();
        assert_eq!(
            set.class_for_element(&t("Part"))
                .unwrap()
                .slot_index(&t("Part")),
            Some(0)
        );
    }

    #[test]
    fn a_choice_mixing_elements_and_a_wildcard_accepts_both() {
        let set = load(&[(
            "a.xsd",
            r###"<xs:element name="a" type="xs:string"/>
               <xs:element name="a2" type="xs:string" substitutionGroup="t:a"/>
               <xs:complexType name="FType"><xs:sequence>
                 <xs:choice><xs:element ref="t:a"/><xs:any namespace="##other"/></xs:choice>
               </xs:sequence></xs:complexType>
               <xs:element name="F" type="t:FType"/>"###,
        )])
        .unwrap();
        let class = set.class_for_element(&t("F")).unwrap();
        assert_eq!(class.slots().len(), 1);
        assert_eq!(class.slot_index(&t("a")), Some(0));
        assert_eq!(class.slot_index(&t("a2")), Some(0));
        assert_eq!(class.slot_index(&QName::new("urn:elsewhere", "x")), Some(0));
        assert_eq!(class.slot_index(&t("b")), None);
        assert_eq!(class.place(&t("b"), None), Placement::Unknown);
    }

    /// Write whole schema documents into `dir`, returning their paths in order.
    fn write_docs(dir: &Path, docs: &[(&str, &str)]) -> Vec<PathBuf> {
        docs.iter()
            .map(|(name, doc)| {
                let path = dir.join(name);
                fs::write(&path, doc).unwrap();
                path
            })
            .collect()
    }

    #[test]
    fn an_inherited_other_wildcard_is_judged_against_the_namespace_that_declared_it() {
        let dir = tempfile::tempdir().unwrap();
        let paths = write_docs(
            dir.path(),
            &[
                (
                    "x.xsd",
                    &format!(
                        r###"<xs:schema xmlns:xs="{XS}" targetNamespace="urn:x" elementFormDefault="qualified">
                               <xs:complexType name="BaseType"><xs:sequence>
                                 <xs:any namespace="##other" minOccurs="0" maxOccurs="unbounded"/>
                               </xs:sequence></xs:complexType>
                             </xs:schema>"###
                    ),
                ),
                (
                    "y.xsd",
                    &format!(
                        r#"<xs:schema xmlns:xs="{XS}" xmlns:x="urn:x" xmlns:y="urn:y" targetNamespace="urn:y" elementFormDefault="qualified">
                             <xs:import namespace="urn:x" schemaLocation="x.xsd"/>
                             <xs:complexType name="DerivedType"><xs:complexContent><xs:extension base="x:BaseType"/></xs:complexContent></xs:complexType>
                             <xs:element name="Derived" type="y:DerivedType"/>
                           </xs:schema>"#
                    ),
                ),
            ],
        );
        let set = SchemaSet::load(vec![Location::File(paths[1].clone())]).unwrap();
        let class = set
            .class_for_element(&QName::new("urn:y", "Derived"))
            .unwrap();
        assert_eq!(class.slot_index(&QName::new("urn:y", "own")), Some(0));
        assert_eq!(class.slot_index(&QName::new("urn:x", "base")), None);
    }
}
