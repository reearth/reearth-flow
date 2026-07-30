//! Atlas-page image codecs: encode an RGBA page and bind it into the glTF as a
//! texture. The [`Codec`] trait and its [`Builder`] caller live here with the
//! core-glTF codecs (PNG/JPEG). Codecs backed by a glTF *extension* (e.g. KTX2)
//! implement [`Codec`] from outside this core `glb` module.

use std::io::Cursor;

use image::RgbaImage;

use super::{Builder, ImageRef, SamplerDesc, TextureRef};

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
    /// Encode one atlas page.
    fn encode(&self, page: &RgbaImage) -> Result<Vec<u8>, CodecError>;
    /// Bind the embedded `image` into a glTF texture with `sampler`, using the
    /// builder's generic texture and extension primitives. Defaults to a core
    /// `source` reference; an extension-backed codec overrides this to declare
    /// its extension and build its own payload.
    fn bind_texture(
        &self,
        builder: &mut Builder,
        image: ImageRef,
        sampler: SamplerDesc,
    ) -> TextureRef {
        builder.push_texture(Some(image), sampler, Vec::new())
    }
}

impl Builder {
    /// Encode `page` with `codec`, embed it, and let the codec bind it as a
    /// glTF texture using `sampler`.
    pub fn push_atlas_texture(
        &mut self,
        page: &RgbaImage,
        codec: &dyn Codec,
        sampler: SamplerDesc,
    ) -> Result<TextureRef, CodecError> {
        let bytes = codec.encode(page)?;
        let image = self.push_image(&bytes, codec.mime());
        Ok(codec.bind_texture(self, image, sampler))
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
    fn encode(&self, page: &RgbaImage) -> Result<Vec<u8>, CodecError> {
        let mut bytes = Vec::new();
        let rgb = image::DynamicImage::ImageRgba8(page.clone()).to_rgb8();
        image::DynamicImage::ImageRgb8(rgb)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Jpeg)?;
        Ok(bytes)
    }
}
