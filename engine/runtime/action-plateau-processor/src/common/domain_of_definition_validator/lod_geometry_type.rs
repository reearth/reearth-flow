//! L-frn-01: the geometry object types described as lod{0-3}Geometry.

use reearth_flow_types::{AttributeValue, Feature};
use serde_json::Number;

use super::report::MemberReporter;
use super::traversal::LodEntry;

/// The level of `lod{0-3}Geometry` named `name`.
pub(super) fn parse_lod_geometry_level(name: &str) -> Option<u8> {
    let rest = name.strip_prefix("lod")?.strip_suffix("Geometry")?;
    let level: u8 = rest.parse().ok()?;
    if level <= 3 { Some(level) } else { None }
}

/// Whether the geometry `entry` describes has a type allowed for its parent.
fn is_valid(entry: &LodEntry) -> bool {
    let geometry = entry.gml_child_tag.as_str();
    if entry.parent_tag == "gen:GenericCityObject" {
        if entry.lod_level == 0 {
            [
                "gml:Point",
                "gml:MultiPoint",
                "gml:MultiCurve",
                "gml:MultiSurface",
            ]
            .contains(&geometry)
        } else {
            ["gml:MultiSurface", "gml:Solid"].contains(&geometry)
        }
    } else if entry.parent_tag.contains(":DmG") {
        // DmGeometricAttribute
        [
            "gml:Point",
            "gml:MultiPoint",
            "gml:MultiCurve",
            "gml:MultiSurface",
        ]
        .contains(&geometry)
    } else if entry.parent_tag.contains(":DmA") {
        // DmAnnotation
        geometry == "gml:Point"
    } else {
        ["gml:MultiSurface", "gml:Solid"].contains(&geometry)
    }
}

/// The L-frn-01 check of a file.
#[derive(Debug, Default)]
pub(super) struct LodGeometryTypeCheck {
    invalid: u32,
}

impl LodGeometryTypeCheck {
    pub(super) fn check_member(&mut self, entries: &[LodEntry], report: &MemberReporter) {
        for entry in entries.iter().filter(|entry| !is_valid(entry)) {
            self.invalid += 1;
            report.send(
                "InvalidLodXGeometry",
                [
                    ("parentTag", entry.parent_tag.clone()),
                    // Empty when the parent is the member itself.
                    ("gmlId", entry.parent_gml_id.clone()),
                    ("invalidGeometry", entry.gml_child_tag.clone()),
                ],
            );
        }
    }

    pub(super) fn write_summary(&self, summary: &mut Feature) {
        summary.insert(
            "invalidLodXGeometry",
            AttributeValue::Number(Number::from(self.invalid)),
        );
    }
}
