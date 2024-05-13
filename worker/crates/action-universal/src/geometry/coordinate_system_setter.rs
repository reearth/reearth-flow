use nusamai_projection::crs::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use reearth_flow_action::error::Error;
use reearth_flow_action::{
    ActionContext, ActionDataframe, ActionResult, AsyncAction, Dataframe, Feature, DEFAULT_PORT,
};

static SUPPORT_EPSG_CODE: Lazy<Vec<EpsgCode>> = Lazy::new(|| {
    vec![
        EPSG_WGS84_GEOGRAPHIC_2D,
        EPSG_WGS84_GEOGRAPHIC_3D,
        EPSG_WGS84_GEOCENTRIC,
        EPSG_JGD2011_GEOGRAPHIC_2D,
        EPSG_JGD2011_GEOGRAPHIC_3D,
        EPSG_JGD2011_JPRECT_I_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_II_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_III_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_IV_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_V_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_VI_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_VII_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_VIII_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_IX_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_X_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_XI_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_XII_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_XIII_JGD2011_HEIGHT,
        EPSG_JGD2011_JPRECT_I,
        EPSG_JGD2011_JPRECT_II,
        EPSG_JGD2011_JPRECT_III,
        EPSG_JGD2011_JPRECT_IV,
        EPSG_JGD2011_JPRECT_V,
        EPSG_JGD2011_JPRECT_VI,
        EPSG_JGD2011_JPRECT_VII,
        EPSG_JGD2011_JPRECT_VIII,
        EPSG_JGD2011_JPRECT_IX,
        EPSG_JGD2011_JPRECT_X,
        EPSG_JGD2011_JPRECT_XI,
        EPSG_JGD2011_JPRECT_XII,
        EPSG_JGD2011_JPRECT_XIII,
        EPSG_JGD2011_JPRECT_XIV,
        EPSG_JGD2011_JPRECT_XV,
        EPSG_JGD2011_JPRECT_XVI,
        EPSG_JGD2011_JPRECT_XVII,
        EPSG_JGD2011_JPRECT_XVIII,
        EPSG_JGD2011_JPRECT_XIX,
        EPSG_JGD2000_JPRECT_I,
        EPSG_JGD2000_JPRECT_II,
        EPSG_JGD2000_JPRECT_III,
        EPSG_JGD2000_JPRECT_IV,
        EPSG_JGD2000_JPRECT_V,
        EPSG_JGD2000_JPRECT_VI,
        EPSG_JGD2000_JPRECT_VII,
        EPSG_JGD2000_JPRECT_VIII,
        EPSG_JGD2000_JPRECT_IX,
        EPSG_JGD2000_JPRECT_X,
        EPSG_JGD2000_JPRECT_XI,
        EPSG_JGD2000_JPRECT_XII,
        EPSG_JGD2000_JPRECT_XIII,
        EPSG_JGD2000_JPRECT_XIV,
        EPSG_JGD2000_JPRECT_XV,
        EPSG_JGD2000_JPRECT_XVI,
        EPSG_JGD2000_JPRECT_XVII,
        EPSG_JGD2000_JPRECT_XVIII,
        EPSG_JGD2000_JPRECT_XIX,
        EPSG_TOKYO_JPRECT_I,
        EPSG_TOKYO_JPRECT_II,
        EPSG_TOKYO_JPRECT_III,
        EPSG_TOKYO_JPRECT_IV,
        EPSG_TOKYO_JPRECT_V,
        EPSG_TOKYO_JPRECT_VI,
        EPSG_TOKYO_JPRECT_VII,
        EPSG_TOKYO_JPRECT_VIII,
        EPSG_TOKYO_JPRECT_IX,
        EPSG_TOKYO_JPRECT_X,
        EPSG_TOKYO_JPRECT_XI,
        EPSG_TOKYO_JPRECT_XII,
        EPSG_TOKYO_JPRECT_XIII,
        EPSG_TOKYO_JPRECT_XIV,
        EPSG_TOKYO_JPRECT_XV,
        EPSG_TOKYO_JPRECT_XVI,
        EPSG_TOKYO_JPRECT_XVII,
        EPSG_TOKYO_JPRECT_XVIII,
        EPSG_TOKYO_JPRECT_XIX,
    ]
});

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CoordinateSystemSetter {
    distance: EpsgCode,
}

#[async_trait::async_trait]
#[typetag::serde(name = "CoordinateSystemSetter")]
impl AsyncAction for CoordinateSystemSetter {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        if !SUPPORT_EPSG_CODE.contains(&self.distance) {
            return Err(Error::input("Unsupported EPSG Code"));
        }
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;
        let result = input
            .features
            .iter()
            .flat_map(|feature| {
                ctx.action_log(format!("Processing feature: {}", feature.id));
                let Some(geometry) = &feature.geometry else {
                    return Some(feature.clone());
                };
                let mut geometry = geometry.clone();
                let mut feature = feature.clone();
                geometry.epsg = Some(self.distance);
                feature.geometry = Some(geometry);
                Some(feature)
            })
            .collect::<Vec<Feature>>();
        Ok(ActionDataframe::from([(
            DEFAULT_PORT.to_owned(),
            Dataframe::new(result),
        )]))
    }
}
