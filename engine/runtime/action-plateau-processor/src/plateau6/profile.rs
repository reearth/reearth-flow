use crate::citygml;
use crate::common::{PlateauProfile, IUR4_NAMESPACES};

/// Profile for PLATEAU 6 (CityGML 3.0 / i-UR 4.0 based).
pub(crate) static PLATEAU6: PlateauProfile = PlateauProfile {
    citygml: &citygml::CITYGML3,
    iur_namespaces: IUR4_NAMESPACES,
    action_prefix: "PLATEAU6",
};
