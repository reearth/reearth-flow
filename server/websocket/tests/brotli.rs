use websocket::tools::{compress_brotli, decompress_brotli};

#[test]
fn test_compress_decompress_brotli() {
    let original_data = b"Hello, world! This is a test string for brotli compression.";

    let compressed = compress_brotli(original_data, 11, 22).unwrap();

    assert!(compressed.len() < original_data.len());

    let decompressed = decompress_brotli(&compressed).unwrap();
    assert_eq!(decompressed, original_data);
}

#[test]
fn test_compress_with_different_quality_levels() {
    let original_data = b"This is a longer test string to check compression with different quality levels. \
                         We expect higher quality to result in better compression but potentially take longer.";

    let compressed_min = compress_brotli(original_data, 0, 22).unwrap();

    let compressed_med = compress_brotli(original_data, 5, 22).unwrap();

    let compressed_max = compress_brotli(original_data, 11, 22).unwrap();

    assert_eq!(decompress_brotli(&compressed_min).unwrap(), original_data);
    assert_eq!(decompress_brotli(&compressed_med).unwrap(), original_data);
    assert_eq!(decompress_brotli(&compressed_max).unwrap(), original_data);
}

#[test]
fn test_compress_with_different_window_sizes() {
    let original_data =
        b"This is another test string to check compression with different window sizes.";

    let compressed_small = compress_brotli(original_data, 11, 10).unwrap();

    let compressed_med = compress_brotli(original_data, 11, 16).unwrap();

    let compressed_large = compress_brotli(original_data, 11, 22).unwrap();

    assert_eq!(decompress_brotli(&compressed_small).unwrap(), original_data);
    assert_eq!(decompress_brotli(&compressed_med).unwrap(), original_data);
    assert_eq!(decompress_brotli(&compressed_large).unwrap(), original_data);
}

#[test]
fn test_with_empty_data() {
    let empty_data = b"";

    let compressed = compress_brotli(empty_data, 11, 22).unwrap();

    let decompressed = decompress_brotli(&compressed).unwrap();
    assert_eq!(decompressed, empty_data);
}

#[test]
fn test_with_binary_data() {
    let binary_data: Vec<u8> = (0..255).collect();

    let compressed = compress_brotli(&binary_data, 11, 22).unwrap();

    let decompressed = decompress_brotli(&compressed).unwrap();
    assert_eq!(decompressed, binary_data);
}
