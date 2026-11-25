//! The two checks of the xlink:hrefs, in this order:
//!
//! 1. Referent existence (xlink参照先なし): the xlink:href refers to an
//!    element of the member, of the external GML file it names, or, for a
//!    city object group, of the file.
//! 2. T03 (association target type, 関連相手先の正しさ): the referent the
//!    first check finds in the member is of a valid type. The type of a
//!    referent in an external GML file is not checked.

use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet};

use fastxml::schema::types::{
    CompiledSchema, ComplexType, ContentModel, DerivationMethod, ElementDef, NsName, Particle,
    TypeDef,
};
use once_cell::sync::Lazy;
use quick_xml::{NsReader, events::Event};
use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_runtime::executor_operation::ExecutorContext;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Feature};
use regex::Regex;
use serde_json::Number;

use super::gml_id_uniqueness::{GmlIdTable, MemberGmlIds, Occurrence};
use super::report::{MemberReporter, Reporter};
use super::traversal::{CollectedXlinkEntry, CollectedXlinks};
use crate::citygml::gml_id;
use crate::citygml::v3::{CORE_NS, DYN_NS, GRP_NS, TRAN_NS, VERS_NS};
use crate::common::PlateauProfile;
use crate::common::schema_lookup::substitution_heads;

static GML_LINK_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.+\.gml)#(.+)").unwrap());

// Valid geometry object types for xlink references.
// Valid geometry objects shall be those listed in Standard Operating Procedure Annex B.3. v3. confirmed.
static VALID_GEOMETRY_TYPES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "gml:pos",
        "gml:posList",
        "gml:Point",
        "gml:MultiPoint",
        "gml:LineString",
        "gml:LinearRing",
        "gml:Polygon",
        "gml:OrientableSurface",
        "gml:MultiSurface",
        "gml:CompositeSurface",
        "gml:Solid",
        "gml:Triangle",
        "gml:TriangulatedSurface",
        "gml:Tin",
    ]
});

/// How the type of a referent is checked.
enum TargetCheck<'a> {
    /// The referent is one of [`VALID_GEOMETRY_TYPES`].
    GeometryTypes,
    /// T03 (association target type): the referent is of a target type
    /// (関連相手先の型) of the association (地物関連) in the XSDs.
    Associations(AssociationTargets<'a>),
    /// The XSDs cannot be read, so the type is not checked.
    Unchecked,
}

/// A city object group, whose xlinks are checked after all members.
struct CityObjectGroupInfo {
    feature_type: String,
    gml_id: String,
    xlinks: Vec<CollectedXlinkEntry>,
}

/// The gml:ids of each external GML file referred to, by its URI. A file that
/// cannot be read is not kept, so that a later file tries it again.
pub(super) type ExternalGmlIds = HashMap<String, HashSet<String>>;

/// The gml:id an xlink:href `#{gml:id}` within the file names.
fn local_gml_id(href: &str) -> &str {
    let mut chars = href.chars();
    chars.next();
    chars.as_str()
}

/// Where the referent of an xlink:href is.
enum Referent<'m> {
    /// An element of the member.
    Member(&'m Occurrence),
    /// An element of an external GML file, whose type is not kept.
    External,
}

/// The checks of the xlink:hrefs of a file.
pub(super) struct XlinkCheck<'a> {
    /// Referent existence: the xlink:hrefs with no referent.
    no_reference: u32,
    /// T03: the xlink:hrefs whose referent is not of a valid type.
    invalid_object_type: u32,
    /// Shared by all files.
    external_gml_ids: &'a mut ExternalGmlIds,
    /// The external GML files of this file that cannot be read, by their URI.
    unreadable_gml_files: HashSet<String>,
    city_object_groups: Vec<CityObjectGroupInfo>,
    target_check: TargetCheck<'a>,
    /// The associations, as `{holder}/{property}`, whose target type the
    /// XSDs do not give; their referents are not checked.
    unresolved_associations: BTreeSet<String>,
}

/// Whether the check needs the XSDs of the city GML: T03 (association target
/// type) checks the type of a referent against them for CityGML 3.0.
pub(super) fn needs_application_schema(profile: &PlateauProfile) -> bool {
    profile.is_citygml3()
}

impl<'a> XlinkCheck<'a> {
    /// The check of a file, sharing `external_gml_ids` with the other files.
    /// The type of a referent is checked against the fixed geometry types for
    /// CityGML 2.0, as the original implementation does, and against the XSDs
    /// `schema` for CityGML 3.0, unless they cannot be read.
    pub(super) fn new(
        external_gml_ids: &'a mut ExternalGmlIds,
        profile: &PlateauProfile,
        schema: Option<&'a CompiledSchema>,
    ) -> Self {
        let target_check = match schema {
            _ if !profile.is_citygml3() => TargetCheck::GeometryTypes,
            Some(schema) => TargetCheck::Associations(AssociationTargets::new(schema)),
            None => TargetCheck::Unchecked,
        };
        Self {
            no_reference: 0,
            invalid_object_type: 0,
            external_gml_ids,
            unreadable_gml_files: HashSet::new(),
            city_object_groups: Vec::new(),
            target_check,
            unresolved_associations: BTreeSet::new(),
        }
    }

    /// Warns of each association whose target type the XSDs do not give.
    pub(super) fn warn_unresolved_associations(&self, ctx: &ExecutorContext) {
        for association in &self.unresolved_associations {
            ctx.warn(
                DiagnosticDraft::new(ErrorCode::PlateauXlinkTargetUnresolved)
                    .with_message(format!("the XSDs give no target type of {association}")),
            );
        }
    }

    /// Keeps the xlinks of a city object group, checked by
    /// [`Self::check_city_object_groups`].
    pub(super) fn add_city_object_group(
        &mut self,
        feature_type: String,
        gml_id: String,
        xlinks: Vec<CollectedXlinkEntry>,
    ) {
        self.city_object_groups.push(CityObjectGroupInfo {
            feature_type,
            gml_id,
            xlinks,
        });
    }

    /// Runs both checks on the xlinks under a member, other than a city
    /// object group.
    pub(super) fn check_member(
        &mut self,
        storage_resolver: &StorageResolver,
        gml_ns: &str,
        city_gml_dir: Option<&Uri>,
        xlinks: &CollectedXlinks,
        member_gml_ids: &MemberGmlIds,
        report: &MemberReporter,
    ) {
        for xlink in &xlinks.entries {
            let referent = self.find_referent(
                storage_resolver,
                gml_ns,
                city_gml_dir,
                &xlink.href,
                member_gml_ids,
            );
            match referent {
                None => self.send_no_reference(xlink, report),
                Some(Referent::Member(occurrence)) => {
                    self.check_referent_type(xlink, occurrence, &xlinks.namespaces, report)
                }
                Some(Referent::External) => {}
            }
        }
    }

    /// Referent existence: the referent of `href` among the gml:ids of the
    /// member, `member_gml_ids`, or of the external GML file it names,
    /// relative to `city_gml_dir` unless absolute; `None` when there is none.
    fn find_referent<'m>(
        &mut self,
        storage_resolver: &StorageResolver,
        gml_ns: &str,
        city_gml_dir: Option<&Uri>,
        href: &str,
        member_gml_ids: &'m MemberGmlIds,
    ) -> Option<Referent<'m>> {
        let Some(caps) = GML_LINK_PATTERN.captures(href) else {
            return member_gml_ids.get(local_gml_id(href)).map(Referent::Member);
        };
        let gml_path = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let ext_gml_id = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
        external_gml_uri(city_gml_dir, gml_path)
            .and_then(|uri| self.external_gml_ids(storage_resolver, &uri, gml_ns))
            .is_some_and(|ids| ids.contains(ext_gml_id))
            .then_some(Referent::External)
    }

    /// T03: sends an error when the referent `occurrence` of `xlink` is not of
    /// a valid type.
    fn check_referent_type(
        &mut self,
        xlink: &CollectedXlinkEntry,
        occurrence: &Occurrence,
        namespaces: &HashMap<String, String>,
        report: &MemberReporter,
    ) {
        if self.is_valid_target(xlink, &occurrence.tag, namespaces) {
            return;
        }
        self.invalid_object_type += 1;
        report.send(
            "XLink_InvalidObjectType",
            [
                ("tag", xlink.tag.clone()),
                ("xpath", xlink.xpath.clone()),
                ("xlinkHref", xlink.href.clone()),
                ("refGmlId", local_gml_id(&xlink.href).to_string()),
                ("refGmlTag", occurrence.tag.clone()),
                ("refGmlXpath", occurrence.xpath.clone()),
            ],
        );
    }

    /// Whether the referent `referent_tag` of `xlink` is of a valid type.
    fn is_valid_target(
        &mut self,
        xlink: &CollectedXlinkEntry,
        referent_tag: &str,
        namespaces: &HashMap<String, String>,
    ) -> bool {
        let targets = match &mut self.target_check {
            TargetCheck::GeometryTypes => return VALID_GEOMETRY_TYPES.contains(&referent_tag),
            TargetCheck::Unchecked => return true,
            TargetCheck::Associations(targets) => targets,
        };
        let name = |qname: &str| ns_name(namespaces, qname);
        match targets.check(
            name(&xlink.holder_tag),
            name(&xlink.tag),
            &name(referent_tag),
        ) {
            TargetMatch::Matches => true,
            TargetMatch::Mismatches => false,
            TargetMatch::Unresolved => {
                self.unresolved_associations
                    .insert(format!("{}/{}", xlink.holder_tag, xlink.tag));
                true
            }
        }
    }

    /// The gml:ids of the external GML file `uri`, or `None` when it cannot be
    /// read.
    fn external_gml_ids(
        &mut self,
        storage_resolver: &StorageResolver,
        uri: &Uri,
        gml_ns: &str,
    ) -> Option<&HashSet<String>> {
        let key = uri.to_string();
        if self.unreadable_gml_files.contains(&key) {
            return None;
        }
        if !self.external_gml_ids.contains_key(&key) {
            let Some(ids) = read_gml_ids(storage_resolver, uri, gml_ns) else {
                self.unreadable_gml_files.insert(key);
                return None;
            };
            self.external_gml_ids.insert(key.clone(), ids);
        }
        self.external_gml_ids.get(&key)
    }

    /// Referent existence: sends an error for `xlink`, which has no referent.
    fn send_no_reference(&mut self, xlink: &CollectedXlinkEntry, report: &MemberReporter) {
        self.no_reference += 1;
        report.send(
            "XLink_NoReference",
            [
                ("tag", xlink.tag.clone()),
                ("xpath", xlink.xpath.clone()),
                ("xlinkHref", xlink.href.clone()),
            ],
        );
    }

    /// Referent existence of the xlinks of the city object groups, against the
    /// gml:ids of the whole file, `file_gml_ids`, unlike those of the other
    /// members. T03 does not check their referents.
    pub(super) fn check_city_object_groups(
        &mut self,
        file_gml_ids: &GmlIdTable,
        reporter: &Reporter,
    ) {
        for group in &self.city_object_groups {
            for xlink in &group.xlinks {
                if file_gml_ids.contains_key(local_gml_id(&xlink.href)) {
                    continue;
                }
                self.no_reference += 1;
                let mut feature = reporter.file_feature(&["index", "udxDirs", "name"]);
                for (key, value) in [
                    ("flag", "XLink_NoReference"),
                    ("tag", &xlink.tag),
                    ("xpath", &xlink.xpath),
                    ("xlinkHref", &xlink.href),
                    ("featureType", &group.feature_type),
                    ("gmlId", &group.gml_id),
                ] {
                    feature.insert(key, AttributeValue::String(value.to_string()));
                }
                reporter.send(feature);
            }
        }
    }

    pub(super) fn write_summary(&self, summary: &mut Feature) {
        summary.insert(
            "xlinkHasNoReference",
            AttributeValue::Number(Number::from(self.no_reference)),
        );
        summary.insert(
            "xlinkInvalidObjectType",
            AttributeValue::Number(Number::from(self.invalid_object_type)),
        );
    }
}

/// The namespace-qualified name of the element `qname`, whose prefix
/// `namespaces` binds.
fn ns_name(namespaces: &HashMap<String, String>, qname: &str) -> NsName {
    let (prefix, local) = qname.split_once(':').unwrap_or(("", qname));
    NsName::new(
        namespaces
            .get(prefix)
            .map(String::as_str)
            .unwrap_or_default(),
        local,
    )
}

/// The URI of the GML file `gml_path` an xlink names, or `None` when it is not
/// a file in the directory of the city GML `city_gml_dir`, such as `a.gml` or
/// `./a.gml`. No other file or URL is read, so that a city GML cannot make the
/// action read anything outside its directory.
fn external_gml_uri(city_gml_dir: Option<&Uri>, gml_path: &str) -> Option<Uri> {
    let name = gml_path.strip_prefix("./").unwrap_or(gml_path);
    if name.contains(['/', '\\', ':']) {
        tracing::debug!(
            href = gml_path,
            "DomainOfDefinitionValidator: not reading an xlink target outside the directory of the city GML"
        );
        return None;
    }
    city_gml_dir?.join(name).ok()
}

/// The gml:ids of all elements of the GML file `uri`, or `None` when it cannot
/// be read or ends with an element open.
fn read_gml_ids(
    storage_resolver: &StorageResolver,
    uri: &Uri,
    gml_ns: &str,
) -> Option<HashSet<String>> {
    let storage = storage_resolver.resolve(uri).ok()?;
    let content = storage.get_sync(uri.path().as_path()).ok()?;
    let mut reader = NsReader::from_reader(content.as_ref());
    let mut buf = Vec::new();
    let mut gml_ids = HashSet::new();
    let mut depth = 0usize;
    loop {
        match reader.read_event_into(&mut buf).ok()? {
            Event::Start(element) => {
                depth += 1;
                gml_ids.extend(gml_id(&reader, &element, gml_ns).ok()?);
            }
            Event::Empty(element) => {
                gml_ids.extend(gml_id(&reader, &element, gml_ns).ok()?);
            }
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Eof => return (depth == 0).then_some(gml_ids),
            _ => {}
        }
        buf.clear();
    }
}

/// The targets of the CityGML 3.0 associations whose property type has no
/// element content, such as `gml:ReferenceType`: the XSDs give them only in
/// `appinfo/gml:targetElement`.
const REFERENCE_TARGETS: &[(&str, &str, &str, &str)] = &[
    (CORE_NS, "generalizesTo", CORE_NS, "AbstractCityObject"),
    (CORE_NS, "relatedTo", CORE_NS, "AbstractCityObject"),
    (DYN_NS, "sensorLocation", CORE_NS, "AbstractCityObject"),
    (GRP_NS, "groupMember", CORE_NS, "AbstractCityObject"),
    (GRP_NS, "parent", CORE_NS, "AbstractCityObject"),
    (TRAN_NS, "predecessor", TRAN_NS, "TrafficSpace"),
    (TRAN_NS, "successor", TRAN_NS, "TrafficSpace"),
    (
        VERS_NS,
        "versionMember",
        CORE_NS,
        "AbstractFeatureWithLifespan",
    ),
    (
        VERS_NS,
        "oldFeature",
        CORE_NS,
        "AbstractFeatureWithLifespan",
    ),
    (
        VERS_NS,
        "newFeature",
        CORE_NS,
        "AbstractFeatureWithLifespan",
    ),
    (VERS_NS, "from", VERS_NS, "Version"),
    (VERS_NS, "to", VERS_NS, "Version"),
];

/// Whether a referent is of a target type of its association.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetMatch {
    Matches,
    Mismatches,
    /// The XSDs give no target type of the association.
    Unresolved,
}

/// T03 (association target type): the target types of the associations, by
/// the XSDs of the application schema.
struct AssociationTargets<'a> {
    schema: &'a CompiledSchema,
    /// The target elements of each association, by the element holding it and
    /// its property; `None` when unresolved.
    targets: HashMap<(NsName, NsName), Option<Vec<NsName>>>,
}

impl<'a> AssociationTargets<'a> {
    fn new(schema: &'a CompiledSchema) -> Self {
        Self {
            schema,
            targets: HashMap::new(),
        }
    }

    /// Whether `referent`, referred to by `property` of `holder`, is a target
    /// element of the association or substitutes for one.
    fn check(&mut self, holder: NsName, property: NsName, referent: &NsName) -> TargetMatch {
        let schema = self.schema;
        let targets = match self.targets.entry((holder, property)) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let (holder, property) = entry.key();
                let targets = association_targets(schema, holder, property);
                entry.insert(targets)
            }
        };
        let Some(targets) = targets else {
            return TargetMatch::Unresolved;
        };
        let matches = targets.contains(referent)
            || substitution_heads(schema, &referent.namespace_uri, &referent.local_name)
                .any(|head| targets.contains(head));
        if matches {
            TargetMatch::Matches
        } else {
            TargetMatch::Mismatches
        }
    }
}

/// The target elements of `property` of `holder`: the elements its type
/// holds, or those of [`REFERENCE_TARGETS`] when it holds none.
fn association_targets(
    schema: &CompiledSchema,
    holder: &NsName,
    property: &NsName,
) -> Option<Vec<NsName>> {
    let declaration = property_declaration(schema, holder, property)?;
    let mut targets = Vec::new();
    if let Some(TypeDef::Complex(property_type)) = element_type(schema, declaration) {
        for_each_child(schema, property_type, &mut |child| {
            targets.push(declared_name(child));
            false
        });
    }
    if targets.is_empty() {
        targets = REFERENCE_TARGETS
            .iter()
            .filter(|(ns, local, _, _)| {
                *ns == &*property.namespace_uri && *local == &*property.local_name
            })
            .map(|(_, _, ns, local)| NsName::new(*ns, *local))
            .collect();
    }
    (!targets.is_empty()).then_some(targets)
}

/// The declaration of `property` as a child of `holder`: a local one in the
/// type of `holder`, else the global one.
fn property_declaration<'s>(
    schema: &'s CompiledSchema,
    holder: &NsName,
    property: &NsName,
) -> Option<&'s ElementDef> {
    let global = || schema.element_ns(&property.namespace_uri, &property.local_name);
    let holder_type = schema
        .element_ns(&holder.namespace_uri, &holder.local_name)
        .and_then(|holder| element_type(schema, holder));
    let Some(TypeDef::Complex(holder_type)) = holder_type else {
        return global();
    };
    let mut local = None;
    for_each_child(schema, holder_type, &mut |child| {
        if declared_name(child) != *property {
            return false;
        }
        local = Some(child);
        true
    });
    match local {
        Some(child) if child.ref_ns.is_none() => Some(child),
        _ => global(),
    }
}

/// The type of an element declaration, named or anonymous.
fn element_type<'s>(schema: &'s CompiledSchema, element: &'s ElementDef) -> Option<&'s TypeDef> {
    match &element.type_ns {
        Some(name) => schema.type_ns(&name.namespace_uri, &name.local_name),
        None => element.inline_type.as_ref(),
    }
}

/// The name of an element an instance has for a child declaration.
fn declared_name(element: &ElementDef) -> NsName {
    if let Some(reference) = &element.ref_ns {
        return reference.clone();
    }
    let local = element
        .name
        .split_once(':')
        .map_or(element.name.as_str(), |(_, local)| local);
    NsName::new(element.namespace.as_deref().unwrap_or_default(), local)
}

/// Calls `f` on each child element declaration of `complex_type` and of its
/// base types up to the first restriction, until `f` returns true.
fn for_each_child<'s>(
    schema: &'s CompiledSchema,
    complex_type: &'s ComplexType,
    f: &mut impl FnMut(&'s ElementDef) -> bool,
) {
    let mut visited = HashSet::new();
    let mut current = Some(complex_type);
    while let Some(complex_type) = current {
        // A cyclic base type chain of a broken schema ends at its first
        // repeated type.
        if !visited.insert(std::ptr::from_ref(complex_type)) {
            return;
        }
        let done = match &complex_type.particle {
            Some(particle) => particle_children(particle, f),
            None => content_children(&complex_type.content).iter().any(&mut *f),
        };
        // A restriction restates the whole content, so its base adds no children.
        if done || complex_type.derivation == Some(DerivationMethod::Restriction) {
            return;
        }
        current = match schema.complex_base_def(complex_type) {
            Some(TypeDef::Complex(base)) => Some(base),
            _ => None,
        };
    }
}

/// Calls `f` on each element declaration of `particle`, until `f` returns
/// true; returns whether it did.
fn particle_children<'s>(
    particle: &'s Particle,
    f: &mut impl FnMut(&'s ElementDef) -> bool,
) -> bool {
    match particle {
        Particle::Element(element) => f(element),
        Particle::Wildcard(_) => false,
        Particle::Sequence { items, .. } | Particle::Choice { items, .. } => {
            items.iter().any(|item| particle_children(item, f))
        }
        Particle::All { elements, .. } => elements.iter().any(f),
    }
}

fn content_children(content: &ContentModel) -> &[ElementDef] {
    match content {
        ContentModel::Sequence(elements)
        | ContentModel::Choice(elements)
        | ContentModel::All(elements)
        | ContentModel::ComplexExtension { elements, .. } => elements,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use fastxml::schema::fetcher::{CachingFetcher, NoopFetcher, SchemaFetcher};
    use fastxml::schema::xsd;
    use pretty_assertions::assert_eq;

    use super::*;

    const GML_NS: &str = "http://www.opengis.net/gml/3.2";
    const BLDG_NS: &str = "http://www.opengis.net/citygml/building/3.0";

    /// A building subdivision holds `bldg:buildingRoom` and properties of
    /// types derived by restriction through its base type, and a traffic
    /// space refers to others by `gml:ReferenceType`.
    fn schema() -> CompiledSchema {
        let gml = br#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            targetNamespace="http://www.opengis.net/gml/3.2" elementFormDefault="qualified">
            <xs:element name="AbstractSurface" abstract="true"/>
            <xs:element name="Polygon" substitutionGroup="gml:AbstractSurface"/>
            <xs:element name="Shell" substitutionGroup="gml:AbstractSurface"/>
            <xs:element name="AbstractRing" abstract="true"/>
            <xs:element name="LinearRing" substitutionGroup="gml:AbstractRing"/>
            <xs:complexType name="SurfacePropertyType">
                <xs:sequence minOccurs="0"><xs:element ref="gml:AbstractSurface"/></xs:sequence>
            </xs:complexType>
            <xs:element name="surfaceMember" type="gml:SurfacePropertyType"/>
            <xs:complexType name="ReferenceType"><xs:sequence/></xs:complexType>
        </xs:schema>"#;
        let core = br#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
            targetNamespace="http://www.opengis.net/citygml/3.0">
            <xs:element name="AbstractCityObject" abstract="true"/>
        </xs:schema>"#;
        let tran = br#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
            xmlns:gml="http://www.opengis.net/gml/3.2"
            xmlns:core="http://www.opengis.net/citygml/3.0"
            xmlns:tran="http://www.opengis.net/citygml/transportation/3.0"
            targetNamespace="http://www.opengis.net/citygml/transportation/3.0"
            elementFormDefault="qualified">
            <xs:import namespace="http://www.opengis.net/gml/3.2" schemaLocation="gml.xsd"/>
            <xs:import namespace="http://www.opengis.net/citygml/3.0" schemaLocation="core.xsd"/>
            <xs:complexType name="TrafficSpaceType">
                <xs:sequence>
                    <xs:element name="predecessor" type="gml:ReferenceType" minOccurs="0"/>
                </xs:sequence>
            </xs:complexType>
            <xs:element name="TrafficSpace" type="tran:TrafficSpaceType" substitutionGroup="core:AbstractCityObject"/>
            <xs:element name="Road" substitutionGroup="core:AbstractCityObject"/>
        </xs:schema>"#;
        let building = br#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
            xmlns:core="http://www.opengis.net/citygml/3.0"
            xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
            targetNamespace="http://www.opengis.net/citygml/building/3.0"
            elementFormDefault="qualified">
            <xs:import namespace="http://www.opengis.net/citygml/3.0" schemaLocation="core.xsd"/>
            <xs:import namespace="http://www.opengis.net/citygml/transportation/3.0" schemaLocation="tran.xsd"/>
            <xs:element name="BuildingRoom" substitutionGroup="core:AbstractCityObject"/>
            <xs:complexType name="BuildingRoomPropertyType">
                <xs:sequence minOccurs="0"><xs:element ref="bldg:BuildingRoom"/></xs:sequence>
            </xs:complexType>
            <xs:element name="BuildingFurniture" substitutionGroup="core:AbstractCityObject"/>
            <xs:complexType name="SpacePropertyType">
                <xs:choice minOccurs="0">
                    <xs:element ref="bldg:BuildingRoom"/>
                    <xs:element ref="bldg:BuildingFurniture"/>
                </xs:choice>
            </xs:complexType>
            <xs:complexType name="RoomPropertyType">
                <xs:complexContent>
                    <xs:restriction base="bldg:SpacePropertyType">
                        <xs:choice minOccurs="0"><xs:element ref="bldg:BuildingRoom"/></xs:choice>
                    </xs:restriction>
                </xs:complexContent>
            </xs:complexType>
            <xs:complexType name="EmptySpacePropertyType">
                <xs:complexContent>
                    <xs:restriction base="bldg:SpacePropertyType"/>
                </xs:complexContent>
            </xs:complexType>
            <xs:complexType name="AbstractBuildingSubdivisionType">
                <xs:sequence>
                    <xs:element name="buildingRoom" type="bldg:BuildingRoomPropertyType" minOccurs="0"/>
                    <xs:element name="room" type="bldg:RoomPropertyType" minOccurs="0"/>
                    <xs:element name="emptySpace" type="bldg:EmptySpacePropertyType" minOccurs="0"/>
                </xs:sequence>
            </xs:complexType>
            <xs:complexType name="BuildingUnitType">
                <xs:complexContent>
                    <xs:extension base="bldg:AbstractBuildingSubdivisionType"><xs:sequence/></xs:extension>
                </xs:complexContent>
            </xs:complexType>
            <xs:element name="BuildingUnit" type="bldg:BuildingUnitType" substitutionGroup="core:AbstractCityObject"/>
        </xs:schema>"#;
        let fetcher = CachingFetcher::new(NoopFetcher);
        fetcher.seed("http://test/gml.xsd", gml.to_vec());
        fetcher.seed("http://test/core.xsd", core.to_vec());
        fetcher.seed("http://test/tran.xsd", tran.to_vec());
        let mut resolver = xsd::SchemaResolver::new(&fetcher);
        resolver
            .resolve_entry(building, "http://test/building.xsd")
            .unwrap();
        xsd::compile_schemas(resolver.take_all_schemas()).unwrap()
    }

    #[test]
    fn test_cyclic_base_types_end() {
        const NS: &str = "http://example.com/test";
        let mut schema = CompiledSchema::new();
        for (name, base) in [("AType", "BType"), ("BType", "AType")] {
            let mut complex_type = ComplexType::new(name);
            complex_type.base_type = Some(format!("t:{base}"));
            complex_type.base_ns = Some(NsName::new(NS, base));
            complex_type.content = ContentModel::ComplexExtension {
                base_type: format!("t:{base}"),
                elements: Vec::new(),
            };
            schema
                .types_ns
                .insert(NsName::new(NS, name), TypeDef::Complex(complex_type));
        }
        let mut holder = ElementDef::new("Holder");
        holder.type_ns = Some(NsName::new(NS, "AType"));
        schema.elements_ns.insert(NsName::new(NS, "Holder"), holder);

        let mut targets = AssociationTargets::new(&schema);
        assert_eq!(
            targets.check(
                NsName::new(NS, "Holder"),
                NsName::new(NS, "property"),
                &NsName::new(NS, "Referent"),
            ),
            TargetMatch::Unresolved
        );
    }

    #[test]
    fn test_target_types() {
        let schema = schema();
        let mut targets = AssociationTargets::new(&schema);
        let gml = |local| NsName::new(GML_NS, local);
        let bldg = |local| NsName::new(BLDG_NS, local);
        let tran = |local| NsName::new(TRAN_NS, local);
        let cases = [
            // A global property, whose type holds the abstract head.
            (
                gml("Shell"),
                gml("surfaceMember"),
                gml("Polygon"),
                TargetMatch::Matches,
            ),
            (
                gml("Shell"),
                gml("surfaceMember"),
                gml("Shell"),
                TargetMatch::Matches,
            ),
            (
                gml("Shell"),
                gml("surfaceMember"),
                gml("LinearRing"),
                TargetMatch::Mismatches,
            ),
            (
                gml("Shell"),
                gml("surfaceMember"),
                bldg("BuildingRoom"),
                TargetMatch::Mismatches,
            ),
            // A local property of the base type of the holder.
            (
                bldg("BuildingUnit"),
                bldg("buildingRoom"),
                bldg("BuildingRoom"),
                TargetMatch::Matches,
            ),
            (
                bldg("BuildingUnit"),
                bldg("buildingRoom"),
                bldg("BuildingUnit"),
                TargetMatch::Mismatches,
            ),
            // A property whose type is derived by restriction, which holds
            // none of the elements its base type drops.
            (
                bldg("BuildingUnit"),
                bldg("room"),
                bldg("BuildingRoom"),
                TargetMatch::Matches,
            ),
            (
                bldg("BuildingUnit"),
                bldg("room"),
                bldg("BuildingFurniture"),
                TargetMatch::Mismatches,
            ),
            (
                bldg("BuildingUnit"),
                bldg("emptySpace"),
                bldg("BuildingRoom"),
                TargetMatch::Unresolved,
            ),
            // A property of `gml:ReferenceType`.
            (
                tran("TrafficSpace"),
                tran("predecessor"),
                tran("TrafficSpace"),
                TargetMatch::Matches,
            ),
            (
                tran("TrafficSpace"),
                tran("predecessor"),
                tran("Road"),
                TargetMatch::Mismatches,
            ),
            // No declaration of the property.
            (
                bldg("BuildingUnit"),
                bldg("storey"),
                bldg("BuildingRoom"),
                TargetMatch::Unresolved,
            ),
        ];
        for (holder, property, referent, expected) in cases {
            assert_eq!(
                targets.check(holder.clone(), property.clone(), &referent),
                expected,
                "{holder:?}/{property:?} -> {referent:?}"
            );
        }
    }
}
