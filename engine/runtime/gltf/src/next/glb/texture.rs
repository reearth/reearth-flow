//! Image and texture construction: [`Builder::push_image`],
//! [`Builder::push_texture`], and the sampler description they accept.

use gltf::json;
use gltf::json::validation::Checked;

use super::Builder;

/// Handle to a texture pushed via [`Builder::push_texture`], for referencing
/// from a [`MaterialDesc`](super::MaterialDesc).
#[derive(Clone, Copy)]
pub struct TextureRef(json::Index<json::Texture>);

impl TextureRef {
    pub(super) fn index(self) -> json::Index<json::Texture> {
        self.0
    }
}

/// Handle to an image embedded via [`Builder::push_image`]. `index` is what a
/// texture extension references (e.g. `EXT_texture_webp`'s `source`).
#[derive(Clone, Copy)]
pub struct ImageRef(json::Index<json::Image>);

impl ImageRef {
    pub fn index(self) -> usize {
        self.0.value()
    }
}

/// How a texture's coordinates are wrapped and filtered — the glTF-agnostic
/// subset this writer needs. Maps to a glTF `sampler`.
#[derive(Clone, Copy)]
pub struct SamplerDesc {
    pub wrap_s: Wrap,
    pub wrap_t: Wrap,
    pub mag: MagFilter,
    pub min: MinFilter,
}

#[derive(Clone, Copy)]
pub enum Wrap {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
}

#[derive(Clone, Copy)]
pub enum MagFilter {
    Nearest,
    Linear,
}

#[derive(Clone, Copy)]
pub enum MinFilter {
    Nearest,
    Linear,
    NearestMipmap,
    LinearMipmap,
}

impl Builder {
    /// Embed `image_bytes` as a bufferView-backed image. A `mime_type` glTF
    /// admits only via an extension (e.g. `"image/webp"`) embeds fine; the
    /// caller references it through [`push_texture`](Self::push_texture).
    pub fn push_image(&mut self, image_bytes: &[u8], mime_type: &str) -> ImageRef {
        let buffer_view = self.push_buffer_view_targeted(image_bytes, None);
        ImageRef(self.root.push(json::Image {
            name: None,
            buffer_view: Some(buffer_view),
            mime_type: Some(json::image::MimeType(mime_type.to_string())),
            uri: None,
            extensions: Default::default(),
            extras: Default::default(),
        }))
    }

    /// Build a texture, attaching the given texture-level extension payloads and
    /// marking each `extensionsUsed`. `source` is `None` when an extension
    /// supplies the image (e.g. `EXT_texture_webp`), which the caller must then
    /// also [`require_extension`](Self::require_extension).
    pub fn push_texture(
        &mut self,
        source: Option<ImageRef>,
        sampler: SamplerDesc,
        extensions: Vec<(&'static str, serde_json::Value)>,
    ) -> TextureRef {
        let sampler_index = self.root.push(json::texture::Sampler {
            name: None,
            mag_filter: Some(Checked::Valid(match sampler.mag {
                MagFilter::Nearest => json::texture::MagFilter::Nearest,
                MagFilter::Linear => json::texture::MagFilter::Linear,
            })),
            min_filter: Some(Checked::Valid(match sampler.min {
                MinFilter::Nearest => json::texture::MinFilter::Nearest,
                MinFilter::Linear => json::texture::MinFilter::Linear,
                MinFilter::NearestMipmap => json::texture::MinFilter::NearestMipmapNearest,
                MinFilter::LinearMipmap => json::texture::MinFilter::LinearMipmapLinear,
            })),
            wrap_s: Checked::Valid(wrap(sampler.wrap_s)),
            wrap_t: Checked::Valid(wrap(sampler.wrap_t)),
            extensions: None,
            extras: Default::default(),
        });
        let mut ext = json::extensions::texture::Texture::default();
        for (name, value) in extensions {
            ext.others.insert(name.to_string(), value);
            self.mark_extension_used(name);
        }
        // Absent source serializes as the schema's skipped `u32::MAX` sentinel.
        let source = source.map_or_else(|| json::Index::new(u32::MAX), |image| image.0);
        TextureRef(self.root.push(json::Texture {
            name: None,
            sampler: Some(sampler_index),
            source,
            extensions: (!ext.others.is_empty()).then_some(ext),
            extras: Default::default(),
        }))
    }
}

fn wrap(w: Wrap) -> json::texture::WrappingMode {
    match w {
        Wrap::Repeat => json::texture::WrappingMode::Repeat,
        Wrap::MirroredRepeat => json::texture::WrappingMode::MirroredRepeat,
        Wrap::ClampToEdge => json::texture::WrappingMode::ClampToEdge,
    }
}
