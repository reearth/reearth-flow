use super::parser::{RawChild, RawNode};
use super::utils::local_name;

pub(super) fn from_bounded_by(bounded_by: &RawNode) -> Option<&str> {
    bounded_by.children.iter().find_map(|child| {
        let RawChild::Element(e) = child else {
            return None;
        };
        if local_name(&e.name.0) != "Envelope" {
            return None;
        }
        e.attrs
            .iter()
            .find(|((q, _), _)| local_name(q) == "srsName")
            .map(|(_, v)| v.as_str())
    })
}

const EPSG_URI_PREFIX: &str = "http://www.opengis.net/def/crs/EPSG/0/";

pub(super) fn epsg_from_srs_name(srs_name: &str) -> Option<u16> {
    srs_name.strip_prefix(EPSG_URI_PREFIX)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epsg_from_opengis_uri() {
        assert_eq!(
            epsg_from_srs_name("http://www.opengis.net/def/crs/EPSG/0/6697"),
            Some(6697)
        );
    }

    #[test]
    fn epsg_non_epsg_uri_returns_none() {
        assert_eq!(epsg_from_srs_name("urn:ogc:def:crs:OGC:2:84"), None);
    }

    #[test]
    fn epsg_non_numeric_returns_none() {
        assert_eq!(epsg_from_srs_name("http://example.com/crs/WGS84"), None);
    }
}
