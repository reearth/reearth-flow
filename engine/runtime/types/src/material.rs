use std::{hash::Hash, path::Path};

use indexmap::IndexSet;
use nusamai_citygml::Color;
use nusamai_gltf::nusamai_gltf_json::{BufferView, MimeType};
use serde::{Deserialize, Serialize};
use url::Url;

/// (CityGML's X3DMaterial)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct X3DMaterial {
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub ambient_intensity: f64,
}

impl From<nusamai_plateau::models::appearance::X3DMaterial> for X3DMaterial {
    fn from(src: nusamai_plateau::models::appearance::X3DMaterial) -> Self {
        Self {
            diffuse_color: src.diffuse_color.unwrap_or(Color::new(0.7, 0.7, 0.7)),
            specular_color: src.specular_color.unwrap_or(Color::new(0.04, 0.04, 0.04)),
            ambient_intensity: src.ambient_intensity.unwrap_or(0.9),
        }
    }
}

impl From<nusamai_plateau::appearance::Material> for X3DMaterial {
    fn from(src: nusamai_plateau::appearance::Material) -> Self {
        Self {
            diffuse_color: src.diffuse_color,
            specular_color: src.specular_color,
            ambient_intensity: src.ambient_intensity,
        }
    }
}

impl Default for X3DMaterial {
    fn default() -> Self {
        Self {
            diffuse_color: Color::new(0.7, 0.7, 0.7),
            specular_color: Color::new(0.04, 0.04, 0.04),
            ambient_intensity: 0.9,
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct Material {
    pub base_color: [f32; 4],
    pub base_texture: Option<Texture>,
}

impl Eq for Material {}

impl Hash for Material {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base_color.iter().for_each(|c| c.to_bits().hash(state));
        self.base_texture.hash(state);
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: None,
        }
    }
}

impl Material {
    pub fn to_gltf(
        &self,
        texture_set: &mut IndexSet<Texture, ahash::RandomState>,
    ) -> nusamai_gltf::nusamai_gltf_json::Material {
        let tex = if let Some(texture) = &self.base_texture {
            let (tex_idx, _) = texture_set.insert_full(texture.clone());
            Some(nusamai_gltf::nusamai_gltf_json::TextureInfo {
                index: tex_idx as u32,
                tex_coord: 0,
                ..Default::default()
            })
        } else {
            None
        };
        nusamai_gltf::nusamai_gltf_json::Material {
            pbr_metallic_roughness: Some(
                nusamai_gltf::nusamai_gltf_json::MaterialPbrMetallicRoughness {
                    base_color_factor: to_f64x4(self.base_color),
                    metallic_factor: 0.0,
                    roughness_factor: 0.9,
                    base_color_texture: tex,
                    ..Default::default()
                },
            ),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Texture {
    pub uri: Url,
}

impl Texture {
    pub fn to_gltf(
        &self,
        images: &mut IndexSet<Image, ahash::RandomState>,
    ) -> nusamai_gltf::nusamai_gltf_json::Texture {
        let (image_index, _) = images.insert_full(Image {
            uri: self.uri.clone(),
        });

        // Get the file extension
        let extension = Path::new(self.uri.path())
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        if extension == Some("webp".to_string()) {
            nusamai_gltf::nusamai_gltf_json::Texture {
                extensions: Some(
                    nusamai_gltf::nusamai_gltf_json::extensions::texture::TextureExtensions {
                        ext_texture_webp: Some(
                            nusamai_gltf::nusamai_gltf_json::extensions::texture::ExtTextureWebp {
                                source: image_index as u32,
                            },
                        ),
                        ..Default::default()
                    },
                ),
                source: Some(image_index as u32),
                ..Default::default()
            }
        } else {
            nusamai_gltf::nusamai_gltf_json::Texture {
                source: Some(image_index as u32),
                ..Default::default()
            }
        }
    }
}

impl From<nusamai_plateau::appearance::Texture> for Texture {
    fn from(texture: nusamai_plateau::appearance::Texture) -> Self {
        Self {
            uri: texture.image_url,
        }
    }
}

#[derive(Debug, Serialize, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Image {
    pub uri: Url,
}

impl Image {
    pub fn to_gltf(
        &self,
        buffer_views: &mut Vec<BufferView>,
        bin_content: &mut Vec<u8>,
    ) -> std::io::Result<nusamai_gltf::nusamai_gltf_json::Image> {
        if let Ok(path) = self.uri.to_file_path() {
            // NOTE: temporary implementation
            let (content, mime_type) = load_image(&path)?;

            buffer_views.push(BufferView {
                name: Some("image".to_string()),
                byte_offset: bin_content.len() as u32,
                byte_length: content.len() as u32,
                ..Default::default()
            });

            bin_content.extend(content);

            Ok(nusamai_gltf::nusamai_gltf_json::Image {
                mime_type: Some(mime_type),
                buffer_view: Some(buffer_views.len() as u32 - 1),
                ..Default::default()
            })
        } else {
            Ok(nusamai_gltf::nusamai_gltf_json::Image {
                uri: Some(self.uri.to_string()),
                ..Default::default()
            })
        }
    }
}

fn load_image(path: &Path) -> std::io::Result<(Vec<u8>, MimeType)> {
    if let Some(ext) = path.extension() {
        match ext.to_ascii_lowercase().to_str() {
            Some("tif" | "tiff" | "png") => {
                let image = image::open(path)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

                let mut writer = std::io::Cursor::new(Vec::new());
                let encoder = image::codecs::png::PngEncoder::new(&mut writer);
                image
                    .write_with_encoder(encoder)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
                Ok((writer.into_inner(), MimeType::ImagePng))
            }
            Some("jpg" | "jpeg") => Ok((std::fs::read(path)?, MimeType::ImageJpeg)),
            Some("webp") => Ok((std::fs::read(path)?, MimeType::ImageWebp)),
            _ => {
                let err = format!("Unsupported image format: {path:?}");
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
            }
        }
    } else {
        let err = format!("Unsupported image format: {path:?}");
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
}

fn to_f64x4(c: [f32; 4]) -> [f64; 4] {
    [
        f64::from(c[0]),
        f64::from(c[1]),
        f64::from(c[2]),
        f64::from(c[3]),
    ]
}
