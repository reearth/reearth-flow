use crate::citygml;
use crate::common::{IUR3_NAMESPACES, PlateauProfile};

/// Profile for PLATEAU 4 (CityGML 2.0 / i-UR 3.x based).
pub(crate) static PLATEAU4: PlateauProfile = PlateauProfile {
    citygml: &citygml::CITYGML2,
    iur_namespaces: IUR3_NAMESPACES,
    action_prefix: "PLATEAU4",
};
