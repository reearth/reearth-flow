//! KTX2 with Basis Universal supercompression (ETC1S or UASTC) — a glTF *extension*
//! codec (`KHR_texture_basisu`), so it lives outside the core `glb` module and
//! implements [`Codec`] from here.

use image::RgbaImage;
use ktx2_rw::{BasisCompressionParams, Ktx2Texture, VkFormat};

use super::glb::{Builder, Codec, CodecError, ImageRef, SamplerDesc, TextureRef};

/// Basis Universal supercompression scheme for [`Ktx2Codec`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Supercompression {
    /// ETC1S: smaller files, lower quality, slower to encode.
    #[default]
    Etc1s,
    /// UASTC: higher quality, larger files.
    Uastc,
}

/// KTX2/Basis codec, carrying a full mip chain built in linear light.
#[derive(Default)]
pub struct Ktx2Codec {
    /// Supercompression scheme applied by `compress_basis`.
    pub supercompression: Supercompression,
}

impl Codec for Ktx2Codec {
    fn block_align(&self) -> u32 {
        // ETC1S/UASTC compress in 4x4 blocks.
        4
    }
    fn mime(&self) -> &'static str {
        "image/ktx2"
    }
    fn bind_texture(
        &self,
        builder: &mut Builder,
        image: ImageRef,
        sampler: SamplerDesc,
    ) -> TextureRef {
        // KTX2 has no core-glTF representation, so the extension is required and
        // carries the image through its own `source`.
        builder.require_extension("KHR_texture_basisu");
        builder.push_texture(
            None,
            sampler,
            vec![(
                "KHR_texture_basisu",
                serde_json::json!({ "source": image.index() }),
            )],
        )
    }
    fn encode(&self, page: &RgbaImage) -> Result<Vec<u8>, CodecError> {
        let (width, height) = page.dimensions();
        let levels = 32 - width.max(height).leading_zeros(); // floor(log2(max)) + 1
        let err = |stage: &str, e: ktx2_rw::Error| CodecError::Encode(format!("KTX2 {stage}: {e}"));
        let mut texture =
            Ktx2Texture::create(width, height, 1, 1, 1, levels, VkFormat::R8G8B8A8Srgb)
                .map_err(|e| err("create", e))?;
        for (level, mip) in srgb_mip_chain(page).into_iter().enumerate() {
            texture
                .set_image_data(level as u32, 0, 0, mip.as_raw())
                .map_err(|e| err("set level", e))?;
        }
        // ETC1S quality is the 1-255 `quality_level` (128 is a good balance).
        // UASTC ignores `quality_level`; its encode effort is the pack level in
        // the low bits of `uastc_flags` (0 = fastest .. 4 = very slow). We use 2
        // (`PACK_UASTC_LEVEL_DEFAULT`), basisu's balanced default; raise toward 4
        // for quality or drop toward 0 for speed.
        const PACK_UASTC_LEVEL_DEFAULT: u32 = 2;
        // thread_count(1) avoids a job_pool destructor deadlock in the basisu
        // vendored by KTX-Software 4.4.0 (fixed upstream, not yet in our build).
        // See https://github.com/BinomialLLC/basis_universal/wiki/Release-Notes.
        // Cross-texture parallelism already comes from the rayon par_bridge that
        // drives encode(), so single-threaded per-texture loses no throughput.
        let params = match self.supercompression {
            Supercompression::Etc1s => BasisCompressionParams::builder()
                .uastc(false)
                .quality_level(128)
                .thread_count(1)
                .build(),
            Supercompression::Uastc => BasisCompressionParams::builder()
                .uastc(true)
                .uastc_flags(PACK_UASTC_LEVEL_DEFAULT)
                .thread_count(1)
                .build(),
        };
        texture
            .compress_basis(&params)
            .map_err(|e| err("compress", e))?;
        texture.write_to_memory().map_err(|e| err("write", e))
    }
}

/// Full mip chain (base down to 1x1) for an sRGB RGBA page. Each level is
/// resized from a linear-light copy of the base and re-encoded to sRGB, so
/// minified texels average correctly. Alpha stays linear at every step.
fn srgb_mip_chain(base: &RgbaImage) -> Vec<RgbaImage> {
    let (width, height) = base.dimensions();
    let levels = 32 - width.max(height).leading_zeros();
    let linear = to_linear(base);
    (0..levels)
        .map(|level| {
            if level == 0 {
                return base.clone();
            }
            let w = (width >> level).max(1);
            let h = (height >> level).max(1);
            from_linear(&image::imageops::resize(
                &linear,
                w,
                h,
                image::imageops::FilterType::Triangle,
            ))
        })
        .collect()
}

/// sRGB RGBA8 to linear-light RGBA (f32); RGB is gamma-expanded, alpha scaled.
fn to_linear(img: &RgbaImage) -> image::Rgba32FImage {
    image::ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y).0;
        image::Rgba([
            srgb_to_linear(p[0]),
            srgb_to_linear(p[1]),
            srgb_to_linear(p[2]),
            p[3] as f32 / 255.0,
        ])
    })
}

/// Inverse of [`to_linear`].
fn from_linear(img: &image::Rgba32FImage) -> RgbaImage {
    image::ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y).0;
        image::Rgba([
            linear_to_srgb(p[0]),
            linear_to_srgb(p[1]),
            linear_to_srgb(p[2]),
            (p[3] * 255.0).round().clamp(0.0, 255.0) as u8,
        ])
    })
}

fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> u8 {
    let c = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (c * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mip chain must run from the base resolution down to 1x1, halving each
    /// axis (floor, floored at 1). `encode` derives the KTX2 level count from
    /// the same formula independently, so an off-by-one here desyncs the two and
    /// corrupts the container.
    #[test]
    fn mip_chain_runs_from_base_to_1x1() {
        // Non-square, non-power-of-two on one axis, so a dimension-math slip shows.
        let dims: Vec<(u32, u32)> = srgb_mip_chain(&RgbaImage::new(8, 5))
            .iter()
            .map(|m| m.dimensions())
            .collect();
        // max(8,5)=8 -> floor(log2 8)+1 = 4 levels.
        assert_eq!(dims, vec![(8, 5), (4, 2), (2, 1), (1, 1)]);
    }

    /// `srgb_to_linear`/`linear_to_srgb` are this file's colour math. The round
    /// trip must recover every 8-bit value (within rounding) and hold the
    /// endpoints exactly, or textures shift in brightness.
    #[test]
    fn srgb_linear_roundtrip_recovers_all_bytes() {
        assert_eq!(linear_to_srgb(srgb_to_linear(0)), 0);
        assert_eq!(linear_to_srgb(srgb_to_linear(255)), 255);
        for v in 0u8..=255 {
            let back = linear_to_srgb(srgb_to_linear(v));
            assert!(
                back.abs_diff(v) <= 1,
                "sRGB round trip drifted: {v} -> {back}"
            );
        }
    }
}
