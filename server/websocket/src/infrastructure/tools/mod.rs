use anyhow::Result;
use brotli::{CompressorWriter, Decompressor};
use std::io::{Read, Write};

pub fn compress_brotli(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = CompressorWriter::new(Vec::new(), 4096, 11, 22);
    encoder.write_all(data)?;
    let compressed = encoder.into_inner();
    Ok(compressed)
}

pub fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = Decompressor::new(data, 4096);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
