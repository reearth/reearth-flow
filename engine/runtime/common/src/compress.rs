use crate::{
    str::{base64_decode_byte, base64_encode},
    Error, Result,
};

pub fn compress(source: &str) -> Result<String> {
    let compressed = zstd::encode_all(source.as_bytes(), 3)
        .map_err(|e| Error::Compress(format!("Failed to compress: {}", e)))?;
    Ok(base64_encode(&compressed))
}

pub fn decode<T: AsRef<[u8]>>(source: T) -> Result<String> {
    let bytes = base64_decode_byte(source)?;
    let decoded = zstd::decode_all(bytes.as_slice())
        .map_err(|e| Error::Compress(format!("Failed to decompress: {}", e)))?;
    Ok(String::from_utf8_lossy(&decoded).to_string())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_and_decode() {
        let original = "This is a test string to compress and decompress.";
        let compressed = compress(original).expect("Compression failed");
        let decompressed = decode(compressed).expect("Decompression failed");
        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_compress_empty_string() {
        let original = "";
        let compressed = compress(original).expect("Compression failed");
        let decompressed = decode(compressed).expect("Decompression failed");
        assert_eq!(original, decompressed);
    }
}
