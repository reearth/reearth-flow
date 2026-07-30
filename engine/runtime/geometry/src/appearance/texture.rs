//! Texture, sampler and raster types.
//!
//! A `Texture` is the image-plus-parameters layer over a shared `Raster`. The heavy resource is the
//! image, modelled as a `Raster` behind `Arc` so clones are cheap and an edit
//! copies only the one raster touched (copy-on-write via `Arc::make_mut`).
//! Splitting `Texture` from `Raster` lets one image back several textures with
//! different sampler / transform, and lets identical rasters dedup by shared
//! `Arc`.

use std::sync::Arc;

use bytes::Bytes;
use reearth_flow_common::image::MimeType;
use reearth_flow_common::uri::Uri;
use serde::{Deserialize, Serialize};

use super::ChannelId;

/// A shared image. Cheap to clone (`Arc<Raster>`); an edit emits a new feature
/// carrying a new `Arc<Raster>`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Raster {
    /// Owned, encoded image bytes, produced by a texture-editing action or a
    /// resolved URI.
    InMemory(RasterData),
    /// The image's own location (its source / identity); loaded lazily, owns no
    /// pixels. The scheme-aware `Uri` is one type across file / ram / gcs / http
    /// backends.
    Uri(Uri),
}

/// Owned, encoded image payload: the original bytes plus their image format.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RasterData {
    pub mime_type: MimeType,
    pub bytes: Bytes,
}

/// An image plus its sampling parameters, layered over a shared [`Raster`].
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Texture {
    /// Shared image; copy-on-write on edit.
    pub raster: Arc<Raster>,
    pub sampler: Sampler,
    /// Optional texture-coordinate transform (offset / rotation / scale).
    pub transform: Option<TextureTransform>,
    /// Which UV set within the resolved theme this map samples; 0 in the common
    /// case.
    pub uv_channel: ChannelId,
}

/// An affine transform applied to texture coordinates before sampling.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct TextureTransform {
    pub offset: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
}

/// How texture coordinates are wrapped and filtered.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sampler {
    pub wrap_s: WrapMode,
    pub wrap_t: WrapMode,
    pub mag_filter: Filter,
    pub min_filter: Filter,
}

impl Default for Sampler {
    /// A repeat-wrapped, linearly filtered sampler; the neutral default for a
    /// source that specifies no sampling parameters (e.g. CityGML with no
    /// `wrapMode`).
    fn default() -> Self {
        Self {
            wrap_s: WrapMode::Repeat,
            wrap_t: WrapMode::Repeat,
            mag_filter: Filter::Linear,
            min_filter: Filter::LinearMipmap,
        }
    }
}

/// Texture-coordinate wrap behaviour. The union of CityGML `wrapMode`
/// (none / wrap / mirror / clamp / border) and glTF sampler wrap.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WrapMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
    None,
}

/// Texture minification / magnification filter.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Linear,
    NearestMipmap,
    LinearMipmap,
}
