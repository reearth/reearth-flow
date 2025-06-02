use anyhow::Result;

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
