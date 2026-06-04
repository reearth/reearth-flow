use crate::citygml;
use crate::common::PlateauProfile;

/// Profile for PLATEAU 4 (CityGML 2.0 / i-UR 3.x based).
pub(crate) static PLATEAU4: PlateauProfile = PlateauProfile {
    citygml: &citygml::CITYGML2,
    uro_ns: "https://www.geospatial.jp/iur/uro/3.0",
    urf_ns: "https://www.geospatial.jp/iur/urf/3.0",
    action_prefix: "PLATEAU4",
};
