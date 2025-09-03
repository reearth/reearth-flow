export type StreamingProgress = {
  bytesProcessed: number;
  featuresProcessed: number;
  estimatedTotal?: number;
  percentage?: number;
}

export type StreamingState<T> = {
  data: T[];
  isStreaming: boolean;
  isComplete: boolean;
  progress: StreamingProgress;
  error: Error | null;
  hasMore: boolean;
}

export type StreamingOptions = {
  batchSize?: number; // Features per batch (default: 1000)
  chunkSize?: number; // Bytes to read per chunk (default: 64KB)
  onProgress?: (progress: StreamingProgress) => void;
  onBatch?: (batch: any[]) => void;
  onError?: (error: Error) => void;
  onComplete?: () => void;
  signal?: AbortSignal;
}

export type JsonlStreamResult<T> = {
  data: T[];
  progress: StreamingProgress;
  isComplete: boolean;
  hasMore: boolean;
}