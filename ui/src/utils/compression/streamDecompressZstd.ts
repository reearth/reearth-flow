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

  let buffer = "";
  let featuresProcessed = 0;
  let bytesDownloaded = 0;
  let currentBatch: T[] = [];

  const response = await fetch(url, { signal });
  if (!response.ok)
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);

  const reader = response.body?.getReader();
  if (!reader) throw new Error("Response body is not readable");

  // ✅ reuse decoder
  const decoder = new TextDecoder("utf-8");

  // queue of batches ready to yield
  const ready: T[][] = [];

  const flushLines = (final: boolean) => {
    // Extract complete lines from buffer without split-allocation storms
    while (true) {
      const idx = buffer.indexOf("\n");
      if (idx === -1) break;

      const line = buffer.slice(0, idx);
      buffer = buffer.slice(idx + 1);

      const trimmed = line.trim();
      if (!trimmed) continue;

      try {
        currentBatch.push(JSON.parse(trimmed) as T);
        featuresProcessed++;
      } catch (e) {
        // keep going; optionally count failures separately
        console.warn(
          "Failed to parse JSONL line:",
          trimmed.substring(0, 100),
          e,
        );
      }

      if (currentBatch.length >= batchSize) {
        ready.push(currentBatch);
        currentBatch = [];
      }
    }

    if (final) {
      const trimmed = buffer.trim();
      buffer = "";

      if (trimmed) {
        try {
          currentBatch.push(JSON.parse(trimmed) as T);
          featuresProcessed++;
        } catch (e) {
          console.warn(
            "Failed to parse final JSONL line:",
            trimmed.substring(0, 100),
            e,
          );
        }
      }

      if (currentBatch.length) {
        ready.push(currentBatch);
        currentBatch = [];
      }
    }
  };

  // Decompress callback: accumulate text + parse out lines
  const decompressor = new Decompress((chunk, final) => {
    buffer += decoder.decode(chunk, { stream: !final });
    if (final) {
      flushLines(final);
    }
  });

  try {
    while (true) {
      const { done, value } = await reader.read();

      if (done) {
        // final flush
        decompressor.push(new Uint8Array(0), true);

        // yield any remaining ready batches
        while (ready.length) {
          const batch = ready.shift();
          if (batch) {
            yield {
              data: batch,
              isComplete: false,
              progress: { bytesDownloaded, featuresProcessed },
            };
          }
        }

        // ✅ ALWAYS yield a terminal completion event
        yield {
          data: [],
          isComplete: true,
          progress: { bytesDownloaded, featuresProcessed },
        };
        break;
      }

      bytesDownloaded += value.length;

      // Push compressed chunk
      decompressor.push(value);

      // progress callback even if no batch emitted yet
      onProgress?.({ bytesDownloaded, featuresProcessed });

      // yield any ready batches
      while (ready.length) {
        const batch = ready.shift();

        if (batch) {
          yield {
            data: batch,
            isComplete: false,
            progress: { bytesDownloaded, featuresProcessed },
          };
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}
