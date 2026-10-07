//! Draco compression of an already-assembled GLB, tuned for this writer's
//! flat-shaded meshes. Position and texture coordinate resolution follow
//! [`crate::draco::draco_config`].
//!
//! When the input GLB carries a `NORMAL` attribute, only its *seams* (the
//! per-polygon vertex splits [`super::glb`] produces) matter:
//! [`NormalEncoding::PredictedOnly`] discards the normal values and emits an
//! all-zero correction stream, so Draco reconstructs each face's normal from
//! the geometry on decode — normals cost effectively nothing while flat
//! shading is preserved. A GLB with no `NORMAL` is compressed as-is (the
//! setting is a no-op with no normal attribute to apply it to).

use draco_oxide::encode::NormalEncoding;
use draco_oxide::io::gltf::transcoder::{Error, GltfTranscoder, TranscoderConfig};

use crate::draco::{draco_config, DracoCompression};

/// Transcode an uncompressed GLB into a Draco-compressed GLB, predicting
/// per-face normals rather than storing them (see the module docs). `draco`
/// carries the position error bound, resolved against the box
/// `position_min`..`position_max` the glb positions span, and `texture_size` is
/// the largest dimension in pixels of the textures the glb references.
pub fn compress(
    glb: &[u8],
    draco: DracoCompression,
    texture_size: Option<u32>,
    position_min: [f64; 3],
    position_max: [f64; 3],
) -> Result<Vec<u8>, Error> {
    let transcoder = GltfTranscoder::new(TranscoderConfig {
        draco: draco_config(draco, texture_size, position_min, position_max)
            .with_normals(NormalEncoding::PredictedOnly),
    });
    let (compressed, warnings) = transcoder.transcode_to_glb(glb)?;
    for warning in warnings {
        tracing::warn!("Draco warning: {warning}");
    }
    Ok(compressed)
}
