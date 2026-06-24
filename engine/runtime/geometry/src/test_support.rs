//! Shared appearance test fixtures.
//!
//! The polygon, polygon-mesh and triangular-mesh constructor tests all need the
//! same handful of `Material` / `Texture` / `UvSource` builders. Keeping them in
//! one place means a new field on `Texture` or `PhongMaterial` is fixed once, not
//! in three drifting copies.

use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::image::MimeType;

use crate::appearance::{
    Filter, Material, PhongMaterial, Raster, RasterData, Sampler, Texture, ThemeId, UvSource,
    WrapMode,
};

/// A theme named `name`.
pub(crate) fn theme(name: &str) -> ThemeId {
    ThemeId(Arc::from(name))
}

/// A repeat/linear sampler.
pub(crate) fn sampler() -> Sampler {
    Sampler {
        wrap_s: WrapMode::Repeat,
        wrap_t: WrapMode::Repeat,
        mag_filter: Filter::Linear,
        min_filter: Filter::LinearMipmap,
    }
}

/// A 1-byte in-memory PNG texture on the default UV channel.
pub(crate) fn texture() -> Texture {
    Texture {
        raster: Arc::new(Raster::InMemory(RasterData {
            mime_type: MimeType::ImagePng,
            bytes: Bytes::from_static(&[0u8]),
        })),
        sampler: sampler(),
        transform: None,
        uv_channel: Default::default(),
    }
}

/// A white Phong material with `map` as its diffuse texture (if any).
pub(crate) fn phong(map: Option<Texture>) -> Material {
    Material::Phong(PhongMaterial {
        diffuse: [1.0, 1.0, 1.0],
        specular: [0.0; 3],
        emissive: [0.0; 3],
        ambient_intensity: 0.0,
        shininess: 0.0,
        transparency: 0.0,
        diffuse_map: map,
        emissive_map: None,
        normal_map: None,
    })
}

/// A textured material (diffuse map set), which requires a UV set.
pub(crate) fn textured() -> Material {
    phong(Some(texture()))
}

/// A colour-only material (no maps), which must not carry a UV set.
pub(crate) fn bare() -> Material {
    phong(None)
}

/// An `Explicit` UV source from `corners`.
pub(crate) fn explicit_uv(corners: &[[f64; 2]]) -> UvSource {
    UvSource::Explicit(corners.to_vec().into_boxed_slice())
}

/// An `Explicit` UV source of `n` zero corners (the length is what matters).
pub(crate) fn uv(n: usize) -> UvSource {
    explicit_uv(&vec![[0.0, 0.0]; n])
}

/// The four `[0,1]²` corners of a unit quad.
pub(crate) fn unit_quad_uv() -> UvSource {
    explicit_uv(&[[0., 0.], [1., 0.], [1., 1.], [0., 1.]])
}
