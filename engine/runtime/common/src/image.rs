use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub enum MimeType {
    #[default]
    #[serde(rename = "image/jpeg")]
    ImageJpeg,
    #[serde(rename = "image/png")]
    ImagePng,
    #[serde(rename = "image/webp")]
    ImageWebp,
}

/// Largest decoded image [`open`] accepts, in bytes.
pub const MAX_DECODED_IMAGE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Decode the image at `path`, rejecting any whose decoded buffer exceeds
/// [`MAX_DECODED_IMAGE_BYTES`].
pub fn open(path: &Path) -> image::ImageResult<image::DynamicImage> {
    open_with_limit(path, MAX_DECODED_IMAGE_BYTES)
}

fn open_with_limit(path: &Path, max_alloc: u64) -> image::ImageResult<image::DynamicImage> {
    let mut reader = image::ImageReader::open(path)?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(max_alloc);
    reader.limits(limits);
    reader.decode()
}

pub fn load_image(path: &Path) -> std::io::Result<(Vec<u8>, MimeType)> {
    if let Some(ext) = path.extension() {
        match ext.to_ascii_lowercase().to_str() {
            Some("tif" | "tiff" | "png") => {
                let image = open(path)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

                let mut writer = std::io::Cursor::new(Vec::new());
                let encoder = image::codecs::png::PngEncoder::new(&mut writer);
                image
                    .write_with_encoder(encoder)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
                Ok((writer.into_inner(), MimeType::ImagePng))
            }
            Some("jpg" | "jpeg") => Ok((std::fs::read(path)?, MimeType::ImageJpeg)),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_png(dir: &Path, width: u32, height: u32) -> std::path::PathBuf {
        let path = dir.join("image.png");
        image::RgbImage::new(width, height).save(&path).unwrap();
        path
    }

    #[test]
    fn open_decodes_image_within_limit() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_png(dir.path(), 16, 16);
        let image = open_with_limit(&path, 16 * 16 * 3).unwrap();
        assert_eq!((image.width(), image.height()), (16, 16));
    }

    #[test]
    fn open_rejects_image_over_limit() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_png(dir.path(), 16, 16);
        let err = open_with_limit(&path, 16 * 16 * 3 - 1).unwrap_err();
        assert!(matches!(err, image::ImageError::Limits(_)), "{err}");
    }
}
