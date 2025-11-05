import { Decompress } from "fzstd";

/**
 * Streams and decompresses a Zstandard-compressed JSONL file from a URL,
 * parsing features progressively in batches.
 *
 * @param url - URL of the compressed .jsonl.zst file
 * @param options - Streaming options
 * @returns AsyncGenerator yielding batches of parsed features
 */
export async function* streamDecompressZstdJsonl<T = any>(
  url: string,
  options: {
    batchSize?: number;
    signal?: AbortSignal;
    onProgress?: (progress: {
      bytesDownloaded: number;
      featuresProcessed: number;
    }) => void;
  } = {},
): AsyncGenerator<{ data: T[]; isComplete: boolean; progress: any }, void> {
  const { batchSize = 1000, signal, onProgress } = options;

  let decompressedBuffer = "";
  let featuresProcessed = 0;
  let bytesDownloaded = 0;
  let currentBatch: T[] = [];

  // Fetch the compressed file
  const response = await fetch(url, { signal });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error("Response body is not readable");
  }

  // Create streaming decompressor
  const decompressor = new Decompress((chunk, final) => {
    // Decode decompressed chunk to string
    const decoder = new TextDecoder("utf-8");
    const textChunk = decoder.decode(chunk, { stream: !final });
    decompressedBuffer += textChunk;

    // Process complete JSONL lines
    const lines = decompressedBuffer.split(/\r?\n/);

    // Keep the last (potentially incomplete) line in the buffer
    if (!final) {
      decompressedBuffer = lines.pop() || "";
    } else {
      decompressedBuffer = "";
    }

    // Parse complete lines
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;

      try {
        const parsed = JSON.parse(trimmed) as T;
        currentBatch.push(parsed);
        featuresProcessed++;
      } catch (error) {
        console.warn(
          "Failed to parse JSONL line:",
          trimmed.substring(0, 100),
          error,
        );
      }
    }
  });

  try {
    while (true) {
      const { done, value } = await reader.read();

      if (done) {
        // Push final chunk to decompressor
        decompressor.push(new Uint8Array(0), true);

        // Yield any remaining batch
        if (currentBatch.length > 0) {
          yield {
            data: [...currentBatch],
            isComplete: true,
            progress: {
              bytesDownloaded,
              featuresProcessed,
            },
          };
          currentBatch = [];
        }
        break;
      }

      // Update progress
      bytesDownloaded += value.length;

      // Push compressed chunk to decompressor
      decompressor.push(value);

      // Yield batch when it reaches the desired size
      if (currentBatch.length >= batchSize) {
        const progress = {
          bytesDownloaded,
          featuresProcessed,
        };

        onProgress?.(progress);

        yield {
          data: [...currentBatch],
          isComplete: false,
          progress,
        };

        currentBatch = [];
      }
    }
  } finally {
    reader.releaseLock();
  }
}
