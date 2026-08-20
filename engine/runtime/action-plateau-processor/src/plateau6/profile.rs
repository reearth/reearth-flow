use crate::citygml;
use crate::common::PlateauProfile;

/// Profile for PLATEAU 6 (CityGML 3.0 / i-UR 4.0 based).
pub(crate) static PLATEAU6: PlateauProfile = PlateauProfile {
    citygml: &citygml::CITYGML3,
    uro_ns: "https://www.geospatial.jp/iur/uro/4.0",
    urf_ns: "https://www.geospatial.jp/iur/urf/4.0",
    action_prefix: "PLATEAU6",
};
