//! L05: the spatial reference system of the city model.

use once_cell::sync::Lazy;
use reearth_flow_types::{AttributeValue, Feature};

pub(super) static VALID_SRS_NAME_6697: &str = "http://www.opengis.net/def/crs/EPSG/0/6697";
pub(super) static VALID_SRS_NAME_6668: &str = "http://www.opengis.net/def/crs/EPSG/0/6668";

static VALID_SRS_NAME_FOR_UNF: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "http://www.opengis.net/def/crs/EPSG/0/10162",
        "http://www.opengis.net/def/crs/EPSG/0/10163",
        "http://www.opengis.net/def/crs/EPSG/0/10164",
        "http://www.opengis.net/def/crs/EPSG/0/10165",
        "http://www.opengis.net/def/crs/EPSG/0/10166",
        "http://www.opengis.net/def/crs/EPSG/0/10167",
        "http://www.opengis.net/def/crs/EPSG/0/10168",
        "http://www.opengis.net/def/crs/EPSG/0/10169",
        "http://www.opengis.net/def/crs/EPSG/0/10170",
        "http://www.opengis.net/def/crs/EPSG/0/10171",
        "http://www.opengis.net/def/crs/EPSG/0/10172",
        "http://www.opengis.net/def/crs/EPSG/0/10173",
        "http://www.opengis.net/def/crs/EPSG/0/10174",
    ]
});

/// The L05 check of a file.
#[derive(Debug, Default)]
pub(super) struct SrsNameCheck {
    /// The `srsName` of the `gml:Envelope`; empty until it is read.
    srs_name: String,
}

impl SrsNameCheck {
    pub(super) fn new(srs_name: String) -> Self {
        Self { srs_name }
    }

    pub(super) fn write_srs_name(&self, summary: &mut Feature) {
        summary.insert("srsName", AttributeValue::String(self.srs_name.clone()));
    }

    /// Writes whether the CRS is valid for `package`. When the document could
    /// not be parsed (e.g. an L01 not-well-formed error), the CRS could not be
    /// read; it is not an L05 error, as the only finding is the L01 one.
    pub(super) fn write_validity(&self, summary: &mut Feature, package: &str, parse_failed: bool) {
        let srs_name = self.srs_name.as_str();
        summary.insert(
            "isCorrectSrsName",
            AttributeValue::Bool(
                parse_failed
                    || srs_name == VALID_SRS_NAME_6697
                    || srs_name == VALID_SRS_NAME_6668
                    || (package == "unf" && VALID_SRS_NAME_FOR_UNF.contains(&srs_name)),
            ),
        );
    }
}
