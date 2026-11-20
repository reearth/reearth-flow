//! T03: the xlink:hrefs with no referent, or whose referent is not a valid
//! geometry object.

use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use quick_xml::{NsReader, events::Event};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Feature};
use regex::Regex;
use serde_json::Number;

use super::gml_id_uniqueness::{GmlIdTable, MemberGmlIds};
use super::report::{MemberReporter, Reporter};
use super::traversal::CollectedXlinkEntry;
use crate::citygml::gml_id;

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

/// The T03 check of a file.
pub(super) struct XlinkCheck<'a> {
    no_reference: u32,
    invalid_object_type: u32,
    /// Shared by all files.
    external_gml_ids: &'a mut ExternalGmlIds,
    /// The external GML files of this file that cannot be read, by their URI.
    unreadable_gml_files: HashSet<String>,
    city_object_groups: Vec<CityObjectGroupInfo>,
}

impl<'a> XlinkCheck<'a> {
    pub(super) fn new(external_gml_ids: &'a mut ExternalGmlIds) -> Self {
        Self {
            no_reference: 0,
            invalid_object_type: 0,
            external_gml_ids,
            unreadable_gml_files: HashSet::new(),
            city_object_groups: Vec::new(),
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

    /// Checks the xlinks of a member, other than a city object group, against
    /// the gml:ids of the member, `member_gml_ids`, or of the external GML file
    /// the xlink names, relative to `city_gml_dir` unless absolute.
    pub(super) fn check_member(
        &mut self,
        storage_resolver: &StorageResolver,
        gml_ns: &str,
        city_gml_dir: Option<&Uri>,
        xlinks: &[CollectedXlinkEntry],
        member_gml_ids: &MemberGmlIds,
        report: &MemberReporter,
    ) {
        for xlink in xlinks {
            let href = &xlink.href;
            if let Some(caps) = GML_LINK_PATTERN.captures(href.as_str()) {
                let gml_path = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let ext_gml_id = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
                let found = external_gml_uri(city_gml_dir, gml_path)
                    .and_then(|uri| self.external_gml_ids(storage_resolver, &uri, gml_ns))
                    .is_some_and(|ids| ids.contains(ext_gml_id));
                if !found {
                    self.send_no_reference(xlink, report);
                }
                continue;
            }
            let gml_id = local_gml_id(href);
            let Some(occurrence) = member_gml_ids.get(gml_id) else {
                self.send_no_reference(xlink, report);
                continue;
            };
            if !VALID_GEOMETRY_TYPES.contains(&occurrence.tag.as_str()) {
                self.invalid_object_type += 1;
                report.send(
                    "XLink_InvalidObjectType",
                    [
                        ("tag", xlink.tag.clone()),
                        ("xpath", xlink.xpath.clone()),
                        ("xlinkHref", href.clone()),
                        ("refGmlId", gml_id.to_string()),
                        ("refGmlTag", occurrence.tag.clone()),
                        ("refGmlXpath", occurrence.xpath.clone()),
                    ],
                );
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

    /// Checks the xlinks of the city object groups against the gml:ids of the
    /// whole file, `file_gml_ids`, unlike those of the other members.
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
