import type {
  StreamingOptions,
  StreamingProgress,
  JsonlStreamResult,
} from "./types";

export class JsonlStreamer<T = any> {
  private decoder = new TextDecoder("utf-8", { fatal: false });
  private buffer = "";
  private bytesProcessed = 0;
  private featuresProcessed = 0;
  private estimatedTotal?: number;
  constructor(private options: StreamingOptions = {}) {
    const {
      batchSize = 1000,
      chunkSize = 64 * 1024, // 64KB
    } = options;

    this.options = {
      ...options,
      batchSize,
      chunkSize,
    };
  }

  async *streamFromResponse(
    response: Response,
  ): AsyncGenerator<JsonlStreamResult<T>, void, unknown> {
    if (!response.body) {
      throw new Error("Response body is null");
    }

    const contentLength = response.headers.get("content-length");
    this.estimatedTotal = contentLength
      ? parseInt(contentLength, 10)
      : undefined;

    const reader = response.body.getReader();
    let batch: T[] = [];

    try {
      while (true) {
        // Check for abort signal
        if (this.options.signal?.aborted) {
          throw new DOMException("Stream aborted", "AbortError");
        }

        const { done, value } = await reader.read();

        if (done) {
          // Process any remaining data in buffer
          if (this.buffer.trim()) {
            const remainingFeatures = this.processBuffer();
            batch.push(...remainingFeatures);
            this.featuresProcessed += remainingFeatures.length;
          }

          // Yield final batch if it has data
          if (batch.length > 0) {
            const progress = this.getProgress();
            yield {
              data: batch,
              progress,
              isComplete: true,
              hasMore: false,
            };
          }
          break;
        }

        // Process the chunk
        this.bytesProcessed += value.length;
        const chunkText = this.decoder.decode(value, { stream: true });
        this.buffer += chunkText;

        // Extract complete lines and parse them
        const newFeatures = this.processBuffer();
        if (newFeatures.length > 0) {
          batch.push(...newFeatures);
          this.featuresProcessed += newFeatures.length;
        }

        // Emit batch when it reaches the desired size
        if (batch.length >= (this.options.batchSize || 1000)) {
          const progress = this.getProgress();

          // Notify progress callback
          this.options.onProgress?.(progress);

          yield {
            data: [...batch],
            progress,
            isComplete: false,
            hasMore: true,
          };

          // Notify batch callback
          this.options.onBatch?.(batch);

          // Clear batch for next iteration
          batch = [];
        }
      }


      this.options.onComplete?.();
    } catch (error) {
      const err = error as Error;
      this.options.onError?.(err);
      throw err;
    } finally {
      reader.releaseLock();
    }
  }

  private processBuffer(): T[] {
    const features: T[] = [];

    // Split by newlines but handle different line ending types
    const lines = this.buffer.split(/\r?\n/);

    // Keep the last line in buffer (might be incomplete)
    // Only keep it if it doesn't end with a newline
    const lastLine = lines.pop() || "";
    this.buffer = lastLine;

    // Process complete lines
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed) {
        try {
          const parsed = JSON.parse(trimmed);
          features.push(parsed);
        } catch (_error) {
          // Continue processing other lines rather than failing completely
          console.warn(
            "Failed to parse JSONL line:",
            trimmed.substring(0, 50) + "...",
          );
        }
      }
    }

    return features;
  }

  private getProgress(): StreamingProgress {
    const percentage = this.estimatedTotal
      ? Math.min((this.bytesProcessed / this.estimatedTotal) * 100, 100)
      : undefined;

    return {
      bytesProcessed: this.bytesProcessed,
      featuresProcessed: this.featuresProcessed,
      estimatedTotal: this.estimatedTotal,
      percentage,
    };
  }

  abort(): void {
    // The signal should be controlled externally via options.signal
    // This class doesn't manage its own AbortController
    throw new Error("Use the AbortController passed in options.signal to abort the stream");
  }
}

// Convenience function for simple streaming
export async function streamJsonl<T = any>(
  url: string,
  options: StreamingOptions = {},
): Promise<AsyncGenerator<JsonlStreamResult<T>, void, unknown>> {
  const controller = new AbortController();
  const signal = options.signal || controller.signal;

  const response = await fetch(url, { signal });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const streamer = new JsonlStreamer<T>({ ...options, signal });
  return streamer.streamFromResponse(response);
}
