//! Loads a set of XSD documents and compiles each complex type into an ordered
//! list of slots.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use fastxml::schema::xsd::{compile_schemas, parse_xsd_ast, register_builtin_types, XsdSchema};
use url::Url;

use super::bundle;
use super::slots::{self, ElementInfo, MAX_DEPTH};
use super::{MaxOccurs, QName, SchemaError, Wildcard};

/// Reads the document at a URL, for the schemas the crate cannot read itself.
type Reader<'a> = &'a dyn Fn(&str) -> Result<Vec<u8>, String>;

/// An ADE schema document a caller's reader can fetch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdeSource {
    /// The target namespace the document must declare.
    pub namespace: String,
    /// An absolute URL the reader understands, such as `gs://` or `file://`.
    pub location: String,
}

/// Where a schema document comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Location {
    /// A key into the embedded bundle.
    Embedded(String),
    File(PathBuf),
    /// An absolute URL, read only through a caller's reader.
    Remote(String),
}

impl Location {
    fn describe(&self) -> String {
        match self {
            Self::Embedded(key) => key.clone(),
            Self::File(path) => path.display().to_string(),
            Self::Remote(url) => url.clone(),
        }
    }

    fn read(&self, reader: Option<Reader>) -> Result<Cow<'static, [u8]>, SchemaError> {
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
            Self::Remote(url) => {
                let read = reader.ok_or_else(|| SchemaError::Read {
                    file: url.clone(),
                    message: "no reader for remote schemas".to_owned(),
                })?;
                read(url)
                    .map(Cow::Owned)
                    .map_err(|message| SchemaError::Read {
                        file: url.clone(),
                        message,
                    })
            }
        }
    }

    /// Where `location`, written in this document, points. A URL resolves only
    /// to the bundle; a relative location resolves against this document.
    /// `SchemaSet::load` falls back to the namespace for an import neither finds.
    /// So a reader is asked only for its entry points and documents relative
    /// to them, never for an absolute URL a schema names.
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
            Self::Remote(base) => {
                if Url::parse(location).is_ok() {
                    return None;
                }
                let base = Url::parse(base).ok()?;
                let joined = base.join(location).ok()?;
                // A network-path reference (`//other/x.xsd`) joins to another
                // authority; it is not "beside" this document, so it is not read.
                (joined.scheme() == base.scheme()
                    && joined.host_str() == base.host_str()
                    && joined.port() == base.port()
                    && !matches!(joined.scheme(), "http" | "https"))
                .then(|| Location::Remote(joined.into()))
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
    LazyLock::new(|| SchemaSet::load(core_entry_points(), None));

/// A compiled set of schemas.
#[derive(Debug)]
pub struct SchemaSet {
    elements: HashMap<QName, ElementInfo>,
    classes: HashMap<QName, ClassModel>,
}

impl SchemaSet {
    /// The embedded CityGML 2.0 set, compiled once per process.
    pub fn core() -> Result<&'static SchemaSet, &'static SchemaError> {
        CORE.as_ref()
    }

    /// The embedded CityGML 2.0 set plus ADE schema documents from disk. An
    /// import whose location finds no document, typically an absolute URL to
    /// an ADE's published copy, is satisfied by any of `paths` (or any schema
    /// they load) whose target namespace it names; their order does not matter.
    pub fn with_ade(paths: &[PathBuf]) -> Result<SchemaSet, SchemaError> {
        let mut entries = core_entry_points();
        entries.extend(paths.iter().map(|path| Location::File(normalize(path))));
        Self::load(entries, None)
    }

    /// The embedded CityGML 2.0 set plus ADE schema documents fetched by
    /// `read`, which is given each source's location and the locations
    /// relative to it. An absolute import resolves as in [`Self::with_ade`],
    /// from the bundle or a loaded schema of its namespace, and is never read.
    pub fn with_ade_from(
        sources: &[AdeSource],
        read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    ) -> Result<SchemaSet, SchemaError> {
        let mut entries = core_entry_points();
        entries.extend(
            sources
                .iter()
                .map(|source| Location::Remote(source.location.clone())),
        );
        let documents = Self::parse(entries, Some(read))?;
        for source in sources {
            let location = Location::Remote(source.location.clone());
            let declared = documents
                .iter()
                .find(|(loaded, _)| loaded == &location)
                .map(|(_, ast)| ast.target_namespace.as_deref().unwrap_or_default());
            if let Some(declared) = declared.filter(|declared| *declared != source.namespace) {
                return Err(SchemaError::Parse {
                    file: source.location.clone(),
                    message: format!(
                        "declares namespace `{declared}`, not `{}`",
                        source.namespace
                    ),
                });
            }
        }
        Self::compile(documents.into_iter().map(|(_, ast)| ast).collect())
    }

    /// The content model of a global element's type.
    pub fn class_for_element(&self, element: &QName) -> Option<&ClassModel> {
        let type_name = self.elements.get(element)?.type_name.as_ref()?;
        self.classes.get(type_name)
    }

    /// The slots of `class`, by index and in schema order, whose property type
    /// has a slot naming `object` or an element that can stand for it: the
    /// properties that can hold it. A wildcard does not count, or every
    /// property typed `gml:MetaDataPropertyType` would hold anything. Empty for
    /// a class the set does not know.
    pub fn slots_holding(&self, class: &QName, object: &QName) -> Vec<usize> {
        let Some(class) = self.class_for_element(class) else {
            return Vec::new();
        };
        class
            .slots()
            .iter()
            .enumerate()
            .filter(|(_, slot)| {
                slot.property_type
                    .as_ref()
                    .and_then(|ty| self.classes.get(ty))
                    .is_some_and(|held| {
                        held.slots()
                            .iter()
                            .any(|inner| inner.accepts.elements.contains(object))
                    })
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// The head a global element substitutes for, if any.
    pub fn substitution_group(&self, element: &QName) -> Option<&QName> {
        self.elements.get(element)?.substitution_group.as_ref()
    }

    /// Every global element in the set.
    pub fn global_elements(&self) -> impl Iterator<Item = &QName> {
        self.elements.keys()
    }

    /// Whether a global element is declared abstract; `false` for one the set
    /// does not know.
    pub fn is_abstract(&self, element: &QName) -> bool {
        self.elements
            .get(element)
            .is_some_and(|info| info.is_abstract)
    }

    /// Whether `element` is `head` or substitutes for it through its chain of
    /// substitution groups.
    pub fn substitutes_for(&self, element: &QName, head: &QName) -> bool {
        let mut current = element;
        for _ in 0..MAX_DEPTH {
            if current == head {
                return true;
            }
            match self.substitution_group(current) {
                Some(next) => current = next,
                None => return false,
            }
        }
        current == head
    }

    fn load(entries: Vec<Location>, read: Option<Reader>) -> Result<SchemaSet, SchemaError> {
        let documents = Self::parse(entries, read)?;
        Self::compile(documents.into_iter().map(|(_, ast)| ast).collect())
    }

    /// Parse `entries` and every document they import, include or redefine,
    /// each paired with where it was read from.
    fn parse(
        entries: Vec<Location>,
        read: Option<Reader>,
    ) -> Result<Vec<(Location, XsdSchema)>, SchemaError> {
        let mut seen = HashSet::new();
        let mut queue: VecDeque<Location> = entries.into();
        let mut schemas = Vec::new();
        // Imports no location found, held until every entry is loaded so one
        // can be satisfied by an entry later in the list.
        let mut unlocated = Vec::new();
        while let Some(location) = queue.pop_front() {
            if !seen.insert(location.clone()) {
                continue;
            }
            let ast = parse_xsd_ast(&location.read(read)?).map_err(|e| SchemaError::Parse {
                file: location.describe(),
                message: e.to_string(),
            })?;
            // An include or redefine shares this schema's namespace, so only
            // an import can fall back to another document of its namespace.
            let imports = ast.imports.iter().filter_map(|import| {
                Some((
                    import.schema_location.as_deref()?,
                    import.namespace.as_ref(),
                ))
            });
            let same_namespace = ast
                .includes
                .iter()
                .map(|include| include.schema_location.as_str())
                .chain(
                    ast.redefines
                        .iter()
                        .map(|redefine| redefine.schema_location.as_str()),
                )
                .map(|href| (href, None));
            for (href, namespace) in imports.chain(same_namespace) {
                let missing = || SchemaError::MissingImport {
                    from: location.describe(),
                    location: href.to_owned(),
                };
                match (location.resolve(href), namespace) {
                    (Some(next), _) => queue.push_back(next),
                    (None, Some(namespace)) => unlocated.push((namespace.clone(), missing())),
                    (None, None) => return Err(missing()),
                }
            }
            schemas.push((location, ast));
        }
        let loaded: HashSet<&str> = schemas
            .iter()
            .map(|(_, ast)| ast.target_namespace.as_deref().unwrap_or_default())
            .collect();
        if let Some((_, err)) = unlocated
            .into_iter()
            .find(|(namespace, _)| !loaded.contains(namespace.as_str()))
        {
            return Err(err);
        }
        Ok(schemas)
    }

    fn compile(schemas: Vec<XsdSchema>) -> Result<SchemaSet, SchemaError> {
        let mut compiled = compile_schemas(schemas).map_err(|e| SchemaError::Compile {
            message: e.to_string(),
        })?;
        register_builtin_types(&mut compiled);
        let (elements, classes) = slots::build(&compiled)?;
        Ok(SchemaSet { elements, classes })
    }
}

/// A complex type's content: an ordered list of slots.
#[derive(Debug, Clone)]
pub struct ClassModel {
    pub(super) slots: Vec<Slot>,
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
    /// sequence, `xs:all` or `xs:any`.
    pub declared: Option<QName>,
    /// The type of the declared element: its own for a local declaration, the
    /// global element's for a reference. `None` where `declared` is, and for
    /// an inline type or a reference to an element with none.
    pub property_type: Option<QName>,
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use fastxml::schema::xsd::XSD_NAMESPACE as XS;

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
        SchemaSet::load(vec![Location::File(dir.path().join(files[0].0))], None)
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
            matches!(&err, SchemaError::Compile { message } if message.contains("Nope")),
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
        // fastxml rejects the cycle before the slot builder ever walks it.
        assert!(
            matches!(&err, SchemaError::Compile { message } if message.contains("'X'") || message.contains("'Y'")),
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

    #[test]
    fn an_all_group_is_one_slot() {
        let set = load(&[(
            "a.xsd",
            r#"<xs:complexType name="AllType"><xs:all><xs:element name="p" type="xs:string"/><xs:element name="q" type="xs:string"/></xs:all></xs:complexType>
               <xs:element name="A" type="t:AllType"/>"#,
        )])
        .unwrap();
        let class = set.class_for_element(&t("A")).unwrap();
        assert_eq!(class.slots().len(), 1);
        assert_eq!(class.slot_index(&t("q")), Some(0));
    }

    #[test]
    fn a_wildcard_list_resolves_target_and_local() {
        let set = load(&[(
            "a.xsd",
            r###"<xs:complexType name="WType"><xs:sequence><xs:any namespace="##targetNamespace ##local" maxOccurs="unbounded"/></xs:sequence></xs:complexType>
               <xs:element name="W" type="t:WType"/>"###,
        )])
        .unwrap();
        let class = set.class_for_element(&t("W")).unwrap();
        assert_eq!(class.slot_index(&t("anything")), Some(0));
        assert_eq!(class.slot_index(&QName::new("", "bare")), Some(0));
        assert_eq!(class.slot_index(&QName::new("urn:other", "x")), None);
    }

    #[test]
    fn an_optional_sequence_of_one_element_makes_that_element_optional() {
        let set = load(&[(
            "a.xsd",
            r#"<xs:element name="target" type="xs:string"/>
               <xs:complexType name="PType"><xs:sequence minOccurs="0"><xs:element ref="t:target"/></xs:sequence></xs:complexType>
               <xs:element name="P" type="t:PType"/>"#,
        )])
        .unwrap();
        let class = set.class_for_element(&t("P")).unwrap();
        assert_eq!(declared(class), ["target"]);
        assert_eq!(
            (class.slots()[0].min, class.slots()[0].max),
            (0, MaxOccurs::Bounded(1))
        );
    }

    #[test]
    fn an_extension_sequence_keeps_its_unbounded_maximum() {
        let set = load(&[(
            "a.xsd",
            &format!(
                r#"{BASE}<xs:complexType name="PType"><xs:complexContent><xs:extension base="t:AType">
                     <xs:sequence maxOccurs="unbounded"><xs:element name="p1" type="xs:string"/></xs:sequence>
                   </xs:extension></xs:complexContent></xs:complexType>
                   <xs:element name="P" type="t:PType"/>"#
            ),
        )])
        .unwrap();
        let class = set.class_for_element(&t("P")).unwrap();
        assert_eq!(declared(class), ["a1", "hook", "p1"]);
        assert_eq!(
            (class.slots()[2].min, class.slots()[2].max),
            (1, MaxOccurs::Unbounded)
        );
    }

    #[test]
    fn an_optional_sequence_of_several_elements_stays_one_slot() {
        let set = load(&[(
            "a.xsd",
            r#"<xs:complexType name="PType"><xs:sequence minOccurs="0">
                 <xs:element name="a" type="xs:string"/><xs:element name="b" type="xs:string"/>
               </xs:sequence></xs:complexType>
               <xs:element name="P" type="t:PType"/>"#,
        )])
        .unwrap();
        let class = set.class_for_element(&t("P")).unwrap();
        assert_eq!(declared(class), ["?"]);
        assert_eq!(
            (class.slots()[0].min, class.slots()[0].max),
            (0, MaxOccurs::Bounded(1))
        );
        assert_eq!(class.slot_index(&t("b")), Some(0));
    }

    #[test]
    fn a_redefined_type_replaces_the_original() {
        let set = load(&[
            (
                "a.xsd",
                r#"<xs:redefine schemaLocation="base.xsd">
                     <xs:complexType name="AType"><xs:complexContent><xs:extension base="t:AType"><xs:sequence>
                       <xs:element name="extra" type="xs:string"/>
                     </xs:sequence></xs:extension></xs:complexContent></xs:complexType>
                   </xs:redefine>"#,
            ),
            ("base.xsd", BASE),
        ])
        .unwrap();
        assert_eq!(
            declared(set.class_for_element(&t("B")).unwrap()),
            ["a1", "hook", "extra", "b1"]
        );
    }

    #[test]
    fn malformed_xml_is_an_error_naming_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.xsd");
        fs::write(&path, format!(r#"<xs:schema xmlns:xs="{XS}"><xs:element"#)).unwrap();
        let err = SchemaSet::load(vec![Location::File(path)], None).unwrap_err();
        assert!(
            matches!(&err, SchemaError::Parse { file, .. } if file.ends_with("broken.xsd")),
            "{err}"
        );
    }

    #[test]
    fn xs_override_is_an_error() {
        let err = load(&[
            ("a.xsd", r#"<xs:override schemaLocation="base.xsd"/>"#),
            ("base.xsd", BASE),
        ])
        .unwrap_err();
        assert!(
            matches!(&err, SchemaError::Compile { message } if message.contains("xs:override")),
            "{err}"
        );
    }

    #[test]
    fn a_two_step_extension_chain_orders_from_the_root() {
        let set = load(&[
            (
                "c.xsd",
                r#"<xs:include schemaLocation="root.xsd"/>
                   <xs:complexType name="CType"><xs:complexContent><xs:extension base="t:BType"><xs:sequence>
                     <xs:element name="c1" type="xs:string"/>
                   </xs:sequence></xs:extension></xs:complexContent></xs:complexType>
                   <xs:element name="C" type="t:CType"/>"#,
            ),
            ("root.xsd", BASE),
        ])
        .unwrap();
        assert_eq!(
            declared(set.class_for_element(&t("C")).unwrap()),
            ["a1", "hook", "b1", "c1"]
        );
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
        let set = SchemaSet::load(vec![Location::File(paths[1].clone())], None).unwrap();
        let class = set
            .class_for_element(&QName::new("urn:y", "Derived"))
            .unwrap();
        assert_eq!(class.slot_index(&QName::new("urn:y", "own")), Some(0));
        assert_eq!(class.slot_index(&QName::new("urn:x", "base")), None);
    }

    /// `a.xsd` imports `b.xsd`'s namespace from an absolute URL that resolves
    /// nowhere, and extends a type from it.
    fn absolute_import_pair(dir: &Path) -> (PathBuf, PathBuf) {
        let paths = write_docs(
            dir,
            &[
                (
                    "a.xsd",
                    &format!(
                        r#"<xs:schema xmlns:xs="{XS}" xmlns:a="urn:a" xmlns:b="urn:b" targetNamespace="urn:a" elementFormDefault="qualified">
                             <xs:import namespace="urn:b" schemaLocation="https://example.com/b.xsd"/>
                             <xs:complexType name="AType"><xs:complexContent><xs:extension base="b:BType"><xs:sequence>
                               <xs:element name="a1" type="xs:string"/>
                             </xs:sequence></xs:extension></xs:complexContent></xs:complexType>
                             <xs:element name="A" type="a:AType"/>
                           </xs:schema>"#
                    ),
                ),
                (
                    "b.xsd",
                    &format!(
                        r#"<xs:schema xmlns:xs="{XS}" xmlns:b="urn:b" targetNamespace="urn:b" elementFormDefault="qualified">
                             <xs:element name="item" type="xs:string"/>
                             <xs:complexType name="BType"><xs:sequence><xs:element ref="b:item"/></xs:sequence></xs:complexType>
                           </xs:schema>"#
                    ),
                ),
            ],
        );
        (paths[0].clone(), paths[1].clone())
    }

    #[test]
    fn an_absolute_import_is_satisfied_by_a_supplied_schema_of_its_namespace_in_either_order() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = absolute_import_pair(dir.path());
        for paths in [[a.clone(), b.clone()], [b, a]] {
            let set = SchemaSet::with_ade(&paths).unwrap();
            let class = set.class_for_element(&QName::new("urn:a", "A")).unwrap();
            assert_eq!(declared(class), ["item", "a1"]);
            assert_eq!(class.slot_index(&QName::new("urn:b", "item")), Some(0));
        }
    }

    #[test]
    fn an_absolute_import_with_no_supplied_schema_of_its_namespace_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let (a, _) = absolute_import_pair(dir.path());
        let err = SchemaSet::with_ade(&[a]).unwrap_err();
        assert!(
            matches!(&err, SchemaError::MissingImport { location, .. } if location == "https://example.com/b.xsd"),
            "{err}"
        );
    }

    fn reader(
        files: &[(&'static str, &'static str)],
    ) -> impl Fn(&str) -> Result<Vec<u8>, String> + use<> {
        let files: Vec<(String, String)> = files
            .iter()
            .map(|(u, b)| (u.to_string(), b.to_string()))
            .collect();
        move |url: &str| {
            files
                .iter()
                .find(|(u, _)| u == url)
                .map(|(_, b)| b.clone().into_bytes())
                .ok_or_else(|| format!("unexpected read of {url}"))
        }
    }

    const ADE: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:a="urn:ade"
        xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
        targetNamespace="urn:ade" elementFormDefault="qualified">
      <xs:import namespace="http://www.opengis.net/citygml/building/2.0"
                 schemaLocation="http://schemas.opengis.net/citygml/building/2.0/building.xsd"/>
      <xs:include schemaLocation="part.xsd"/>
      <xs:element name="Hall" type="bldg:BuildingType" substitutionGroup="bldg:_AbstractBuilding"/>
    </xs:schema>"#;
    const PART: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:ade" elementFormDefault="qualified">
      <xs:element name="note" type="xs:string"/>
    </xs:schema>"#;

    #[test]
    fn an_ade_reads_relative_imports_through_the_reader_and_never_absolute_ones() {
        let read = reader(&[("gs://b/ade/ade.xsd", ADE), ("gs://b/ade/part.xsd", PART)]);
        let set = SchemaSet::with_ade_from(
            &[AdeSource {
                namespace: "urn:ade".into(),
                location: "gs://b/ade/ade.xsd".into(),
            }],
            &read,
        )
        .unwrap();
        let hall = QName::new("urn:ade", "Hall");
        assert!(set.class_for_element(&hall).is_some());
        assert!(set
            .global_elements()
            .any(|q| q == &QName::new("urn:ade", "note")));
        let city_object = QName::new("http://www.opengis.net/citygml/2.0", "_CityObject");
        assert!(set.substitutes_for(&hall, &city_object));
        assert!(!set.is_abstract(&hall));
        assert!(set.is_abstract(&QName::new(
            "http://www.opengis.net/citygml/building/2.0",
            "_AbstractBuilding"
        )));
    }

    #[test]
    fn an_absolute_import_nothing_supplies_is_missing() {
        let ade = ADE.replace(
            r#"<xs:include schemaLocation="part.xsd"/>"#,
            r#"<xs:import namespace="urn:elsewhere" schemaLocation="https://example.com/x.xsd"/>"#,
        );
        let ade: &'static str = Box::leak(ade.into_boxed_str());
        let read = reader(&[("gs://b/ade/ade.xsd", ade)]);
        let err = SchemaSet::with_ade_from(
            &[AdeSource {
                namespace: "urn:ade".into(),
                location: "gs://b/ade/ade.xsd".into(),
            }],
            &read,
        )
        .unwrap_err();
        assert!(matches!(err, SchemaError::MissingImport { .. }), "{err}");
    }

    #[test]
    fn an_absolute_import_in_the_readers_own_scheme_is_not_read() {
        let ade = ADE.replace(
            r#"<xs:include schemaLocation="part.xsd"/>"#,
            r#"<xs:import namespace="urn:elsewhere" schemaLocation="gs://b/other/x.xsd"/>"#,
        );
        let ade: &'static str = Box::leak(ade.into_boxed_str());
        let other = PART.replace("urn:ade", "urn:elsewhere");
        let other: &'static str = Box::leak(other.into_boxed_str());
        let read = reader(&[("gs://b/ade/ade.xsd", ade), ("gs://b/other/x.xsd", other)]);
        let err = SchemaSet::with_ade_from(
            &[AdeSource {
                namespace: "urn:ade".into(),
                location: "gs://b/ade/ade.xsd".into(),
            }],
            &read,
        )
        .unwrap_err();
        assert!(
            matches!(&err, SchemaError::MissingImport { location, .. } if location == "gs://b/other/x.xsd"),
            "{err}"
        );
    }

    #[test]
    fn a_network_path_import_is_not_read_from_another_authority() {
        let ade = ADE.replace(
            r#"<xs:include schemaLocation="part.xsd"/>"#,
            r#"<xs:import namespace="urn:elsewhere" schemaLocation="//other/x.xsd"/>"#,
        );
        let ade: &'static str = Box::leak(ade.into_boxed_str());
        let other = PART.replace("urn:ade", "urn:elsewhere");
        let other: &'static str = Box::leak(other.into_boxed_str());
        let read = reader(&[("gs://b/ade/ade.xsd", ade), ("gs://other/x.xsd", other)]);
        let err = SchemaSet::with_ade_from(
            &[AdeSource {
                namespace: "urn:ade".into(),
                location: "gs://b/ade/ade.xsd".into(),
            }],
            &read,
        )
        .unwrap_err();
        assert!(matches!(err, SchemaError::MissingImport { .. }), "{err}");
    }

    #[test]
    fn a_source_whose_namespace_does_not_match_its_document_is_rejected() {
        let read = reader(&[("gs://b/ade/ade.xsd", ADE), ("gs://b/ade/part.xsd", PART)]);
        let err = SchemaSet::with_ade_from(
            &[AdeSource {
                namespace: "urn:wrong".into(),
                location: "gs://b/ade/ade.xsd".into(),
            }],
            &read,
        )
        .unwrap_err();
        assert!(err.to_string().contains("urn:wrong"), "{err}");
    }
}
