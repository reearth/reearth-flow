//! Atlas-page image codecs: encode an RGBA page and bind it into the glTF as a
//! texture. The [`Codec`] trait and its [`Builder`] caller live here with the
//! core-glTF codecs (PNG/JPEG). Codecs backed by a glTF *extension* (e.g. KTX2)
//! implement [`Codec`] from outside this core `glb` module.

use std::io::Cursor;

use image::RgbaImage;

use super::{Builder, SamplerDesc, TextureRef};

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error(transparent)]
    Image(#[from] image::ImageError),
    /// An extension codec failed to encode (message already contextualized).
    #[error("{0}")]
    Encode(String),
}

/// One atlas-page image codec: how to encode a page and how it binds to a glTF
/// texture.
pub trait Codec {
    /// Atlas region alignment (texels) needed so no compression block straddles
    /// two packed regions. `1` for uncompressed codecs.
    fn block_align(&self) -> u32;
    /// glTF mime type of the encoded bytes.
    fn mime(&self) -> &'static str;
    /// Texture extension carrying the image when it has no core-glTF
    /// representation; `None` binds through the core `source`.
    fn image_extension(&self) -> Option<&'static str>;
    /// Encode one atlas page.
    fn encode(&self, page: &RgbaImage) -> Result<Vec<u8>, CodecError>;
}

impl Builder {
    /// Encode `page` with `codec` and bind it as a glTF texture using `sampler`,
    /// declaring the codec's extension when it has no core image representation.
    pub fn push_atlas_texture(
        &mut self,
        page: &RgbaImage,
        codec: &dyn Codec,
        sampler: SamplerDesc,
    ) -> Result<TextureRef, CodecError> {
        let bytes = codec.encode(page)?;
        let image = self.push_image(&bytes, codec.mime());
        let texture = match codec.image_extension() {
            Some(ext) => {
                self.require_extension(ext);
                self.push_texture(
                    None,
                    sampler,
                    vec![(ext, serde_json::json!({ "source": image.index() }))],
                )
            }
            None => self.push_texture(Some(image), sampler, Vec::new()),
        };
        Ok(texture)
    }
}

/// PNG: lossless, alpha preserved.
pub struct PngCodec;

impl Codec for PngCodec {
    fn block_align(&self) -> u32 {
        1
    }
    fn mime(&self) -> &'static str {
        "image/png"
    }
    fn image_extension(&self) -> Option<&'static str> {
        None
    }
    fn encode(&self, page: &RgbaImage) -> Result<Vec<u8>, CodecError> {
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(page.clone())
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
        Ok(bytes)
    }
}

/// JPEG: lossy, opaque (alpha is dropped).
pub struct JpegCodec;

impl Codec for JpegCodec {
    fn block_align(&self) -> u32 {
        1
    }
    fn mime(&self) -> &'static str {
        "image/jpeg"
    }
    fn image_extension(&self) -> Option<&'static str> {
        None
    }
    fn encode(&self, page: &RgbaImage) -> Result<Vec<u8>, CodecError> {
        let mut bytes = Vec::new();
        let rgb = image::DynamicImage::ImageRgba8(page.clone()).to_rgb8();
        image::DynamicImage::ImageRgb8(rgb)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Jpeg)?;
        Ok(bytes)
    }
}
