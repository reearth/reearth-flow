//! Parsing a GML `srsName` CRS reference to an [`EpsgCode`].

use reearth_flow_geometry::coordinate::EpsgCode;

/// The two OGC-standard encodings a CRS `srsName` can take, each an identifier
/// of the form `objectType`/`authority`/`version`/`code`.
enum SrsEncoding {
    /// OGC 07-092r3, e.g. `urn:ogc:def:crs:EPSG::6697`.
    Urn,
    /// OGC 09-048r3, e.g. `http://www.opengis.net/def/crs/EPSG/0/6697`.
    HttpUri,
}

impl SrsEncoding {
    /// The encoding of `srs_name` paired with its identifier body — the part
    /// after the fixed prefix — or `None` if it is neither standard encoding.
    fn detect(srs_name: &str) -> Option<(Self, &str)> {
        if let Some(body) = srs_name.strip_prefix("urn:ogc:def:") {
            Some((Self::Urn, body))
        } else if let Some(body) = srs_name
            .strip_prefix("http://www.opengis.net/def/")
            .or_else(|| srs_name.strip_prefix("https://www.opengis.net/def/"))
        {
            Some((Self::HttpUri, body))
        } else {
            None
        }
    }

    fn separator(&self) -> char {
        match self {
            Self::Urn => ':',
            Self::HttpUri => '/',
        }
    }
}

/// A CRS reference decomposed from a `srsName` into its OGC identifier parts.
struct CrsRef<'a> {
    object_type: &'a str,
    authority: &'a str,
    code: &'a str,
}

impl<'a> CrsRef<'a> {
    fn parse(srs_name: &'a str) -> Option<Self> {
        let (encoding, body) = SrsEncoding::detect(srs_name)?;
        let mut parts = body.split(encoding.separator());
        let object_type = parts.next()?;
        let authority = parts.next()?;
        let _version = parts.next()?;
        let code = parts.next()?;
        // Exactly four segments; a fifth means a compound or malformed reference.
        parts.next().is_none().then_some(Self {
            object_type,
            authority,
            code,
        })
    }

    fn epsg_code(&self) -> Option<EpsgCode> {
        if self.object_type.eq_ignore_ascii_case("crs")
            && self.authority.eq_ignore_ascii_case("EPSG")
        {
            self.code.parse::<u16>().ok().map(EpsgCode::new)
        } else {
            None
        }
    }
}

/// The EPSG code a CRS `srsName` references, or `None` if it isn't an EPSG CRS.
pub(super) fn parse_epsg(srs_name: &str) -> Option<EpsgCode> {
    CrsRef::parse(srs_name)?.epsg_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(srs_name: &str) -> Option<u16> {
        parse_epsg(srs_name).map(|e| e.get())
    }

    #[test]
    fn ogc_http_uri() {
        assert_eq!(code("http://www.opengis.net/def/crs/EPSG/0/6697"), Some(6697));
        assert_eq!(code("https://www.opengis.net/def/crs/EPSG/0/4326"), Some(4326));
    }

    #[test]
    fn ogc_urn() {
        assert_eq!(code("urn:ogc:def:crs:EPSG::6697"), Some(6697));
        assert_eq!(code("urn:ogc:def:crs:EPSG:9.9.1:6697"), Some(6697));
    }

    #[test]
    fn non_epsg_authority_rejected() {
        // Trailing number belongs to the OGC authority, not EPSG.
        assert_eq!(code("http://www.opengis.net/def/crs/OGC/0/84"), None);
        assert_eq!(code("http://www.opengis.net/def/crs/OGC/1.3/CRS84"), None);
    }

    #[test]
    fn non_crs_object_type_rejected() {
        assert_eq!(code("urn:ogc:def:coordinateOperation:EPSG::1234"), None);
    }

    #[test]
    fn compound_and_malformed_rejected() {
        assert_eq!(code("urn:ogc:def:crs,crs:EPSG::6697,crs:EPSG::5773"), None);
        assert_eq!(code("EPSG:6697"), None); // legacy short form is not an OGC identifier
        assert_eq!(code("nonsense"), None);
    }
}
