import * as fzstd from "fzstd";

/**
 * Decompresses Zstandard-compressed intermediate data (JSONL format).
 * Returns the decompressed JSONL string (newline-delimited JSON objects).
 *
 * @param compressedData - Uint8Array of compressed data
 * @returns Decompressed JSONL string
 */
export function decompressIntermediateData(
  compressedData: Uint8Array,
): string | undefined {
  try {
    const decompressed = fzstd.decompress(compressedData);
    const decoder = new TextDecoder("utf-8");
    const decompressedJsonl = decoder.decode(decompressed);

    console.log(
      `Decompressed ${compressedData.length} bytes -> ${decompressedJsonl.length} characters`,
    );
    return decompressedJsonl;
  } catch (err) {
    console.error("Decompression error:", err);
    return undefined;
  }
}
