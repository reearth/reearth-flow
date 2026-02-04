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
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const reader = response.body?.getReader();
  if (!reader) {
    throw new Error("Response body is not readable");
  }

  const decoder = new TextDecoder("utf-8");
  const ready: T[][] = [];

  const flushLines = (final: boolean) => {
    while (true) {
      const nl = buffer.indexOf("\n");
      if (nl === -1) break;

      // Extract one line (without \n)
      let line = buffer.slice(0, nl);
      buffer = buffer.slice(nl + 1);

      // Handle Windows CRLF: strip trailing \r if present
      if (line.endsWith("\r")) line = line.slice(0, -1);

      const trimmed = line.trim();
      if (!trimmed) continue;

      try {
        currentBatch.push(JSON.parse(trimmed) as T);
        featuresProcessed++;
      } catch (e) {
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
      let last = buffer;
      buffer = "";

      if (last.endsWith("\r")) last = last.slice(0, -1);
      const trimmed = last.trim();

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

  const decompressor = new Decompress((chunk, final) => {
    const isFinal = final === true; // coerce undefined -> false
    buffer += decoder.decode(chunk, { stream: !isFinal });
    flushLines(isFinal);
  });

  try {
    while (true) {
      const { done, value } = await reader.read();

      if (done) {
        // Finalize decompression / flush final bytes + final line
        decompressor.push(new Uint8Array(0), true);

        // Yield any remaining ready batches
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

        // Always emit terminal completion event (prevents infinite load)
        yield {
          data: [],
          isComplete: true,
          progress: { bytesDownloaded, featuresProcessed },
        };
        break;
      }

      bytesDownloaded += value.length;

      decompressor.push(value);

      // Invoke progress callback on every chunk read, even if no batches have been yielded yet
      onProgress?.({ bytesDownloaded, featuresProcessed });

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
