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
                sampler: Some(0), // Reference the sampler with proper mipmap filtering
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
                sampler: Some(0), // Reference the sampler with proper mipmap filtering
                source: Some(image_index as u32),
                ..Default::default()
            }
        }
    }
}

impl From<nusamai_plateau::appearance::Texture> for Texture {
    fn from(texture: nusamai_plateau::appearance::Texture) -> Self {
        Self {
            uri: resolve_unescaped_file_uri(texture.image_url),
        }
    }
}

/// Recover a texture whose file name contains an unescaped `#` or `?`.
///
/// Some datasets write `imageURI` as a raw file name (e.g. `appearance/#A.jpg`), which URI
/// parsing splits into a path and a fragment or query. If the URI as parsed does not point to
/// a file but the literal text does, the literal path is returned instead.
fn resolve_unescaped_file_uri(uri: Url) -> Url {
    if uri.scheme() != "file" || (uri.fragment().is_none() && uri.query().is_none()) {
        return uri;
    }
    let mut stripped = uri.clone();
    stripped.set_fragment(None);
    stripped.set_query(None);
    let Ok(path) = stripped.to_file_path() else {
        return uri;
    };
    if path.is_file() {
        return uri;
    }

    let decode = |s: &str| {
        percent_encoding::percent_decode_str(s)
            .decode_utf8_lossy()
            .into_owned()
    };
    let mut literal = path.into_os_string();
    if let Some(query) = uri.query() {
        literal.push("?");
        literal.push(decode(query));
    }
    if let Some(fragment) = uri.fragment() {
        literal.push("#");
        literal.push(decode(fragment));
    }
    let literal = std::path::PathBuf::from(literal);
    if !literal.is_file() {
        return uri;
    }
    match Url::from_file_path(&literal) {
        Ok(resolved) => {
            tracing::warn!(
                "texture URI {uri} contains an unescaped '#' or '?'; using file {}",
                literal.display()
            );
            resolved
        }
        Err(()) => uri,
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn setup(files: &[&str]) -> (tempfile::TempDir, Url) {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("appearance");
        std::fs::create_dir(&app).unwrap();
        for f in files {
            std::fs::write(app.join(f), b"").unwrap();
        }
        let gml = Url::from_file_path(dir.path().join("a.gml")).unwrap();
        (dir, gml)
    }

    fn resolve(gml: &Url, image_uri: &str) -> Url {
        resolve_unescaped_file_uri(gml.join(image_uri).unwrap())
    }

    #[test]
    fn test_hash_at_start_of_file_name() {
        let (dir, gml) = setup(&["#A.jpg"]);
        let resolved = resolve(&gml, "appearance/#A.jpg");
        assert_eq!(resolved.fragment(), None);
        assert_eq!(
            resolved.to_file_path().unwrap(),
            dir.path().join("appearance/#A.jpg")
        );
    }

    #[test]
    fn test_hash_and_space_in_middle_of_file_name() {
        let (dir, gml) = setup(&["Material #63.jpg"]);
        let resolved = resolve(&gml, "appearance/Material #63.jpg");
        assert_eq!(
            resolved.to_file_path().unwrap(),
            dir.path().join("appearance/Material #63.jpg")
        );
    }

    #[test]
    fn test_question_mark_and_hash_in_file_name() {
        let (dir, gml) = setup(&["a?b#c.jpg"]);
        let resolved = resolve(&gml, "appearance/a?b#c.jpg");
        assert_eq!(
            resolved.to_file_path().unwrap(),
            dir.path().join("appearance/a?b#c.jpg")
        );
    }

    #[test]
    fn test_fragment_on_existing_file_is_kept() {
        let (_dir, gml) = setup(&["a.jpg", "a.jpg#xywh=0,0,1,1"]);
        let uri = gml.join("appearance/a.jpg#xywh=0,0,1,1").unwrap();
        assert_eq!(resolve_unescaped_file_uri(uri.clone()), uri);
    }

    #[test]
    fn test_unresolvable_uri_is_unchanged() {
        let (_dir, gml) = setup(&[]);
        let uri = gml.join("appearance/#missing.jpg").unwrap();
        assert_eq!(resolve_unescaped_file_uri(uri.clone()), uri);
    }

    #[test]
    fn test_plain_uri_is_unchanged() {
        let (_dir, gml) = setup(&[]);
        let uri = gml.join("appearance/missing.jpg").unwrap();
        assert_eq!(resolve_unescaped_file_uri(uri.clone()), uri);
    }
}
