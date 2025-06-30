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

pub fn load_image(path: &Path) -> std::io::Result<(Vec<u8>, MimeType)> {
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
