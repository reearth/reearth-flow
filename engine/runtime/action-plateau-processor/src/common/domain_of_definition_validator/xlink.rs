//! T03: the xlink:hrefs with no referent, or whose referent is not a valid
//! geometry object.

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use once_cell::sync::Lazy;
use quick_xml::{NsReader, events::Event};
use reearth_flow_common::uri::{PROTOCOL_SEPARATOR, Uri};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{AttributeValue, Feature};
use regex::Regex;
use serde_json::Number;

use super::code_value::city_gml_dir;
use super::gml_id_uniqueness::GmlIdTable;
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

/// The T03 check of a file.
#[derive(Default)]
pub(super) struct XlinkCheck {
    no_reference: u32,
    invalid_object_type: u32,
    /// The gml:ids of each external GML file referred to, by the path the
    /// xlinks write; none for a file that cannot be read.
    external_file_to_gml_ids: HashMap<String, Vec<String>>,
    city_object_groups: Vec<CityObjectGroupInfo>,
}

impl XlinkCheck {
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
    /// the xlink names, relative to the city GML of `file` unless absolute.
    pub(super) fn check_member(
        &mut self,
        storage_resolver: &StorageResolver,
        gml_ns: &str,
        file: &Feature,
        xlinks: &[CollectedXlinkEntry],
        member_gml_ids: &GmlIdTable,
        report: &MemberReporter,
    ) {
        for xlink in xlinks {
            let href = &xlink.href;
            if let Some(caps) = GML_LINK_PATTERN.captures(href.as_str()) {
                let gml_path = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let ext_gml_id = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
                if !self.external_file_to_gml_ids.contains_key(gml_path) {
                    let gml_ids = external_gml_uri(file, gml_path)
                        .and_then(|uri| read_gml_ids(storage_resolver, &uri, gml_ns))
                        .unwrap_or_default();
                    self.external_file_to_gml_ids
                        .insert(gml_path.to_string(), gml_ids);
                }
                if !self
                    .external_file_to_gml_ids
                    .get(gml_path)
                    .is_some_and(|ids| ids.contains(&ext_gml_id.to_string()))
                {
                    self.send_no_reference(xlink, report);
                }
            } else if !member_gml_ids.contains_key(&href.chars().skip(1).collect::<String>()) {
                self.send_no_reference(xlink, report);
            } else if let Some(occurrences) =
                member_gml_ids.get(&href.chars().skip(1).collect::<String>())
            {
                for item in occurrences.iter().filter(|&item| {
                    !VALID_GEOMETRY_TYPES
                        .contains(&item.get("tag").map(|t| t.as_str()).unwrap_or_default())
                }) {
                    self.invalid_object_type += 1;
                    report.send(
                        "XLink_InvalidObjectType",
                        [
                            ("tag", xlink.tag.clone()),
                            ("xpath", xlink.xpath.clone()),
                            ("xlinkHref", href.clone()),
                            ("refGmlId", href.chars().skip(1).collect::<String>()),
                            ("refGmlTag", item.get("tag").cloned().unwrap_or_default()),
                            (
                                "refGmlXpath",
                                item.get("xpath").cloned().unwrap_or_default(),
                            ),
                        ],
                    );
                }
            }
        }
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
                if file_gml_ids.contains_key(&xlink.href.chars().skip(1).collect::<String>()) {
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

/// The URI of the GML file `gml_path` an xlink of the city GML of `file`
/// names, or `None` when it cannot be resolved.
fn external_gml_uri(file: &Feature, gml_path: &str) -> Option<Uri> {
    if gml_path.contains(PROTOCOL_SEPARATOR) || Path::new(gml_path).is_absolute() {
        return Uri::from_str(gml_path).ok();
    }
    city_gml_dir(file).ok()?.join(gml_path).ok()
}

/// The gml:ids of all elements of the GML file `uri`, or `None` when it cannot
/// be read.
fn read_gml_ids(
    storage_resolver: &StorageResolver,
    uri: &Uri,
    gml_ns: &str,
) -> Option<Vec<String>> {
    let storage = storage_resolver.resolve(uri).ok()?;
    let content = storage.get_sync(uri.path().as_path()).ok()?;
    let mut reader = NsReader::from_reader(content.as_ref());
    let mut buf = Vec::new();
    let mut gml_ids = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).ok()? {
            Event::Start(element) | Event::Empty(element) => {
                gml_ids.extend(gml_id(&reader, &element, gml_ns).ok()?);
            }
            Event::Eof => return Some(gml_ids),
            _ => {}
        }
        buf.clear();
    }
}
