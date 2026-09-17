//! Draco encoder settings shared by the glb writers.

use draco_oxide::encode::{AttributeConfig, Config, Quantization};
use draco_oxide::{AttributeType, ConfigType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Whether mesh geometry is compressed with Draco, and how precisely.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DracoCompression {
    /// # Disabled
    /// Write mesh geometry uncompressed.
    Disabled,
    /// # Enabled
    /// Compress mesh geometry with Draco.
    #[serde(rename_all = "camelCase")]
    Enabled {
        /// # Quantization Error
        /// Upper bound, in meters, on how far compression may move a vertex. Must
        /// be positive. When unset, the encoder's default resolution is used.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        quantization_error: Option<f64>,
    },
}

impl DracoCompression {
    /// Compression with the encoder's default resolution.
    pub const DEFAULT_ENABLED: Self = Self::Enabled {
        quantization_error: None,
    };

    /// Whether the glb is compressed.
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled { .. })
    }
}

/// Upper bound, in texels of the referenced texture, on how far texture coordinate
/// quantization may move a vertex.
const TEXCOORD_MAX_ERROR_TEXELS: f64 = 0.5;

/// Builds the Draco encoder configuration. A requested position error bound is
/// resolved against the bounding box spanned by `position_min`..`position_max`, and
/// `texture_size` bounds the texture coordinate error to
/// [`TEXCOORD_MAX_ERROR_TEXELS`].
pub(crate) fn draco_config(
    draco: DracoCompression,
    texture_size: Option<u32>,
    position_min: [f64; 3],
    position_max: [f64; 3],
) -> Config {
    let mut config = <Config as ConfigType>::default();

    let position_bound = match draco {
        DracoCompression::Enabled {
            quantization_error: Some(max_error),
        } => {
            let box_is_valid = position_min
                .iter()
                .zip(&position_max)
                .all(|(lo, hi)| lo <= hi);
            (max_error > 0.0 && box_is_valid).then_some(max_error)
        }
        _ => None,
    };
    if let Some(max_error) = position_bound {
        let min = position_min.map(|v| v as f32);
        let max = position_max.map(|v| v as f32);
        config = config.with_attribute(
            AttributeType::Position,
            AttributeConfig {
                quantization: Some(Quantization::from_bounding_box(
                    &min,
                    &max,
                    (max_error * 2.0) as f32,
                )),
                ..Default::default()
            },
        );
    }

    if let Some(texture_size) = texture_size.filter(|size| *size > 0) {
        let max_error = 2.0 * TEXCOORD_MAX_ERROR_TEXELS / texture_size as f64;
        config = config.with_attribute(
            AttributeType::TextureCoordinate,
            AttributeConfig {
                quantization: Some(Quantization::MaxError(max_error as f32)),
                ..Default::default()
            },
        );
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn enabled(quantization_error: f64) -> DracoCompression {
        DracoCompression::Enabled {
            quantization_error: Some(quantization_error),
        }
    }

    fn resolved_quantization(
        draco: DracoCompression,
        min: [f64; 3],
        max: [f64; 3],
    ) -> Option<Quantization> {
        draco_config(draco, None, min, max)
            .attribute_config(draco_oxide::AttributeType::Position)
            .quantization
    }

    #[test]
    fn error_bound_resolves_against_the_whole_bounding_box() {
        let quantization = resolved_quantization(
            enabled(0.001),
            [-500.0, -20.0, -500.0],
            [500.0, 80.0, 500.0],
        )
        .unwrap();
        // Draco bounds the step, a vertex moves by at most half of one.
        assert_eq!(
            quantization,
            Quantization::Bounded {
                range: 1000.0,
                max_error: 0.002,
            }
        );

        // A primitive covering only part of the box quantizes no coarser than the bound.
        let bits = quantization.resolve(0.0);
        assert!(100.0 / ((1u64 << bits) - 1) as f32 / 2.0 <= 0.001);
    }

    fn resolved_texcoord_quantization(texture_size: Option<u32>) -> Option<Quantization> {
        draco_config(
            DracoCompression::DEFAULT_ENABLED,
            texture_size,
            [0.0; 3],
            [1.0; 3],
        )
        .attribute_config(AttributeType::TextureCoordinate)
        .quantization
    }

    #[test]
    fn texture_size_bounds_the_texcoord_error_to_half_a_texel() {
        let quantization = resolved_texcoord_quantization(Some(2048)).unwrap();
        assert_eq!(quantization, Quantization::MaxError(1.0 / 2048.0));

        // Coordinates spanning the texture once resolve to a step of one texel, so a
        // vertex moves by at most half of one; coordinates tiling it eight times get
        // the extra bits that keep the same accuracy in texels.
        let texels_per_step = |range: f32| {
            let bits = quantization.resolve(range);
            range * 2048.0 / ((1u64 << bits) - 1) as f32
        };
        assert_eq!(quantization.resolve(1.0), 12);
        assert!(texels_per_step(1.0) <= 1.0);
        assert_eq!(quantization.resolve(8.0), 15);
        assert!(texels_per_step(8.0) <= 1.0);
    }
}
