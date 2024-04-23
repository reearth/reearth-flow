use hashbrown::HashMap;
use nusamai_plateau::{
    appearance::{AppearanceStore, Material, Texture, Theme},
    Entity,
};

use crate::AttributeValue;

pub struct GeometryValue {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Appearance {
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    pub themes: HashMap<String, Theme>,
}

impl Appearance {
    pub fn new(
        textures: Vec<Texture>,
        materials: Vec<Material>,
        themes: HashMap<String, Theme>,
    ) -> Self {
        Self {
            textures,
            materials,
            themes,
        }
    }
}

impl From<AppearanceStore> for Appearance {
    fn from(store: AppearanceStore) -> Self {
        Self {
            textures: store.textures,
            materials: store.materials,
            themes: store.themes,
        }
    }
}

pub struct ActionGeometry {
    pub metadata: AttributeValue,
    pub appearance: Option<Appearance>,
}

impl From<Entity> for ActionGeometry {
    fn from(src: Entity) -> Self {
        let appearance = src.appearance_store.read().unwrap();
        Self {
            metadata: src.root.into(),
            appearance: Some(Appearance::new(
                appearance.textures.clone(),
                appearance.materials.clone(),
                appearance.themes.clone(),
            )),
        }
    }
}
