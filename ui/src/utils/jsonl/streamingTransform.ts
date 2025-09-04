import { intermediateDataTransform } from "./transformIntermediateData";

export type BatchTransformOptions = {
  /** Maximum features to process per batch */
  batchSize?: number;
  /** Sampling ratio (0-1) to reduce dataset size */
  samplingRatio?: number;
  /** Maximum total features to keep in memory */
  maxFeatures?: number;
  /** Callback for progress updates */
  onProgress?: (processed: number, total: number) => void;
};

export type BatchTransformResult = {
  transformedFeatures: any[];
  totalProcessed: number;
  sampledCount: number;
  hasMoreData: boolean;
};

/**
 * Transform a batch of intermediate data features for streaming processing
 */
export function transformBatch(
  features: any[],
  options: BatchTransformOptions = {},
): BatchTransformResult {
  const { samplingRatio = 1.0, maxFeatures, onProgress } = options;

  const transformedFeatures: any[] = [];
  let sampledCount = 0;

  for (let i = 0; i < features.length; i++) {
    const feature = features[i];

    // Apply sampling if specified
    if (samplingRatio < 1.0 && Math.random() > samplingRatio) {
      continue;
    }

    // Check max features limit
    if (maxFeatures && transformedFeatures.length >= maxFeatures) {
      return {
        transformedFeatures,
        totalProcessed: i,
        sampledCount,
        hasMoreData: true,
      };
    }

    // Transform the feature
    try {
      const transformed = intermediateDataTransform(feature);
      transformedFeatures.push(transformed);
      sampledCount++;
    } catch (error) {
      console.warn("Failed to transform feature:", error, feature);
      // Include raw feature as fallback
      transformedFeatures.push(feature);
      sampledCount++;
    }

    // Progress callback
    if (onProgress && i % 100 === 0) {
      onProgress(i + 1, features.length);
    }
  }

  // Final progress update
  if (onProgress) {
    onProgress(features.length, features.length);
  }

  return {
    transformedFeatures,
    totalProcessed: features.length,
    sampledCount,
    hasMoreData: false,
  };
}

/**
 * Progressive transformer for streaming data
 */
export class StreamingDataTransformer {
  private totalProcessed = 0;
  private totalSampled = 0;
  private options: BatchTransformOptions;

  constructor(options: BatchTransformOptions = {}) {
    this.options = options;
  }

  /**
   * Process a new batch of features
   */
  processBatch(features: any[]): BatchTransformResult {
    const result = transformBatch(features, {
      ...this.options,
      onProgress: (processed, total) => {
        this.options.onProgress?.(
          this.totalProcessed + processed,
          this.totalProcessed + total,
        );
      },
    });

    this.totalProcessed += result.totalProcessed;
    this.totalSampled += result.sampledCount;

    return {
      ...result,
      totalProcessed: this.totalProcessed,
      sampledCount: this.totalSampled,
    };
  }

  /**
   * Get current statistics
   */
  getStats() {
    return {
      totalProcessed: this.totalProcessed,
      totalSampled: this.totalSampled,
    };
  }

  /**
   * Reset transformer state
   */
  reset() {
    this.totalProcessed = 0;
    this.totalSampled = 0;
  }
}

/**
 * Detect geometry types in a batch and return statistics
 */
export function analyzeBatchGeometry(features: any[]) {
  const stats = {
    FlowGeometry2D: 0,
    FlowGeometry3D: 0,
    CityGmlGeometry: 0,
    Unknown: 0,
    total: features.length,
  };

  for (const feature of features) {
    const geometryValue = feature?.geometry?.value;

    if (!geometryValue) {
      stats.Unknown++;
      continue;
    }

    if (geometryValue.FlowGeometry2D || geometryValue.flowGeometry2D) {
      stats.FlowGeometry2D++;
    } else if (geometryValue.FlowGeometry3D || geometryValue.flowGeometry3D) {
      stats.FlowGeometry3D++;
    } else if (geometryValue.CityGmlGeometry) {
      stats.CityGmlGeometry++;
    } else {
      stats.Unknown++;
    }
  }

  return stats;
}

/**
 * Get the predominant geometry type from batch statistics
 */
export function getPredominantGeometryType(
  stats: ReturnType<typeof analyzeBatchGeometry>,
) {
  const { FlowGeometry2D, FlowGeometry3D, CityGmlGeometry, Unknown } = stats;

  const max = Math.max(
    FlowGeometry2D,
    FlowGeometry3D,
    CityGmlGeometry,
    Unknown,
  );

  if (max === 0) return null;
  if (max === FlowGeometry2D) return "FlowGeometry2D";
  if (max === FlowGeometry3D) return "FlowGeometry3D";
  if (max === CityGmlGeometry) return "CityGmlGeometry";
  return "Unknown";
}
