use crate::{
    str::{base64_decode_byte, base64_encode},
    Error, Result,
};

pub fn compress(source: &str) -> Result<String> {
    let compressed = zstd::encode_all(source.as_bytes(), 3)
        .map_err(|e| Error::Compress(format!("Failed to compress: {e}")))?;
    Ok(base64_encode(&compressed))
}

pub fn decode<T: AsRef<[u8]>>(source: T) -> Result<String> {
    let bytes = base64_decode_byte(source)?;
    let decoded = zstd::decode_all(bytes.as_slice())
        .map_err(|e| Error::Compress(format!("Failed to decompress: {e}")))?;
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

    #[test]
    fn test_compress_japanese_text() {
        let original = "東京都渋谷区の建物データ";
        let compressed = compress(original).unwrap();
        let decompressed = decode(compressed).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_compress_large_text() {
        let original = "X".repeat(10000);
        let compressed = compress(&original).unwrap();
        let decompressed = decode(compressed).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_compress_plateau_xml() {
        let xml = r#"<?xml version="1.0"?>
<bldg:Building xmlns:bldg="http://www.opengis.net/citygml/building/2.0">
    <gml:name>テストビル</gml:name>
</bldg:Building>"#;
        
        let compressed = compress(xml).unwrap();
        let decompressed = decode(compressed).unwrap();
        assert_eq!(xml, decompressed);
    }

    #[test]
    fn test_compress_special_characters() {
        let original = "Special: \n\t\r<>&\"'";
        let compressed = compress(original).unwrap();
        let decompressed = decode(compressed).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_decode_invalid_base64() {
        let result = decode("not-valid-base64!!!");
        assert!(result.is_err());
    }
}

