use crate::shared::errors::AppError;
use crate::shared::result::AppResult;
use anyhow::Result;

pub fn normalize_id(value: &str) -> String {
    value
        .strip_suffix(":main")
        .unwrap_or(value)
        .trim()
        .to_string()
}

pub fn ensure_not_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::invalid_input(format!("{field} cannot be empty")))
    } else {
        Ok(())
    }
}

pub fn compress_brotli(uncompressed_data: &[u8], quality: i32, lgwin: i32) -> Result<Vec<u8>> {
    let mut input = std::io::Cursor::new(uncompressed_data);
    let mut compressed_data = Vec::new();

    let params = brotli::enc::BrotliEncoderParams {
        quality,
        lgwin,
        ..Default::default()
    };

    brotli::BrotliCompress(&mut input, &mut compressed_data, &params)?;
    Ok(compressed_data)
}

pub fn decompress_brotli(compressed_data: &[u8]) -> Result<Vec<u8>> {
    let mut input = std::io::Cursor::new(compressed_data);
    let mut decompressed_data = Vec::new();

    brotli::BrotliDecompress(&mut input, &mut decompressed_data)?;
    Ok(decompressed_data)
}

pub fn first_zero_bit(x: u32) -> u32 {
    (x + 1) & !x
}
